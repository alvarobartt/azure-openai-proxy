#!/bin/bash

set -m

trap "kill -TERM $UPSTREAM_PID $PROXY_PID 2>/dev/null" TERM INT

ldconfig 2>/dev/null || echo "Unable to refresh ld cache"

source /usr/src/.venv/bin/activate

export UPSTREAM_HOST="http://localhost"

# TODO: we should also skip the `-p/--port` argument handling and make sure that's not set to 80,
# and that only one of `PORT`, `UPSTREAM_PORT` or `-p/--port` is set
UPSTREAM_PORT="${UPSTREAM_PORT:-8080}"

if [[ -n "$UPSTREAM_PORT" ]]; then
    if [[ "$UPSTREAM_PORT" -eq 80 ]]; then
        echo "ERROR: UPSTREAM_PORT environment variable cannot be set to 80 when running text-generation-inference with Docker, as the port 80 is reserved for the openai-azure-proxy service."
        exit 1
    fi
    UPSTREAM_PORT=$UPSTREAM_PORT
fi

export UPSTREAM_PORT="$UPSTREAM_PORT"
unset PORT

export HF_HUB_USER_AGENT_ORIGIN="azure:foundry:gpu-cuda:inference:tgi-native" 

text-generation-launcher --port "${UPSTREAM_PORT}" "$@" &
UPSTREAM_PID=$!

openai-azure-proxy --host 0.0.0.0 --port 80 --upstream-host "${UPSTREAM_HOST}" --upstream-port "${UPSTREAM_PORT}" &
PROXY_PID=$!

wait -n $UPSTREAM_PID $PROXY_PID

kill -TERM $UPSTREAM_PID $PROXY_PID 2>/dev/null
wait $UPSTREAM_PID $PROXY_PID 2>/dev/null
