# `azure-openai-proxy`

`azure-openai-proxy` is a proxy for [Azure AI Model Inference API](https://learn.microsoft.com/en-us/azure/ai-foundry/model-inference/overview)
routes to OpenAI-compatible APIs (`/v1/chat/completions`, `/v1/embeddings`), written in Rust.

[More information about the Azure AI Model Inference API](https://learn.microsoft.com/en-us/rest/api/aifoundry/modelinference/?view=rest-aifoundry-model-inference-2025-04-01).

## Usage

> [!NOTE]
> The `azure-openai-proxy` is backend agnostic, meaning that it will work seamlessly
> if the proxy is deployed on another instance that's not the instance running the inference
> engine; so there's no such thing as requirements, besides having an OpenAI-compatible
> service running somewhere that's accessible.

### Docker (Recommended)

When running the `azure-openai-proxy` via Docker, the `tgi-entrypoint.sh` will handle
both the initialization of the inference engine (in this case being [Text Generation Inference (TGI)](https://github.com/huggingface/text-generation-inference)),
as well as the initialization of the proxy; and the graceful shutdown of those via
proper signal handling.

```bash
git clone git@github.com:alvarobartt/azure-openai-proxy.git
cd azure-openai-proxy/
docker build --platform=linux/amd64 -t huggingface-azure-tgi:latest -f Dockerfile --target tgi .
```

Note that also the [SGLang](https://github.com/sgl-project/sglang) and [vLLM](https://github.com/vllm-project/vllm)
backends have a target defined within the `Dockerfile`, as well as their respective entrypoint, to be built as:

```bash
docker build --platform=linux/amd64 -t huggingface-azure-sglang:latest -f Dockerfile --target sglang .
docker build --platform=linux/amd64 -t huggingface-azure-vllm:latest -f Dockerfile --target vllm .
```

> [!NOTE]
> Ideally the containers should be published to a publicly available Docker Registry,
> either the Docker Hub (docker.io) or the Azure Container Registry (mcr.microsoft.com).

Then you need to make sure that you have access to at least a single NVIDIA GPU,
as it's required by TGI to run the inference server for the model aforementioned; and
then run the container as follows:

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

Or SGLang as follows:

```bash
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

Or vLLM as follows:

```bash
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

Finally, you should be able to send requests in an Azure AI Model Inference compatible manner to the
deployed proxy, and it should upstream those to the inference engine (being TGI in this
case), making sure that the I/O schemas and paths are compliant with the Azure AI Model Inference API.

> [!WARNING]
> There may be a subtle overhead in the proxy, as in all the proxy services,
> due to the network. The delay shouldn't be noticeable, but just take that into consideration
> when benchmarking or testing the deployed service via the proxy, as the numbers won't be
> the same as when directly calling to whatever engine is behind.

```bash
curl http://localhost/chat/completions?api-version=2025-04-01 \
    -H "Content-Type: application/json" \
    -d '{"messages":[{"role":"system","content":"You are a helpful assistant."},{"role":"user","content":"What is Deep Learning?"}]}'
```

Alternatively, you can also use `azure-ai-inference` as follows:

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
> If deployed within Azure infrastructure, you should replace the "http://localhost"
> with the actual endpoint URI and the required credentials.

### Locally

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
