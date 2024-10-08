FROM --platform=${BUILDPLATFORM} rust:latest AS build

WORKDIR /usr/local/build

COPY . .

# install spin
RUN curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash

RUN rustup target add wasm32-wasi

RUN ./spin build

# reference: https://www.fermyon.com/blog/spin-in-docker
FROM scratch

COPY --from=build /usr/local/build/spin.toml /spin.toml
COPY --from=build /usr/local/build/target/wasm32-wasi/release/absurd_smov_proxy.wasm /target/wasm32-wasi/release/absurd_smov_proxy.wasm

ENTRYPOINT [ "/spin.toml" ]

# build command: docker buildx build --provenance=false -t absurd-smov-proxy .
# run command: docker run --rm -d --runtime=io.containerd.spin.v2 --platform=wasi/wasm --name absurd-smov-proxy -p 3000:80 absurd-smov-proxy