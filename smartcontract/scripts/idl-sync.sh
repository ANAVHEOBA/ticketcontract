#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "[idl] build"
IDL_LOG="$(mktemp)"
anchor idl build >"$IDL_LOG"
echo "[idl] build complete"

IDL_SRC="$ROOT_DIR/target/idl/ticketing_core.json"
IDL_DST_DIR="$ROOT_DIR/app/idl"
IDL_DST="$IDL_DST_DIR/ticketing_core.json"

mkdir -p "$IDL_DST_DIR"
cp "$IDL_SRC" "$IDL_DST"
echo "[idl] synced to $IDL_DST"
