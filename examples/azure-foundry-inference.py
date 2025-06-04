from azure.ai.inference import ChatCompletionsClient
from azure.ai.inference.models import SystemMessage, UserMessage

if __name__ == "__main__":
    client = ChatCompletionsClient(endpoint="http://localhost", credential="")  # type: ignore

    response = client.complete(
        messages=[
            SystemMessage("You are a helpful assistant."),
            UserMessage("How many feet are in a mile?"),
        ],
        max_tokens=128,
    )

    print(response.choices[0].message.content)
    print(f"\nToken usage: {response.usage}")
