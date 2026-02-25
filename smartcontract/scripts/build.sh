#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "[build] anchor build"
BUILD_LOG="$(mktemp)"
if ! anchor build 2>&1 | tee "$BUILD_LOG"; then
  echo "[build] failed"
  exit 1
fi

if rg -q "Stack offset .* exceeded max offset" "$BUILD_LOG"; then
  echo "[build] detected SBF stack overflow warning; treat as failure"
  exit 1
fi

echo "[build] success"
