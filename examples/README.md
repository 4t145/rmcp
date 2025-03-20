# Client Examples

- [Client SSE](clients/src/sse.rs), using reqwest and eventsource-client.
- [Client stdio](clients/src/std_io.rs), using tokio to spawn child process.
- [Everything](clients/src/everything_stdio.rs), test with `@modelcontextprotocol/server-everything`
- [Collection](clients/src/collection.rs), How to transpose service into dynamic object, so they will have a same type.

# Server Examples

- [Server SSE](clients/src/axum.rs), using axum as web server. 
- [Server stdio](clients/src/std_io.rs), using tokio async io. 


# Transport Examples

- [Tcp](transport/src/tcp.rs)

## Use Mcp Inspector
```sh
npx @modelcontextprotocol/inspector
```