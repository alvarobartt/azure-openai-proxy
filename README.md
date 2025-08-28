# Azure AI Model Inference Proxy for OpenAI APIs

A Rust-based proxy that exposes [Azure AI Model Inference API](https://learn.microsoft.com/en-us/azure/ai-foundry/model-inference/overview) routes and forwards requests to OpenAI-compatible backends.

> [!WARNING]
> This project is an independent open source implementation and is not affiliated with or endorsed by Microsoft or Azure.

## Features

- Backend-agnostic, works with any OpenAI-compatible service
- Includes presets / targets for Text Generation Inference, vLLM, SGLang, and Text Embeddings Inference
- Matches Azure AI Model Inference API routes seamlessly
- Supports chat-completions and embedding models

## Usage

```console
git clone git@github.com:alvarobartt/azure-openai-proxy.git
cd azure-openai-proxy/
```

Once cloned, build the `Dockerfile` for [Text Generation Inference (TGI)](https://github.com/huggingface/text-generation-inference):

```bash
docker build --platform=linux/amd64 \
    -t huggingface-azure-tgi:latest \
    -f Dockerfile \
    --target tgi \
    .
```

When running the `azure-openai-proxy` via Docker, the initialization of both the inference engine
and the proxy will be automatically handled, as well as the graceful shutdown with signal handling.

Then you need to make sure that you have access to at least a single NVIDIA GPU and run the container as:

```bash
docker run \
    --gpus all \
    --shm-size 1g \
    -p 80:80 \
    -e UPSTREAM_PORT=8080 \
    -v ~/.cache/huggingface:/data \
    huggingface-azure-tgi:latest \
    --model-id TinyLlama/TinyLlama-1.1B-Chat-v1.0
```

Note that also the [SGLang](https://github.com/sgl-project/sglang) and [vLLM](https://github.com/vllm-project/vllm)
images can be built and ran.

<details>
    <summary>Or SGLang as follows:</summary>

```bash
docker build --platform=linux/amd64 \
    -t huggingface-azure-sglang:latest \
    -f Dockerfile \
    --target sglang \
    .

docker run \
    --ipc host \
    --gpus all \
    --shm-size 1g \
    -p 80:80 \
    -e UPSTREAM_PORT=30000 \
    -v ~/.cache/huggingface:/root/.cache/huggingface \
    huggingface-azure-sglang:latest \
    --model-path TinyLlama/TinyLlama-1.1B-Chat-v1.0
```
</details>

<details>
    <summary>Or vLLM as follows:</summary>

```bash
docker build --platform=linux/amd64 \
    -t huggingface-azure-vllm:latest \
    -f Dockerfile \
    --target vllm \
    .

docker run \
    --ipc host \
    --gpus all \
    --runtime nvidia \
    --shm-size 1g \
    -p 80:80 \
    -e UPSTREAM_PORT=8000 \
    -v ~/.cache/huggingface:/root/.cache/huggingface \
    huggingface-azure-vllm:latest \
    --model TinyLlama/TinyLlama-1.1B-Chat-v1.0
```
</details>

Finally, you should be able to send requests with the Azure AI Model Inference API specification, and
the proxy will forward those to the underlying inference engine.

```bash
curl http://localhost/chat/completions?api-version=2025-04-01 \
    -H "Content-Type: application/json" \
    -d '{"messages":[{"role":"system","content":"You are a helpful assistant."},{"role":"user","content":"What is Deep Learning?"}]}'
```

Alternatively, you can also use the `azure-ai-inference` Python SDK as:

```python
from azure.ai.inference import ChatCompletionsClient
from azure.ai.inference.models import SystemMessage, UserMessage

client = ChatCompletionsClient(endpoint="http://localhost", credential="")

response = client.complete(
    messages=[
        SystemMessage(content="You are a helpful assistant."),
        UserMessage(content="What is Deep Learning?"),
    ],
)
```

Find more examples at [`examples/`](examples/).

> [!NOTE]
> If deployed on Azure, you should replace the host i.e., "http://localhost", with the
> Azure AI / ML Endpoint URL, as well as setting the required endpoint credentials.

## Development

Install the Rust binary as it follows:

```bash
cargo install --path .
```

Then you can easily run it and connect it to a running OpenAI-compatible server that
exposes the `/v1/chat/completions` endpoint:

```bash
azure-openai-proxy --upstream-host https://api.openai.com --upstream-port 80
```

For more information check the `--help`:

```console
$ azure-openai-proxy --help
A proxy for Azure AI Model Inference API routes to OpenAI-compatible APIs

Usage: azure-openai-proxy [OPTIONS]

Options:
  -h, --host <HOST>                    [env: HOST=] [default: 0.0.0.0]
  -p, --port <PORT>                    [env: PORT=] [default: 80]
  -u, --upstream-host <UPSTREAM_HOST>  [env: UPSTREAM_HOST=] [default: 0.0.0.0]
  -u, --upstream-port <UPSTREAM_PORT>  [env: UPSTREAM_PORT=] [default: 8080]
  -h, --help                           Print help
  -V, --version                        Print version
```

## License

This project is licensed under either of the following licenses, at your option:

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
