FROM rust:slim-bookworm AS builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release 2>/dev/null || true && rm -rf src
COPY src/ src/
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 poppler-utils && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/the-accountant .
COPY migrations/ migrations/
EXPOSE 8080
CMD ["./the-accountant"]
