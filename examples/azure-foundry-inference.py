# /// script
# dependencies = [
#   "azure-ai-inference",
#   "azure-identity",
# ]
# ///

from azure.ai.inference import ChatCompletionsClient
from azure.ai.inference.models import SystemMessage, UserMessage

if __name__ == "__main__":
    client = ChatCompletionsClient(endpoint="http://localhost", credential="")  # type: ignore

    response = client.complete(
        model="TinyLlama/TinyLlama-1.1B-Chat-v1.0",
        messages=[
            SystemMessage("You are a helpful assistant."),
            UserMessage("How many feet are in a mile?"),
        ],
        max_tokens=256,
    )

    print(response.choices[0].message.content)
    print(f"\nToken usage: {response.usage}")
