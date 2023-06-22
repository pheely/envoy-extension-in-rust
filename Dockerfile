FROM scratch

COPY ./target/wasm32-wasi/release/add_authorization_header.wasm plugin.wasm