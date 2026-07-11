# Monitoring Guide

Monitoring is exposed through `monitor.example.com` and restricted by `MONITOR_ALLOWED_CIDR` in `.env`.

## Services

- Grafana: `https://monitor.example.com/grafana/`
- Uptime Kuma: `https://monitor.example.com/uptime/`
- Prometheus: `https://monitor.example.com/prometheus/`

## First Login

Grafana credentials come from:

```text
GRAFANA_ADMIN_USER
GRAFANA_ADMIN_PASSWORD
```

Set a strong password in `.env` before first deploy.

## Suggested Uptime Kuma Checks

Create HTTPS checks for:

- `https://app.example.com/health`
- `https://app.example.com/login`
- `https://rtc.example.com`
- `https://stream.example.com/broadcast/test/llhls.m3u8`
- `https://monitor.example.com/grafana/`

Create TCP/UDP-aware external checks if your monitoring provider supports them:

- RTC TCP media: `rtc.example.com:7881`
- TURN TLS: `turn.example.com:5349`
- Stream relay RTMP: `stream.example.com:1936`
- Stream relay SRT UDP: `stream.example.com:9999`

## During Rehearsal

Watch:

- Docker service health in `docker compose ps`.
- Grafana service health, EC2 CPU/RAM/disk, network throughput, and top container CPU/RAM panels.
- Prometheus target health for `/api/metrics`, RTC server, ingress, and egress.
- Participant status in the operator dashboard.
- Browser network quality indicators under the grid.
- Stream relay logs during SRT ingest.

Useful commands:

```bash
docker compose ps
docker stats --no-stream
docker compose logs -f rtc-server streaming-relay campus-connect-web nginx
./scripts/healthcheck.sh
```

For host-level checks over SSH:

```bash
top
free -h
df -h
uptime
sudo ss -s
```

Install deeper live tools when debugging lag:

```bash
sudo apt-get update
sudo apt-get install -y htop sysstat iftop iotop
htop
sar -u 1 10
sar -n DEV 1 10
sudo iotop -oPa
```

## Scale Decision Rules

Upgrade the EC2 instance if any of these happen during a real rehearsal:

- CPU stays above 75-80% for more than 5 minutes.
- `streaming-relay` is the top CPU container while Program Mode adaptive playback is active.
- RAM usage stays above 80% or swap starts being used.
- Network outbound is close to the instance/network limit or packet loss appears in browser quality indicators.
- Disk is above 80% or I/O wait is high in `htop`/`sar`.

If CPU jumps only after enabling adaptive Program Mode, the relay transcodes are the likely bottleneck. Move from `c7i.2xlarge` to `c7i.4xlarge` first.

## Alert Guidance

At minimum, alert on:

- `campus-connect-web` health failure.
- `rtc-server` restart loop.
- `streaming-relay` restart loop.
- Disk usage above 80%.
- CPU sustained above 80% during rehearsal.
- Program test stream unavailable.

## Retention

Prometheus is configured for 30 days of retention. Keep long-term incident notes outside the instance, such as in an operations doc or ticket tracker.
