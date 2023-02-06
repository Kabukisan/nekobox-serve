FROM rust:bullseye
COPY . /source

# Install dependencies
RUN apt update && \
    apt -y install curl python-is-python3 ffmpeg redis redis-server && \
    curl -L https://yt-dl.org/downloads/latest/youtube-dl -o /usr/local/bin/youtube-dl && \
    chmod a+rx /usr/local/bin/youtube-dl

# Build nekobox-serve
RUN cd source && \
    mkdir /root/nekobox && \
    cargo build --release && \
    cp target/release/nekobox-serve /root/nekobox && \
    rm -r /source

EXPOSE 3000

WORKDIR /root/nekobox
ENTRYPOINT ["sh", "-c", "redis-server --daemonize yes && ./nekobox-serve"]

