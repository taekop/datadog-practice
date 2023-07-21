FROM rust:1.71.0-slim-bullseye

WORKDIR /app

RUN apt update && apt install -y pkg-config libssl-dev

COPY . .

RUN cargo build

ENTRYPOINT ["/app/target/debug/backend"]
