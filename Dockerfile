FROM rustlang/rust:nightly AS build

RUN cargo install constellation-server

FROM debian:stretch-slim

WORKDIR /usr/src/constellation

COPY --from=build /usr/local/cargo/bin/constellation /usr/local/bin/constellation

RUN apt-get update
RUN apt-get install -y libssl-dev

CMD [ "constellation", "-c", "/etc/constellation.cfg" ]

EXPOSE 53/udp 8080
