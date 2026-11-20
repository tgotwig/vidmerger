FROM ubuntu:24.04
RUN apt-get update && \
    apt-get install -y --no-install-recommends ffmpeg && \
    rm -rf /var/lib/apt/lists/*
COPY vidmerger /vidmerger
ENTRYPOINT ["./vidmerger", "/data"]
