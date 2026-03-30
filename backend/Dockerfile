FROM rust:latest as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /usr/src/app/target/release/backend /usr/local/bin/backend
COPY public /app/public

EXPOSE 8080
CMD ["backend"]
