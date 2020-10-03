FROM alpine

RUN apk add ffmpeg

COPY target/x86_64-unknown-linux-musl/release/vidmerger /

CMD ./vidmerger data/
