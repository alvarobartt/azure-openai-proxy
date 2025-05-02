#!/bin/bash

ldconfig 2>/dev/null || echo 'unable to refresh ld cache, not a big deal in most cases'

source /usr/src/.venv/bin/activate

trap 'kill -TERM $TGI_PID' TERM
text-generation-launcher --port 8080 "$@" &
TGI_PID=$!

oaiaz &
PROXY_PID=$!

wait $PROXY_PID
kill -TERM $TGI_PID
wait $TGI_PID
