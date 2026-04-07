#!/usr/bin/env bash

set -euo pipefail

echo "[1/4] frontend build"
npm run check:frontend

echo "[2/4] backend check"
npm run check:backend

echo "[3/4] backend tests"
npm run test:backend

echo "[4/4] tauri debug bundle (.app)"
npm run tauri build -- --debug

echo "verify complete"
