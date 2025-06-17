#!/bin/bash
# install vitest and related packages if missing
set -e
npm install --save-dev vitest jsdom happy-dom @types/node --no-audit --prefer-offline

