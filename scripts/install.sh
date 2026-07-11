#!/usr/bin/env bash
set -euo pipefail

if [[ "$(id -u)" -ne 0 ]]; then
  echo "Run as root on Ubuntu 24.04: sudo ./scripts/install.sh" >&2
  exit 1
fi

apt-get update
apt-get install -y ca-certificates curl gnupg lsb-release ufw jq gettext-base tar dnsutils

install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
chmod a+r /etc/apt/keyrings/docker.gpg

cat >/etc/apt/sources.list.d/docker.list <<EOF
deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu $(. /etc/os-release && echo "$VERSION_CODENAME") stable
EOF

apt-get update
apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
systemctl enable --now docker

ufw allow 22/tcp
ufw allow 80/tcp
ufw allow 443/tcp
ufw allow 1935/tcp
ufw allow 1936/tcp
ufw allow from 172.16.0.0/12 to any port 7880 proto tcp
ufw allow from 172.16.0.0/12 to any port 6789 proto tcp
ufw allow 3478/tcp
ufw allow 3478/udp
ufw allow 5349/tcp
ufw allow 7881/tcp
ufw allow 7885/udp
ufw allow 9999/udp
ufw allow 10000/tcp
ufw allow 10000:10003/udp
ufw allow 50000:60000/udp
ufw --force enable

if [[ ! -f .env ]]; then
  cp .env.example .env
  chmod 600 .env
  echo "Created .env from .env.example. Fill it in before deploy."
fi

echo "Docker, Compose, firewall rules, and envsubst are installed."
