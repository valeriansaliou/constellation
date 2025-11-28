FROM rust:latest AS build

ARG PREBUILT_TAG
ARG TARGETPLATFORM

ENV PREBUILT_TAG=$PREBUILT_TAG

WORKDIR /app
COPY . /app

RUN case ${TARGETPLATFORM} in \
    "linux/amd64")  echo "x86_64" > .arch && echo "x86_64-unknown-linux-musl" > .toolchain ;; \
    "linux/arm64")  echo "aarch64" > .arch && echo "aarch64-unknown-linux-musl" > .toolchain ;; \
    *)              echo "Unsupported platform: $TARGETPLATFORM" && exit 1 ;; \
    esac

# Run full build?
RUN if [ -z "$PREBUILT_TAG" ]; then \
    apt-get update && \
        apt-get install -y musl-tools && \
        rustup target add $(cat .toolchain) \
    ; fi
RUN if [ -z "$PREBUILT_TAG" ]; then \
    cargo build --release --target $(cat .toolchain) && \
        mkdir -p ./constellation/ && \
        mv ./target/$(cat .toolchain)/release/constellation ./constellation/ \
    ; fi

# Pull pre-built binary?
RUN if [ ! -z "$PREBUILT_TAG" ]; then \
    wget https://github.com/valeriansaliou/constellation/releases/download/$PREBUILT_TAG/$PREBUILT_TAG-$(cat .arch).tar.gz && \
        tar -xzf $PREBUILT_TAG-$(cat .arch).tar.gz \
    ; fi

FROM scratch

WORKDIR /usr/src/constellation

COPY --from=build /app/constellation/constellation /usr/local/bin/constellation

CMD [ "constellation", "-c", "/etc/constellation.cfg" ]

EXPOSE 53/udp 8080
