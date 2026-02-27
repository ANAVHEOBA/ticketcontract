#!/usr/bin/env bash
set -euo pipefail

API_BASE="${API_BASE:-http://localhost:8080/api/v1}"

echo "Running backend smoke checks against $API_BASE"

curl -fsS "$API_BASE/health" >/dev/null
curl -fsS "$API_BASE/docs/openapi.yaml" >/dev/null
curl -fsS "$API_BASE/docs/postman_collection.json" >/dev/null

echo "Smoke checks passed (health + docs endpoints)."
