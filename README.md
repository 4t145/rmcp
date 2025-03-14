# RMCP
A better and clean rust Model Context Protocol SDK implementation with tokio async runtime.

## Comparing to official SDK

The [Official SDK](https://github.com/modelcontextprotocol/rust-sdk/pulls) has too much limit and it was originally built for [goose](https://github.com/block/goose) rather than general using purpose.

All the features listed on specification would be implemented in this crate. And the first and most important thing is, this crate has the correct and intact data [types](crates/rmcp/src/model.rs). See it yourself. 

## Usage

### Import from github
```toml
rmcp = { git = "https://github.com/4t145/rust-mcp-sdk", features = ["server"] }
```

### Quick start

#### 1. Build a transport
- A transport type for client should be a `Sink` of `ClientJsonRpcMessage` and a `Stream` of `ServerJsonRpcMessage`
- A transport type for server should be a `Sink` of `ServerJsonRpcMessage` and a `Stream` of `ClientJsonRpcMessage`

We already have some transport type or builder function in [`rmcp::transport`](crates/rmcp/src/transport.rs).

```rust
use rmcp::transport::io::async_rw;
use tokio::io::{stdin, stdout};
let transport = async_rw(stdin(), stdout());
```

#### 2. Build a service
You can easily build a service by using [`ServerHandlerService`](crates/rmcp/src/handler/server.rs) or [`ClientHandlerService`](crates/rmcp/src/handler/client.rs).

```rust
use rmcp::ServerHandlerService;
let service = ServerHandlerService::new(common::counter::Counter::new());
```

You can reference the [server examples](examples/servers/src/common/counter.rs).

#### 3. Serve them together
```rust
// this call will finishe the initialization process
let server = rmcp::serve_server(service, transport).await?;
```

#### 4. Get remote interface by `peer()`
```rust
// request 
let roots = server.peer().list_roots().await?;

// or send notification
server.peer().notify_cancelled(...).await?;
```
For client, you will get server's api. And for server, you will get client api.

#### 5. Waiting for service shutdown
```rust
let quit_reason = server.waiting().await?;
// or cancel it
let quit_reason = server.cancel().await?;
```

### Examples
See [examples](examples/README.md)

### Features
- `client`: use client side sdk
- `server`: use server side sdk


## Related Resources
- [MCP Specification](https://spec.modelcontextprotocol.io/specification/2024-11-05/)

- [Schema](https://github.com/modelcontextprotocol/specification/blob/main/schema/2024-11-05/schema.ts)
