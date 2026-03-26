# syntax=docker/dockerfile:1
FROM rust:1.94.0 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
# Установить openssl и необходимые библиотеки
RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/links_preview /app/links_preview

# COPY --from=builder /app/target/release/links_preview /app/links_preview
# COPY --from=builder /app/target/release/deps /app/target/release/deps
#COPY Rocket.toml ./Rocket.toml
CMD ["/app/links_preview"]
