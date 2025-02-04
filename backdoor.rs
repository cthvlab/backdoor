use async_trait::async_trait;
use std::net::SocketAddr;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
use web_sys::{WebTransport, WebTransportDatagramDuplexStream};
#[cfg(not(target_arch = "wasm32"))]
use quinn::{ClientConfig, Endpoint, ServerConfig};
use tokio_tungstenite::{connect_async, accept_async, tungstenite::protocol::Message};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use futures_util::{StreamExt, SinkExt};
use url::Url;
use std::sync::Arc;

#[async_trait]
trait Transport {
    async fn connect(addr: &str) -> Result<Self, Box<dyn std::error::Error>> where Self: Sized;
    async fn send(&self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>>;
    async fn receive(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    async fn listen(addr: &str) -> Result<Self, Box<dyn std::error::Error>> where Self: Sized;
}

#[cfg(target_arch = "wasm32")]
struct WebTransportClient {
    transport: WebTransport,
}

#[cfg(target_arch = "wasm32")]
#[async_trait]
impl Transport for WebTransportClient {
    async fn connect(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let transport = WebTransport::new(&JsValue::from_str(addr))?;
        Ok(WebTransportClient { transport })
    }

    async fn send(&self, _data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    async fn receive(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(vec![])
    }
    
    async fn listen(_addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Err("Listening is not supported for WebTransport".into())
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct QuinnClient {
    endpoint: Endpoint,
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl Transport for QuinnClient {
    async fn connect(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let socket_addr: SocketAddr = addr.parse()?;
        let endpoint = Endpoint::client(ClientConfig::default())?;
        Ok(QuinnClient { endpoint })
    }

    async fn send(&self, _data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    async fn receive(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(vec![])
    }
    
    async fn listen(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let socket_addr: SocketAddr = addr.parse()?;
        let server_cfg = ServerConfig::default();
        let endpoint = Endpoint::server(server_cfg, socket_addr.into())?;
        Ok(QuinnClient { endpoint })
    }
}

struct WebSocketClient {
    socket: Arc<RwLock<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>>>,
}

#[async_trait]
impl Transport for WebSocketClient {
    async fn connect(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let url = Url::parse(addr)?;
        let (ws_stream, _) = connect_async(url).await?;
        Ok(WebSocketClient {
            socket: Arc::new(RwLock::new(ws_stream)),
        })
    }

    async fn send(&self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let mut socket = self.socket.write().await;
        socket.send(Message::Binary(data.to_vec())).await?;
        Ok(())
    }

    async fn receive(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut socket = self.socket.write().await;
        if let Some(Ok(msg)) = socket.next().await {
            return Ok(msg.into_data());
        }
        Ok(vec![])
    }

    async fn listen(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        println!("WebSocket server listening on {}", addr);
        let (stream, _) = listener.accept().await?;
        let ws_stream = accept_async(stream).await?;
        Ok(WebSocketClient {
            socket: Arc::new(RwLock::new(ws_stream)),
        })
    }
}

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
        let client = QuinnClient::connect("127.0.0.1:4433").await.unwrap();
        client.send(b"Hello, Server!").await.unwrap();
        let response = client.receive().await.unwrap();
        println!("Received: {:?}", response);

        let ws_client = WebSocketClient::connect("ws://127.0.0.1:8080").await.unwrap();
        ws_client.send(b"Hello via WebSocket!").await.unwrap();
        let ws_response = ws_client.receive().await.unwrap();
        println!("Received via WebSocket: {:?}", ws_response);
    }
}
