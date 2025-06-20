#!/usr/bin/env bash
set -e
PORT=6006
mkdir -p storybook-snapshots
BROWSER=none npx storybook dev --no-open -p $PORT --quiet &
SB_PID=$!
npx wait-on http://localhost:$PORT
npx test-storybook --browsers chromium --url http://localhost:$PORT || true
kill $SB_PID
