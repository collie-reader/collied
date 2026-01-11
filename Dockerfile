FROM rust:1.85 AS builder
WORKDIR /usr/src/collied

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

COPY . .
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -r -s /bin/false collied \
    && mkdir -p /etc/collied \
    && chown collied:collied /etc/collied
USER collied

COPY --from=builder /usr/src/collied/target/release/collied /usr/local/bin/collied

ENV PORT=3000
EXPOSE $PORT
CMD ["sh", "-c", "collied serve --port $PORT"]
