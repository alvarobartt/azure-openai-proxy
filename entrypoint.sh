#!/bin/bash

set -m

trap 'kill -TERM $TGI_PID $PROXY_PID 2>/dev/null' TERM INT

ldconfig 2>/dev/null || echo 'unable to refresh ld cache, not a big deal in most cases'

source /usr/src/.venv/bin/activate

text-generation-launcher --port 8080 "$@" &
TGI_PID=$!

openai-azure-proxy &
PROXY_PID=$!

wait -n $TGI_PID $PROXY_PID

kill -TERM $TGI_PID $PROXY_PID 2>/dev/null
wait $TGI_PID $PROXY_PID 2>/dev/null
