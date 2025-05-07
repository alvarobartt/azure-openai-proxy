FROM lukemathwalker/cargo-chef:latest-rust-1.86 AS chef

WORKDIR /app
RUN apt-get update && apt-get install -y protobuf-compiler

FROM chef AS planner

COPY src/ src/
COPY Cargo.toml Cargo.lock ./

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY src/ src/
COPY Cargo.toml Cargo.lock ./

RUN cargo build --release

FROM ghcr.io/huggingface/text-generation-inference:3.2.3 AS tgi

COPY --from=builder /app/target/release/openai-azure-proxy /usr/local/bin/openai-azure-proxy

COPY tgi-entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]

FROM lmsysorg/sglang:v0.4.6.post2-cu124 AS sglang

COPY --from=builder /app/target/release/openai-azure-proxy /usr/local/bin/openai-azure-proxy

COPY sglang-entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]

FROM vllm/vllm-openai:v0.8.5.post1 AS vllm

COPY --from=builder /app/target/release/openai-azure-proxy /usr/local/bin/openai-azure-proxy

COPY vllm-entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
