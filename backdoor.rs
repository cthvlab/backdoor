use async_trait::async_trait; // Для асинхронных трейтов.
use std::net::SocketAddr; // Для работы с сетевыми адресами.
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture; // Для работы с асинхронными вызовами в WebAssembly.
#[cfg(target_arch = "wasm32")]
use web_sys::{WebTransport, WebTransportDatagramDuplexStream}; // WebTransport для WASM.
#[cfg(not(target_arch = "wasm32"))]
use quinn::{ClientConfig, Endpoint, ServerConfig}; // QUIC для нативных приложений.
use tokio_tungstenite::{connect_async, accept_async, tungstenite::protocol::Message}; // WebSocket.
use tokio::net::TcpListener; // Для прослушивания TCP-соединений.
use tokio::sync::RwLock; // Асинхронная блокировка для безопасного доступа.
use futures_util::{StreamExt, SinkExt}; // Для работы с потоками и сокетами.
use url::Url; // Для парсинга URL.
use std::sync::Arc; // Для многопоточного доступа к данным.
use webrtc::api::APIBuilder; // Для создания WebRTC API.
use webrtc::peer_connection::{RTCConfiguration, RTCPeerConnection}; // Для работы с соединениями WebRTC.
use webrtc::data_channel::{DataChannel, DataChannelMessage}; // Для работы с каналами данных WebRTC.

