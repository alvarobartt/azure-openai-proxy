#!/bin/bash

set -m

trap "kill -TERM $UPSTREAM_PID $PROXY_PID 2>/dev/null" TERM INT

PROXY_HOST="${HOST:-0.0.0.0}"
PROXY_PORT="${PORT:-80}"

UPSTREAM_HOST="0.0.0.0"
UPSTREAM_PORT="${UPSTREAM_PORT:-30000}"

# TODO: we should also skip the `-p/--port` argument handling and make sure that's not set to 80,
# and that only one of `PORT`, `UPSTREAM_PORT` or `-p/--port` is set
if [[ -n "$UPSTREAM_PORT" ]]; then
    if [[ "$UPSTREAM_PORT" -eq 80 ]]; then
        echo "ERROR: UPSTREAM_PORT environment variable cannot be set to 80 when running SGLang with Docker, as the port 80 is reserved for the openai-azure-proxy service. Use the port 30000 instead, which is SGLang's default."
        exit 1
    fi
    UPSTREAM_PORT=$UPSTREAM_PORT
fi

export HF_HUB_USER_AGENT_ORIGIN="azure:foundry:gpu-cuda:inference:sglang-native"

python3 -m sglang.launch_server --host "$UPSTREAM_HOST" --port "$UPSTREAM_PORT" "$@" &
UPSTREAM_PID=$!

openai-azure-proxy --host "$PROXY_HOST" --port "$PROXY_PORT" --upstream-host "$UPSTREAM_HOST" --upstream-port "$UPSTREAM_PORT" &
PROXY_PID=$!

wait -n $UPSTREAM_PID $PROXY_PID

kill -TERM $UPSTREAM_PID $PROXY_PID 2>/dev/null
wait $UPSTREAM_PID $PROXY_PID 2>/dev/null
