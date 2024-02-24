build web
`cargo build --release --target wasm32-unknown-unknown`
`wasm-bindgen --no-typescript --target web --out-dir ./public/ --out-name "gemblo" ./target/wasm32-unknown-unknown/release/gemblo.wasm`