#[async_trait]
trait Transport {
    async fn connect(addr: &str) -> Result<Self, Box<dyn std::error::Error>> where Self: Sized;
    async fn send(&self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>>;
    async fn receive(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    async fn listen(addr: &str) -> Result<Self, Box<dyn std::error::Error>> where Self: Sized;
}

// WebTransportClient для работы в WebAssembly (браузер).
#[cfg(target_arch = "wasm32")]
struct WebTransportClient {
    transport: WebTransport, // Структура для WebTransport.
}

#[cfg(target_arch = "wasm32")]
#[async_trait]
impl Transport for WebTransportClient {
    async fn connect(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let transport = WebTransport::new(&JsValue::from_str(addr))?; // Подключаем WebTransport.
        Ok(WebTransportClient { transport })
    }

    async fn send(&self, _data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        // В WebTransport пока нет реализации отправки данных в примере.
        Ok(())
    }

    async fn receive(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // В WebTransport пока нет реализации получения данных.
        Ok(vec![]) 
    }

    async fn listen(_addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // WebTransport не поддерживает слушание серверов в текущей реализации.
        Err("Listening is not supported for WebTransport".into())
    }
}

// QuinnClient для работы с QUIC (для нативных приложений).
#[cfg(not(target_arch = "wasm32"))]
struct QuinnClient {
    endpoint: Endpoint, // Точка подключения для QUIC.
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl Transport for QuinnClient {
    async fn connect(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Подключаемся к серверу через QUIC.
        let socket_addr: SocketAddr = addr.parse()?;
        let endpoint = Endpoint::client(ClientConfig::default())?;
        Ok(QuinnClient { endpoint })
    }

    async fn send(&self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        // Отправка данных через QUIC (реализуйте логику для отправки).
        Ok(())
    }

    async fn receive(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Получение данных через QUIC (реализуйте логику для получения).
        Ok(vec![]) 
    }

    async fn listen(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Создаем сервер QUIC для прослушивания.
        let socket_addr: SocketAddr = addr.parse()?;
        let server_cfg = ServerConfig::default();
        let endpoint = Endpoint::server(server_cfg, socket_addr.into())?;
        Ok(QuinnClient { endpoint })
    }
}

// WebRTCClient для работы с WebRTC.
struct WebRTCClient {
    peer_connection: RTCPeerConnection, // Соединение WebRTC.
    data_channel: Option<DataChannel>, // Канал передачи данных.
}

#[async_trait]
impl Transport for WebRTCClient {
    async fn connect(_addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Создаем API для WebRTC.
        let api = APIBuilder::new().build();
        let config = RTCConfiguration::default(); // Конфигурация по умолчанию.
        let peer_connection = api.new_peer_connection(config).await?; // Создаем новое соединение.

        // Создаем канал данных.
        let data_channel = peer_connection.create_data_channel("data", None).await?;

        Ok(WebRTCClient {
            peer_connection,
            data_channel: Some(data_channel),
        })
    }

    async fn send(&self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        // Отправка данных через WebRTC.
        if let Some(channel) = &self.data_channel {
            channel.send(DataChannelMessage::Binary(data.to_vec())).await?; // Отправка бинарных данных.
        }
        Ok(())
    }

    async fn receive(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Получение данных через WebRTC DataChannel.
        if let Some(channel) = &self.data_channel {
            let msg = channel.recv().await?; // Чтение сообщения.
            match msg {
                DataChannelMessage::Binary(data) => Ok(data), // Возвращаем бинарные данные.
                _ => Ok(vec![]), // Если сообщение не бинарное, возвращаем пустой вектор.
            }
        } else {
            Ok(vec![])
        }
    }

    async fn listen(_addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // WebRTC не поддерживает серверное слушание соединений в прямом смысле.
        Err("Listening is not supported for WebRTC".into())
    }
}

// WebSocketClient для работы с WebSockets.
struct WebSocketClient {
    socket: Arc<RwLock<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>>>, // Асинхронный WebSocket поток.
}

#[async_trait]
impl Transport for WebSocketClient {
    async fn connect(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Подключаемся к WebSocket-серверу.
        let url = Url::parse(addr)?;
        let (ws_stream, _) = connect_async(url).await?;
        Ok(WebSocketClient {
            socket: Arc::new(RwLock::new(ws_stream)),
        })
    }

    async fn send(&self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        // Отправляем данные через WebSocket.
        let mut socket = self.socket.write().await;
        socket.send(Message::Binary(data.to_vec())).await?; // Отправка бинарных данных.
        Ok(())
    }

    async fn receive(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Получаем данные через WebSocket.
        let mut socket = self.socket.write().await;
        if let Some(Ok(msg)) = socket.next().await {
            return Ok(msg.into_data());
        }
        Ok(vec![]) // Если данных нет, возвращаем пустой вектор.
    }

    async fn listen(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Слушаем входящие соединения для WebSocket.
        let listener = TcpListener::bind(addr).await?;
        println!("WebSocket server listening on {}", addr);
        let (stream, _) = listener.accept().await?;
        let ws_stream = accept_async(stream).await?;
        Ok(WebSocketClient {
            socket: Arc::new(RwLock::new(ws_stream)),
        })
    }
}

// Основная асинхронная функция.
#[tokio::main]
async fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        let client = WebTransportClient::connect("wss://127.0.0.1:4433").await.unwrap();
        client.send(b"Hello, Server!").await.unwrap();
        let response = client.receive().await.unwrap();
        println!("Received: {:?}", response);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Работаем с нативным приложением.
        let client = QuinnClient::connect("127.0.0.1:4433").await.unwrap();
        client.send(b"Hello, Server!").await.unwrap();
        let response = client.receive().await.unwrap();
        println!("Received: {:?}", response);

        // WebSocket клиент.
        let ws_client = WebSocketClient::connect("ws://127.0.0.1:8080").await.unwrap();
        ws_client.send(b"Hello via WebSocket!").await.unwrap();
        let ws_response = ws_client.receive().await.unwrap();
        println!("Received via WebSocket: {:?}", ws_response);

        // WebRTC клиент.
        let webrtc_client = WebRTCClient::connect("wss://127.0.0.1:4433").await.unwrap();
        webrtc_client.send(b"Hello via WebRTC!").await.unwrap();
        let webrtc_response = webrtc_client.receive().await.unwrap();
        println!("Received via WebRTC: {:?}", webrtc_response);
    }
}
