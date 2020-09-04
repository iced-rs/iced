cargo build --release --target wasm32-unknown-unknown
wasm-bindgen ../../target/wasm32-unknown-unknown/release/todos.wasm --out-dir ../../target/web --web
cp web/index.html ../../target/web/
