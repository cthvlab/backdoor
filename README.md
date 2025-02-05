# Backdoor Protocol - Универсальный Web 3 клиент-сервер
```
##############       
####       ###       
## #####   ### ###   
##   ###         ### 
##   ### ############
##   ###         ### 
##   ###   ### ###   
##   ###   ###       
#### #########       
   #####     
```

## Описание

Backdoor Protocol - это высокопроизводительный и безопасный клиент-сервер, разработанный на Rust, поддерживающий Web 3 технологии. Он предоставляет единое решение для различных транспортных протоколов: **QUIC (через TLS 1.3), WebTransport, WebSockets, WebRTC**, что делает его универсальным для любых платформ, включая **серверы, десктопные приложения, браузеры (через WASM)**.

## Возможности

- **Гибкость:** Работает как клиент и сервер.
- **Поддержка нескольких протоколов:** QUIC, WebTransport, WebSockets, WebRTC.
- **Web 3 Совместимость:** Поддержка децентрализованных решений и P2P.
- **Высокая производительность:** Использует асинхронные технологии Rust (`tokio`).
- **Безопасность:** TLS 1.3 для шифрования соединений.
- **Кроссплатформенность:** Работает на **Linux, Windows, MacOS, WASM (браузер)**.

## Установка

### Требования

- Rust (версия 1.65+)
- Cargo
- WebAssembly target (для браузерной сборки):
  ```sh
  rustup target add wasm32-unknown-unknown
  ```

### Сборка и запуск

#### Сервер

```sh
cargo run --release
```

#### Клиент

```sh
cargo run --release --example client
```

#### WASM (браузер)

```sh
wasm-pack build --target web
```

## Использование

### Запуск сервера QUIC

```rust
let server = QuinnClient::listen("127.0.0.1:4433").await;
```

### Запуск WebSocket сервера

```rust
let ws_server = WebSocketClient::listen("ws://127.0.0.1:8080").await;
```

### Подключение клиента WebSocket

```rust
let ws_client = WebSocketClient::connect("ws://127.0.0.1:8080").await;
```

### Подключение клиента WebTransport (для браузеров)

```rust
let web_transport_client = WebTransportClient::connect("wss://127.0.0.1:4433").await;
```

## Разработка и тестирование

Вы можете протестировать работу через WebSocket, QUIC или WebTransport, используя встроенные примеры.

### Запуск тестового клиента

```sh
cargo run --example client
```

### Запуск тестового сервера

```sh
cargo run --example server
```
Пример кода, который показывает использование WebSocket, QUIC и WebRTC:

```rust
#[tokio::main]
async fn main() {
    // WebSocket клиент
    let ws_client = WebSocketClient::connect("ws://127.0.0.1:8080").await.unwrap();
    ws_client.send(b"Hello via WebSocket!").await.unwrap();
    let ws_response = ws_client.receive().await.unwrap();
    println!("Received via WebSocket: {:?}", ws_response);

    // WebRTC клиент
    let webrtc_client = WebRTCClient::connect("wss://127.0.0.1:4433").await.unwrap();
    webrtc_client.send(b"Hello via WebRTC!").await.unwrap();
    let webrtc_response = webrtc_client.receive().await.unwrap();
    println!("Received via WebRTC: {:?}", webrtc_response);

    // QUIC клиент
    let quinn_client = QuinnClient::connect("127.0.0.1:4433").await.unwrap();
    quinn_client.send(b"Hello via QUIC!").await.unwrap();
    let quinn_response = quinn_client.receive().await.unwrap();
    println!("Received via QUIC: {:?}", quinn_response);
}
```



## Контрибьюции

Мы всегда рады новым идеям и улучшениям. Если вы хотите внести свой вклад в развитие, создайте Pull Request в репозитории на GitHub.

## Авторы

Разработано сообществом ЮАИ [yuai.ru](https://yuai.ru) 
