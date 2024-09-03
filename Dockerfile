FROM rust:latest AS build

WORKDIR /usr/local/build

COPY . .

# install spin
RUN curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash

RUN rustup target add wasm32-wasi

RUN ./spin build

FROM --platform=wasi/wasm scratch

COPY --from=build /usr/local/build/target/wasm32-wasi/release/absurd_smov_proxy.wasm /usr/local/app/entrypoint.wasm

ENTRYPOINT [ "/usr/local/app/entrypoint.wasm" ]