FROM rust:latest as builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev libsqlite3-dev
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /usr/src/app/target/release/backend /usr/local/bin/backend
COPY public /app/public

EXPOSE 8080
CMD ["backend"]
