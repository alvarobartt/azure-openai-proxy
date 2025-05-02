FROM lukemathwalker/cargo-chef:latest-rust-1.86 AS chef

WORKDIR /app
RUN apt-get update && apt-get install -y protobuf-compiler

FROM chef AS planner

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM ghcr.io/huggingface/text-generation-inference:3.2.3

COPY --from=builder /app/target/release/oaiaz /usr/local/bin/oaiaz
EXPOSE 80 8080
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
