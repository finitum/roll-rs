FROM rust as builder
WORKDIR /build

RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

COPY . .

WORKDIR /build/wasm/
RUN wasm-pack build --target web

FROM nginxinc/nginx-unprivileged:mainline-alpine
COPY --from=builder /build/wasm/pkg /usr/share/nginx/html/pkg/
COPY --from=builder /build/wasm/index.html /usr/share/nginx/html/

