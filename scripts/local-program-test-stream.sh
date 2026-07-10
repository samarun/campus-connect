#!/usr/bin/env bash
set -euo pipefail

if ! command -v ffmpeg >/dev/null 2>&1; then
  echo "ffmpeg is required. Install it first, for example: brew install ffmpeg" >&2
  exit 1
fi

STREAM_URL="${1:-srt://127.0.0.1:9999?streamid=default/broadcast/program}"

cat <<EOF
Publishing local program test stream.

Target:
  $STREAM_URL

Open:
  http://localhost:3000/player/program

Press Ctrl+C to stop the test stream.
EOF

ffmpeg -hide_banner -re \
  -f lavfi -i testsrc2=size=1920x1080:rate=30 \
  -f lavfi -i sine=frequency=1000:sample_rate=48000 \
  -c:v libx264 -preset veryfast -tune zerolatency -profile:v high -level:v 4.2 \
  -pix_fmt yuv420p -b:v 12M -minrate 12M -maxrate 12M -bufsize 24M \
  -x264-params "nal-hrd=cbr:force-cfr=1" -g 60 -keyint_min 60 -sc_threshold 0 \
  -c:a aac -ar 48000 -b:a 192k \
  -f mpegts "$STREAM_URL"
