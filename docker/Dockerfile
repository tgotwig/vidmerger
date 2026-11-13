FROM rust:1.100.0 AS builder
ARG TARGETPLATFORM
RUN apt-get update && apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl
WORKDIR /app
COPY Cargo.toml ./
# Cargo fetch needs a target; keep this layer independent of application source changes.
RUN mkdir src && printf 'fn main() {}\n' > src/main.rs && cargo fetch && rm -r src
COPY . .
RUN case "$TARGETPLATFORM" in \
        "linux/amd64")   TARGET="x86_64-unknown-linux-musl" ;; \
        "linux/arm64")   TARGET="aarch64-unknown-linux-musl" ;; \
    esac && \
    cargo build --release --locked --target "$TARGET" && \
    cp "target/$TARGET/release/vidmerger" /vidmerger

FROM alpine
RUN apk add ffmpeg
COPY --from=builder /vidmerger /vidmerger
ENTRYPOINT ["./vidmerger", "/data"]
