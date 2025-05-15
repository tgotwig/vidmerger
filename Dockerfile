FROM rust:1.87.0 AS builder
ARG TARGETPLATFORM
RUN apt-get update && apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl
WORKDIR /app
COPY . .
RUN case "$TARGETPLATFORM" in \
        "linux/amd64")   TARGET="x86_64-unknown-linux-musl" ;; \
        "linux/arm64")   TARGET="aarch64-unknown-linux-musl" ;; \
    esac && \
    cargo build --release --target $TARGET && \
    cp target/$TARGET/release/vidmerger /vidmerger

FROM alpine
RUN apk add ffmpeg
COPY --from=builder /vidmerger /vidmerger
ENTRYPOINT ["./vidmerger", "/data"]
