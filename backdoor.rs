use async_trait::async_trait; // Позволяет определять асинхронные трейты
use std::net::SocketAddr; // Используется для хранения сетевого адреса
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture; // Обеспечивает поддержку асинхронного кода в WASM
#[cfg(target_arch = "wasm32")]
use web_sys::{WebTransport, WebTransportDatagramDuplexStream}; // WebTransport API для браузера
#[cfg(not(target_arch = "wasm32"))]
use quinn::{ClientConfig, Endpoint, ServerConfig}; // QUIC библиотека для нативных платформ
use tokio_tungstenite::{connect_async, accept_async}; // Подключение WebSocket клиента и сервера
use tokio::net::TcpListener; // Асинхронный TCP слушатель
use tokio::sync::Mutex; // Мьютекс для асинхронного доступа к данным
use futures_util::{StreamExt, SinkExt}; // Утилиты для работы с асинхронными потоками
use url::Url; // Разбор URL
use std::sync::Arc; // Arc позволяет разделять владение данными между потоками

#[async_trait]
trait Transport {
    async fn connect(addr: &str) -> Self; // Метод подключения к серверу
    async fn send(&self, data: &[u8]); // Метод отправки данных
    async fn receive(&self) -> Vec<u8>; // Метод получения данных
    async fn listen(addr: &str) -> Self; // Метод запуска сервера
}

#[cfg(target_arch = "wasm32")]
struct WebTransportClient {
    transport: WebTransport, // Клиент WebTransport
}

#[cfg(target_arch = "wasm32")]
#[async_trait]
impl Transport for WebTransportClient {
    async fn connect(addr: &str) -> Self {
        let transport = WebTransport::new(addr).unwrap(); // Создаем WebTransport-соединение
        WebTransportClient { transport }
    }

    async fn send(&self, data: &[u8]) {
        // Реализация отправки данных через WebTransport
    }

    async fn receive(&self) -> Vec<u8> {
        vec![] // Реализация приема данных через WebTransport
    }
    
    async fn listen(_addr: &str) -> Self {
        unimplemented!("Listening is handled via JavaScript in WebTransport") // WebTransport-сервер не поддерживается в Rust
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct QuinnClient {
    endpoint: Endpoint, // QUIC клиент
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl Transport for QuinnClient {
    async fn connect(addr: &str) -> Self {
        let socket_addr: SocketAddr = addr.parse().unwrap(); // Преобразуем строку в сетевой адрес
        let client_cfg = ClientConfig::default(); // Конфигурация QUIC клиента
        let endpoint = Endpoint::client("0.0.0.0:0".parse().unwrap()).expect("Failed to create client endpoint");
        QuinnClient { endpoint }
    }

    async fn send(&self, data: &[u8]) {
        // Реализация отправки данных через QUIC
    }

    async fn receive(&self) -> Vec<u8> {
        vec![] // Реализация приема данных через QUIC
    }
    
    async fn listen(addr: &str) -> Self {
        let socket_addr: SocketAddr = addr.parse().unwrap(); // Парсим адрес
        let server_cfg = ServerConfig::default(); // Конфигурация QUIC сервера
        let endpoint = Endpoint::server(server_cfg, socket_addr).expect("Failed to create server endpoint");
        QuinnClient { endpoint }
    }
}

struct WebSocketClient {
    socket: Arc<Mutex<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>>>, // WebSocket соединение с поддержкой многопоточности
}

#[async_trait]
impl Transport for WebSocketClient {
    async fn connect(addr: &str) -> Self {
        let url = Url::parse(addr).unwrap(); // Разбираем URL
        let (ws_stream, _) = connect_async(url).await.expect("Failed to connect to WebSocket"); // Подключаемся к WebSocket серверу
        WebSocketClient {
            socket: Arc::new(Mutex::new(ws_stream)),
        }
    }

    async fn send(&self, data: &[u8]) {
        let mut socket = self.socket.lock().await;
        socket.send(data.into()).await.expect("Failed to send WebSocket message"); // Отправляем данные по WebSocket
    }

    async fn receive(&self) -> Vec<u8> {
        let mut socket = self.socket.lock().await;
        if let Some(Ok(msg)) = socket.next().await {
            return msg.into_data(); // Получаем данные по WebSocket
        }
        vec![]
    }

    async fn listen(addr: &str) -> Self {
        let listener = TcpListener::bind(addr).await.expect("Failed to bind WebSocket server"); // Создаем WebSocket сервер
        println!("WebSocket server listening on {}", addr);
        let (stream, _) = listener.accept().await.expect("Failed to accept WebSocket connection"); // Принимаем входящее соединение
        let ws_stream = accept_async(stream).await.expect("Failed to accept WebSocket");
        WebSocketClient {
            socket: Arc::new(Mutex::new(ws_stream)),
        }
    }
}

#[tokio::main]
async fn main() {
    #[cfg(target_arch = "wasm32")]
    let client = WebTransportClient::connect("wss://127.0.0.1:4433").await; // Подключаемся через WebTransport

    #[cfg(not(target_arch = "wasm32"))]
    let client = QuinnClient::connect("127.0.0.1:4433").await; // Подключаемся через QUIC

    #[cfg(not(target_arch = "wasm32"))]
    let ws_client = WebSocketClient::connect("ws://127.0.0.1:8080").await; // Подключаемся через WebSocket

    client.send(b"Hello, Server!").await; // Отправляем данные
    let response = client.receive().await;
    println!("Received: {:?}", response); // Выводим ответ

    #[cfg(not(target_arch = "wasm32"))]
    ws_client.send(b"Hello via WebSocket!").await; // Отправляем данные по WebSocket
    let ws_response = ws_client.receive().await;
    println!("Received via WebSocket: {:?}", ws_response); // Выводим ответ WebSocket
}
