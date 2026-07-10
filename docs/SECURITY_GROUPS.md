# Security Groups

Use the narrowest source CIDRs possible. `0.0.0.0/0` is acceptable for public viewer paths and browser RTC media only when required by site networks.

## Inbound

| Port | Protocol | Source | Purpose |
| --- | --- | --- | --- |
| 22 | TCP | Admin/VPN IP only | SSH |
| 80 | TCP | 0.0.0.0/0 | ACME HTTP challenge and redirect |
| 443 | TCP | 0.0.0.0/0 | App, RTC signaling, stream playback, monitoring vhost |
| 1935 | TCP | Encoder IPs | RTMP ingest for RTC ingress |
| 1936 | TCP | Encoder IPs | RTMP ingest for stream relay |
| 3478 | UDP/TCP | 0.0.0.0/0 | coturn TURN/STUN |
| 5349 | TCP | 0.0.0.0/0 | coturn TURN/TLS |
| 7881 | TCP | 0.0.0.0/0 | RTC media over TCP |
| 7885 | UDP | Encoder IPs | WHIP ingest |
| 9999 | UDP | Encoder IPs | SRT ingest for stream relay |
| 10000 | TCP | 0.0.0.0/0 | Stream relay WebRTC TCP ICE if enabled |
| 10000-10003 | UDP | 0.0.0.0/0 | Stream relay WebRTC ICE if enabled |
| 50000-60000 | UDP | 0.0.0.0/0 | RTC media |

## Outbound

Allow all outbound, or at minimum:

- TCP 80/443 for package installs, TLS issuance, image pulls, and browser playback.
- UDP 19302 for STUN discovery.
- UDP/TCP media ports for RTC and stream relay paths.

## Monitoring Access

`monitor.example.com` is additionally restricted by nginx with `MONITOR_ALLOWED_CIDR`. Use office static IP or VPN CIDR.
