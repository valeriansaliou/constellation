FROM rustlang/rust:nightly-buster-slim AS build

RUN apt-get update
RUN apt-get install -y musl-tools

RUN rustup --version
RUN rustup target add x86_64-unknown-linux-musl

RUN rustc --version && \
    rustup --version && \
    cargo --version

WORKDIR /app
COPY . /app
RUN cargo clean && cargo build --release --target x86_64-unknown-linux-musl
RUN strip ./target/x86_64-unknown-linux-musl/release/constellation

FROM scratch

WORKDIR /usr/src/constellation

COPY --from=build /app/target/x86_64-unknown-linux-musl/release/constellation /usr/local/bin/constellation

CMD [ "constellation", "-c", "/etc/constellation.cfg" ]

EXPOSE 53/udp 8080
