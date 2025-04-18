FROM rust AS build
COPY --from=mwader/static-ffmpeg:7.1.1 /ffmpeg /usr/local/bin/
COPY --from=mwader/static-ffmpeg:7.1.1 /ffprobe /usr/local/bin/
WORKDIR /temp/task
RUN wget https://github.com/go-task/task/releases/download/v3.42.1/task_linux_386.tar.gz && \
    tar xvf task_linux_386.tar.gz && \
    chmod +x task && \
    mv task /usr/local/bin/task
WORKDIR /build
COPY . /build
RUN task

FROM debian:stable-slim
COPY --from=mwader/static-ffmpeg:7.1.1 /ffmpeg /usr/local/bin/
COPY --from=mwader/static-ffmpeg:7.1.1 /ffprobe /usr/local/bin/
WORKDIR /app
COPY --from=build /build/target/debug/vidmerger /usr/local/bin/
ENTRYPOINT ["vidmerger", "/data"]
