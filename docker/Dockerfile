# Image from a prebuilt glibc Linux binary: CI uses the release asset (.github/workflows/release-linux-*.yml), locally see run-docker in Taskfile.yml
FROM debian:stable-slim
RUN apt-get update && apt-get install -y --no-install-recommends ffmpeg && rm -rf /var/lib/apt/lists/*
COPY vidmerger /vidmerger
ENTRYPOINT ["./vidmerger", "/data"]
