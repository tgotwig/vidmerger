FROM rust:latest

# Install ffmpeg
RUN apt-get update && apt-get install -y ffmpeg

# Install go-task
RUN curl -L https://github.com/go-task/task/releases/download/v3.9.0/task_linux_amd64.tar.gz | tar xz && mv task /usr/local/bin
