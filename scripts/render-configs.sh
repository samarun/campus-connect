#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if [[ ! -f .env ]]; then
  echo "Missing .env. Copy .env.example to .env and fill in values." >&2
  exit 1
fi

if grep -Eq '^(ADMIN_PASSWORD_HASH|OPERATOR_PASSWORD_HASH|MAIN_PASSWORD_HASH|CAMPUS_B_PASSWORD_HASH|CAMPUS_C_PASSWORD_HASH)=\$2' .env; then
  echo "bcrypt hashes in .env must be wrapped in single quotes, for example ADMIN_PASSWORD_HASH='\$2b\$12\$...'." >&2
  exit 1
fi

set -a
# shellcheck disable=SC1091
source .env
set +a

required=(
  PUBLIC_IP
  APP_DOMAIN
  RTC_DOMAIN
  TURN_DOMAIN
  STREAM_DOMAIN
  MONITOR_DOMAIN
  MONITOR_ALLOWED_CIDR
  SESSION_SECRET
  RTC_API_KEY
  RTC_API_SECRET
  TURN_STATIC_AUTH_SECRET
)

for name in "${required[@]}"; do
  value="${!name:-}"
  if [[ -z "$value" || "$value" == CHANGE_ME* ]]; then
    echo "Environment variable $name is missing or still uses CHANGE_ME." >&2
    exit 1
  fi
done

if ! command -v envsubst >/dev/null 2>&1; then
  echo "envsubst is required. Run ./scripts/install.sh first or install gettext-base." >&2
  exit 1
fi

mkdir -p runtime/rtc runtime/streaming
envsubst < rtc.yaml > runtime/rtc/server.yaml
envsubst < config/rtc/ingress.yaml > runtime/rtc/ingress.yaml
envsubst < config/rtc/egress.yaml > runtime/rtc/egress.yaml
envsubst < config/streaming/Server.xml > runtime/streaming/Server.xml

chmod 600 runtime/rtc/server.yaml runtime/rtc/ingress.yaml runtime/rtc/egress.yaml runtime/streaming/Server.xml
echo "Rendered runtime media configs."
