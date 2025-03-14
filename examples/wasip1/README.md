# Build
```sh
cargo build -p example-wasip1 --target wasm32-wasip1
```

# Run
```sh
npx @modelcontextprotocol/inspector wasmtime target/wasm32-wasip1/debug/example-wasip1.wasm
```
