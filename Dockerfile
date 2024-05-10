FROM alpine:latest

COPY target/x86_64-unknown-linux-musl/release/eugene-bot /usr/local/bin/eugene-bot

ENTRYPOINT ["eugene-bot"]
