## Genetic Algorithm with WebAssembly, out of the browser

Ensure you have a compiled [wasmtime](https://github.com/CraneStation/wasmtime/).

To run:

```
cargo +nightly build --target wasm32-unknown-wasi --release

../wasmtime/target/release/wasmtime --dir=. target/wasm32-unknown-wasi/release/wasi-genetic.wasm 5000 500 0.4 0.001 0.3 cities.csv
```
