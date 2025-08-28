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

FROM ghcr.io/huggingface/text-generation-inference:3.3.4 AS tgi

COPY --from=builder /app/target/release/azure-openai-proxy /usr/local/bin/azure-openai-proxy

COPY entrypoints/tgi-entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]

FROM lmsysorg/sglang:v0.5.1.post3-cu126 AS sglang

COPY --from=builder /app/target/release/azure-openai-proxy /usr/local/bin/azure-openai-proxy

COPY entrypoints/sglang-entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]

FROM vllm/vllm-openai:v0.10.1 AS vllm

COPY --from=builder /app/target/release/azure-openai-proxy /usr/local/bin/azure-openai-proxy

COPY entrypoints/vllm-entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]

FROM ghcr.io/huggingface/text-embeddings-inference:cpu-1.8.0 AS tei-cpu

COPY --from=builder /app/target/release/azure-openai-proxy /usr/local/bin/azure-openai-proxy

COPY entrypoints/tei-cpu-entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]

FROM ghcr.io/huggingface/text-embeddings-inference:cuda-1.8.0 AS tei-gpu

COPY --from=builder /app/target/release/azure-openai-proxy /usr/local/bin/azure-openai-proxy

COPY entrypoints/tei-gpu-entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
