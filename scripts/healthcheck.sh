#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if [[ -f .env ]]; then
  if grep -Eq '^(ADMIN_PASSWORD_HASH|OPERATOR_PASSWORD_HASH|MAIN_PASSWORD_HASH|CAMPUS_B_PASSWORD_HASH|CAMPUS_C_PASSWORD_HASH)=\$2' .env; then
    echo "bcrypt hashes in .env must be wrapped in single quotes before running healthcheck." >&2
    exit 1
  fi
  set -a
  # shellcheck disable=SC1091
  source .env
  set +a
fi

echo "Docker services:"
docker compose ps

echo
echo "Local web health:"
docker compose exec -T campus-connect-web wget -qO- http://127.0.0.1:3000/api/health || true

echo
echo "Public endpoints:"
for url in \
  "https://${APP_DOMAIN:-app.example.com}/health" \
  "https://${RTC_DOMAIN:-rtc.example.com}" \
  "https://${STREAM_DOMAIN:-stream.example.com}/broadcast/test/llhls.m3u8"; do
  echo "- $url"
  curl -k -I --max-time 10 "$url" | head -n 1 || true
done
