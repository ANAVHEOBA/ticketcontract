#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "[test] build gate"
./scripts/build.sh

echo "[test] full integration suite"
cargo test -p ticketing_core --test integration -- --nocapture

echo "[test] focused security + governance + end_to_end"
cargo test -p ticketing_core --test integration security:: -- --nocapture
cargo test -p ticketing_core --test integration governance_hooks:: -- --nocapture
cargo test -p ticketing_core --test integration end_to_end:: -- --nocapture

echo "[test] done"
