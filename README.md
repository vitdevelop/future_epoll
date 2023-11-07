### Simple Rust future executor with linux epoll
[![Rust](https://github.com/vitdevelop/future_epoll/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/vitdevelop/future_epoll/actions/workflows/rust.yml)

Futures implemented

Time
- `wait(seconds)`

TCP
- `TcpServer::listen -> TcpServer`
- `TcpServer.accept -> TcpClient`
- `TcpClient.read -> usize`
- `TcpClient.write -> usize`

#### Usage:
##### Installation
```
git clone https://github.com/vitdevelop/future_epoll.git
cd future_epoll
```

##### Build release:
`cargo build --release`

##### Clean:
`cargo clean`

##### Run:
`cargo run --release`

##### Connect:
`telnet localhost 8080`
