FROM rust:1.81 AS builder
WORKDIR /usr/src/collied
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /usr/src/collied/target/release/collied /usr/local/bin/collied
ENV PORT=3000
EXPOSE $PORT
CMD ["collied", "-p", $PORT]
