FROM rust:1.64 as builder
WORKDIR /build

RUN cargo install wasm-pack

COPY . .

WORKDIR /build/wasm/
RUN wasm-pack build --target web

FROM nginxinc/nginx-unprivileged:mainline-alpine
COPY --from=builder /build/wasm/pkg /usr/share/nginx/html/pkg/
COPY --from=builder /build/wasm/index.html /usr/share/nginx/html/

