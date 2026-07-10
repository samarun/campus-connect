#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

./scripts/preflight.sh
./scripts/render-configs.sh

if [[ ! -f campus-connect-web/package-lock.json ]]; then
  echo "Creating package-lock.json for the web app."
  npm --prefix campus-connect-web install --package-lock-only
fi

docker compose pull redis rtc-server rtc-ingress rtc-egress coturn streaming-relay nginx certbot prometheus grafana uptime-kuma || true
docker compose up -d --build

echo "Campus Connect deploy started."
docker compose ps
