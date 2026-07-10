#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
backup_dir="$ROOT_DIR/backups/campus-connect-$timestamp"
mkdir -p "$backup_dir"
chmod 700 "$backup_dir"

cp docker-compose.yml "$backup_dir/"
cp .env "$backup_dir/env.secure"
cp -R rtc.yaml config docs scripts "$backup_dir/"

volumes=(
  campus-connect_redis-data
  campus-connect_app-data
  campus-connect_letsencrypt
  campus-connect_prometheus-data
  campus-connect_grafana-data
  campus-connect_uptime-kuma-data
)

for volume in "${volumes[@]}"; do
  if docker volume inspect "$volume" >/dev/null 2>&1; then
    docker run --rm -v "$volume:/volume:ro" -v "$backup_dir:/backup" alpine:3.21 \
      tar czf "/backup/$volume.tar.gz" -C /volume .
  fi
done

tar czf "$backup_dir.tgz" -C "$ROOT_DIR/backups" "$(basename "$backup_dir")"
rm -rf "$backup_dir"
chmod 600 "$backup_dir.tgz"

echo "Backup written to $backup_dir.tgz"
