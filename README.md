```
cargo +nightly build --target wasm32-unknown-wasi --release

../wasmtime/target/release/wasmtime --dir=. target/wasm32-unknown-wasi/release/wasi-genetic.wasm 5000 500 0.4 0.001 0.3 cities.csv
```
