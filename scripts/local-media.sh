#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

case "${1:-up}" in
  up)
    mkdir -p runtime/streaming
    PUBLIC_IP="${LOCAL_STREAM_PUBLIC_IP:-127.0.0.1}" envsubst '$PUBLIC_IP' < config/streaming/Server.xml > runtime/streaming/Server.xml
    docker compose -f docker-compose.local-media.yml up -d
    docker compose -f docker-compose.local-media.yml ps
    cat <<EOF

Local media stack is starting.

RTC:
  ws://localhost:7880
  advertised media IP: ${LOCAL_RTC_NODE_IP:-127.0.0.1}
  API key: local_dev_key
  API secret: local_dev_secret

Streaming relay:
  SRT ingest: srt://localhost:9999?streamid=default/broadcast/program
  RTMP ingest: rtmp://localhost:1936/broadcast/program
  LL-HLS playback: http://localhost:3333/broadcast/program/llhls.m3u8

Use ./scripts/local-web.sh in another terminal for the web app.
EOF
    ;;
  down)
    docker compose -f docker-compose.local-media.yml down
    ;;
  logs)
    docker compose -f docker-compose.local-media.yml logs -f "${@:2}"
    ;;
  ps)
    docker compose -f docker-compose.local-media.yml ps
    ;;
  *)
    echo "Usage: $0 [up|down|logs|ps]" >&2
    exit 1
    ;;
esac
