# Client Examples

- [Client SSE](clients/src/sse.rs), using reqwest and eventsource-client.
- [Client stdio](clients/src/std_io.rs), using tokio to spawn child process.


# Server Examples

- [Server SSE](clients/src/axum.rs), using axum as web server. 
- [Server stdio](clients/src/std_io.rs), using tokio async io. 

## Use Mcp Inspector
```sh
npx @modelcontextprotocol/inspector
```