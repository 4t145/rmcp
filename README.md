# RMCP
[![Crates.io Version](https://img.shields.io/crates/v/rmcp)](https://crates.io/crates/rmcp)
![Release status](https://github.com/4t145/rmcp/actions/workflows/release.yml/badge.svg)
[![docs.rs](https://img.shields.io/docsrs/rmcp)](https://docs.rs/rmcp/latest/rmcp)

A better and clean rust Model Context Protocol SDK implementation with tokio async runtime.

## Comparing to official SDK

The [Official SDK](https://github.com/modelcontextprotocol/rust-sdk/pulls) has too much limit and it was originally built for [goose](https://github.com/block/goose) rather than general using purpose.

All the features listed on specification would be implemented in this crate. And the first and most important thing is, this crate has the correct and intact data [types](crates/rmcp/src/model.rs). See it yourself. 

## Usage

### Import from github
```toml
rmcp = { version = "0.1", features = ["server"] }
```

### Quick start

#### 1. Build a transport
The transport type must implemented [`IntoTransport`] trait, which allow split into a sink and a stream.

For client, the sink item is [`ClientJsonRpcMessage`](crate::model::ClientJsonRpcMessage) and stream item is [`ServerJsonRpcMessage`](crate::model::ServerJsonRpcMessage)

For server, the sink item is [`ServerJsonRpcMessage`](crate::model::ServerJsonRpcMessage) and stream item is [`ClientJsonRpcMessage`](crate::model::ClientJsonRpcMessage)

##### These types is automatically implemented [`IntoTransport`] trait
1. For type that already implement both [`Sink`] and [`Stream`] trait, they are automatically implemented [`IntoTransport`] trait
2. For tuple of sink `Tx` and stream `Rx`, type `(Tx, Rx)` are automatically implemented [`IntoTransport`] trait
3. For type that implement both [`tokio::io::AsyncRead`] and [`tokio::io::AsyncWrite`] trait, they are automatically implemented [`IntoTransport`] trait
4. For tulpe of [`tokio::io::AsyncRead`] `R `and [`tokio::io::AsyncWrite`] `W`, type `(R, W)` are automatically implemented [`IntoTransport`] trait


```rust
use rmcp::transport::io::async_rw;
use tokio::io::{stdin, stdout};
let transport = (stdin(), stdout());
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

### Use marcos to declaring tool
Use `toolbox` and `tool` macros to create tool quickly.

Check this [file](examples/servers/src/common/caculater.rs).
```rust
use rmcp::{ServerHandler, model::ServerInfo, schemars, tool, tool_box};

use super::counter::Counter;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SumRequest {
    #[schemars(description = "the left hand side number")]
    pub a: i32,
    pub b: i32,
}
#[derive(Debug, Clone)]
pub struct Calculater;
impl Calculater {
    // async function
    #[tool(description = "Calculate the sum of two numbers")]
    fn async sum(&self, #[tool(aggr)] SumRequest { a, b }: SumRequest) -> String {
        (a + b).to_string()
    }

    // sync function
    #[tool(description = "Calculate the sum of two numbers")]
    fn sub(
        &self,
        #[tool(param)]
        // this macro will transfer the schemars and serde's attributes
        #[schemars(description = "the left hand side number")]
        a: i32,
        #[tool(param)]
        #[schemars(description = "the left hand side number")]
        b: i32,
    ) -> String {
        (a - b).to_string()
    }

    // create a static toolbox to store the tool attributes
    tool_box!(Calculater { sum, sub });
}

impl ServerHandler for Calculater {
    // impl call_tool and list_tool by quering static toolbox
    tool_box!(@derive);
    
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("A simple caculator".into()),
            ..Default::default()
        }
    }
}
```
The only thing you should do is to make the function's return type implement `IntoCallToolResult`.

And you can just implement `IntoContents`, and the return value will be marked as success automatically. 

If you return a type of `Result<T, E>` where `T` and `E` both implemented `IntoContents`, it's also OK.

### Examples
See [examples](examples/README.md)

### Features
- `client`: use client side sdk
- `server`: use server side sdk


## Related Resources
- [MCP Specification](https://spec.modelcontextprotocol.io/specification/2024-11-05/)

- [Schema](https://github.com/modelcontextprotocol/specification/blob/main/schema/2024-11-05/schema.ts)
