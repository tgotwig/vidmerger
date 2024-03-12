FROM alpine

RUN apk add ffmpeg

COPY target/x86_64-unknown-linux-musl/release/vidmerger /

# Set vidmerger as the default executable
ENTRYPOINT ["./vidmerger"]

# Set the default arguments to vidmerger
CMD ["/data"]
