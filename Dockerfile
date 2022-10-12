ARG filename=GP1_Team04_Highscore

FROM rust:1.63-buster as builder
WORKDIR /usr/src/$filename
COPY . .
RUN apt-get update -y

RUN cargo build --release

FROM debian:buster-slim
RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/$filename/target/release/$filename /usr/local/bin/$filename
RUN ln -s /usr/local/bin/$filename /usr/local/bin/docker_entrypoint.sh
RUN chmod +x /usr/local/bin/docker_entrypoint.sh
RUN chmod +x /usr/local/bin/$filename

EXPOSE 80
EXPOSE 443

CMD ["GP1_Team04_Highscore"]