# Build Stage
FROM rust:latest AS build_vidmerger
RUN apt-get update && apt-get install -y musl-tools \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY . .

## manual build without task
RUN rustup target add x86_64-unknown-linux-musl && \
    cargo build --release --target=x86_64-unknown-linux-musl
ENTRYPOINT [ "bash" ]


# Final Stage
FROM alpine

## install current apk ffmpeg
RUN apk add --no-cache ffmpeg

## copy recent built vidmerger
COPY --from=build_vidmerger /build/target/x86_64-unknown-linux-musl/release/vidmerger /usr/local/bin/
ENTRYPOINT ["vidmerger", "/data"]
