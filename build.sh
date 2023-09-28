cargo build --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/HARwash.wasm --out-dir ./pkg --web
