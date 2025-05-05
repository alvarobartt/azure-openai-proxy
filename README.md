# `openai-azure-proxy`

`openai-azure-proxy` is a proxy for OpenAI-compatible routes (`/v1/chat/completions`, `/v1/embeddings`,
etc.) to [Azure AI Model Inference](https://learn.microsoft.com/en-us/azure/ai-foundry/model-inference/overview).

[More information about the Azure AI Model Inference schemas](https://learn.microsoft.com/en-us/rest/api/aifoundry/modelinference/?view=rest-aifoundry-model-inference-2025-04-01).

## Usage

> [!NOTE]
> The `openai-azure-proxy` is backend agnostic, meaning that it will work seamlessly
> if the proxy is deployed on another instance that's not the instance running the inference
> engine; so there's no such thing as requirements, besides having an OpenAI-compatible
> service running somewhere that's accessible.

> [!TIP]
> The capabilities / resources of the instance will depend on which model from the 
> Hugging Face Hub you want to run, TL;DR the bigger the model in number of parameters
> (by the order of billion of parameters) the larger the total VRAM of the instance.
>
> The examples below, runs the Azure AI proxy for [`TinyLlama/TinyLlama-1.1B-Chat-v1.0`](https://huggingface.co/TinyLlama/TinyLlama-1.1B-Chat-v1.0)
> which requires ~2.42GiB of VRAM including the CUDA overhead and the KV Cache, with
> the default context size of 2048. Note that the larger the context size i.e. maximum I/O
> supported tokens, the larger the KV Cache will be and the more VRAM will be consumed.

### Docker (Recommended)

When running the `openai-azure-proxy` via Docker, the `tgi-entrypoint.sh` will handle
both the initialization of the inference engine (in this case being [Text Generation Inference (TGI)](https://github.com/huggingface/text-generation-inference)),
as well as the initialization of the proxy; and the graceful shutdown of those via
proper signal handling.

```bash
git clone git@github.com:alvarobartt/openai-azure-proxy.git
cd openai-azure-proxy/
docker build --platform=linux/amd64 -t text-generation-inference-azure:latest -f Dockerfile .
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
    -p 80:80 \
    -p 8080:8080 \
    -e UPSTREAM_PORT=8080 \
    text-generation-inference-azure:latest \
    --model-id TinyLlama/TinyLlama-1.1B-Chat-v1.0
```

Finally, you should be able to send requests in an Azure AI compatible manner to the
deployed proxy, and it should upstream those to the inference engine (being TGI in this
case), making sure that the I/O schemas and paths are compliant with Azure AI.

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
UPSTREAM_HOST="https://api.openai.com" UPSTREAM_PORT="80" openai-azure-proxy 
```

## License

This project is licensed under either of the following licenses, at your option:

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
