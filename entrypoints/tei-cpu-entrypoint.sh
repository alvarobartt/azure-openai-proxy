#!/bin/bash

set -m

trap "kill -TERM $UPSTREAM_PID $PROXY_PID 2>/dev/null" TERM INT

ldconfig 2>/dev/null || echo "Unable to refresh ld cache"

PROXY_HOST="${HOST:-0.0.0.0}"
PROXY_PORT="${PORT:-80}"
# We need to unset the HOST and the PORT because otherwise it's picked up by TEI
unset HOST PORT

UPSTREAM_HOST="0.0.0.0"
UPSTREAM_PORT="${UPSTREAM_PORT:-8080}"

# TODO: we should also skip the `-p/--port` argument handling and make sure that's not set to 80,
# and that only one of `PORT`, `UPSTREAM_PORT` or `-p/--port` is set
if [[ -n "$UPSTREAM_PORT" ]]; then
    if [[ "$UPSTREAM_PORT" -eq 80 ]]; then
        echo "ERROR: UPSTREAM_PORT environment variable cannot be set to 80 when running text-embeddings-inference with Docker, as the port 80 is reserved for the azure-openai-proxy service. Use another port as e.g. 8080, since 80 is the default port for TEI, but that leads to conflicts in this scenario."
        exit 1
    fi
    UPSTREAM_PORT=$UPSTREAM_PORT
fi

export HF_HUB_USER_AGENT_ORIGIN="azure:foundry:cpu:inference:tei"

text-embeddings-router --hostname "$UPSTREAM_HOST" --port "$UPSTREAM_PORT" "$@" &
UPSTREAM_PID=$!

azure-openai-proxy --host "$PROXY_HOST" --port "$PROXY_PORT" --upstream-host "$UPSTREAM_HOST" --upstream-port "$UPSTREAM_PORT" --upstream-type embeddings &
PROXY_PID=$!

wait -n $UPSTREAM_PID $PROXY_PID

kill -TERM $UPSTREAM_PID $PROXY_PID 2>/dev/null
wait $UPSTREAM_PID $PROXY_PID 2>/dev/null
