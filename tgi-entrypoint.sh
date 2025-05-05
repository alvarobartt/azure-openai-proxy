#!/bin/bash

set -m

trap 'kill -TERM $UPSTREAM_PID $PROXY_PID 2>/dev/null' TERM INT

ldconfig 2>/dev/null || echo 'unable to refresh ld cache, not a big deal in most cases'

source /usr/src/.venv/bin/activate

UPSTREAM_HOST="host.docker.internal"
export UPSTREAM_HOST="$UPSTREAM_HOST"

UPSTREAM_PORT=8080
export UPSTREAM_PORT="$UPSTREAM_PORT"
unset PORT

# TODO: we should also skip the `-p/--port` argument handling and make sure that's not set to 80,
# and that only one of `PORT`, `UPSTREAM_PORT` or `-p/--port` is set
text-generation-launcher --port "${UPSTREAM_PORT}" "$@" &
UPSTREAM_PID=$!

openai-azure-proxy &
PROXY_PID=$!

wait -n $UPSTREAM_PID $PROXY_PID

kill -TERM $UPSTREAM_PID $PROXY_PID 2>/dev/null
wait $UPSTREAM_PID $PROXY_PID 2>/dev/null
