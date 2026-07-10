#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

./scripts/render-configs.sh
docker compose up -d --build
docker compose restart nginx rtc-server rtc-ingress rtc-egress campus-connect-web streaming-relay coturn
docker compose ps
