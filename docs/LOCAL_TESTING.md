# Local Testing

Use this before deploying to AWS. There are three levels of local testing.

## Level 1: Web App and UI

This tests login, role routing, operator dashboard, site pages, receiver pages, player page, device test, health endpoint, and generic customer branding.

Start the local web app:

```bash
./scripts/local-web.sh
```

Open:

```text
http://localhost:3000/login
```

Use this local-only password for every role:

```text
campus-connect-test-123
```

Test these pages:

- `/login`
- `/operator`
- `/campus/main`
- `/campus/b`
- `/campus/c`
- `/receiver/main`
- `/receiver/campus-b`
- `/receiver/campus-c`
- `/receiver/main?view=grid`
- `/player/program`
- `/test`
- `/health`

Expected result:

- Login works for `admin`, `operator`, `main`, `campus-b`, and `campus-c`.
- Generic Campus Connect branding appears on all pages.
- Operator can save customer/site labels from **Customer setup**.
- Operator can switch Greeting Mode and Program Mode.
- Device test can access local camera, microphone, and speaker.
- `/api/health` returns JSON.

Browser video and stream playback will not fully work in Level 1 because the local media servers are not running.

## Level 2: Real Local Media

This tests actual browser video and actual SRT ingest/playback without AWS charges.

Start the media stack:

```bash
./scripts/local-media.sh up
```

For same-machine testing, the local RTC container advertises `127.0.0.1` so the browser can establish the media connection through Docker port mappings. If you test from another device on your LAN or over Tailscale, advertise the host IP that those devices can reach:

```bash
LOCAL_RTC_NODE_IP=<host-lan-or-tailscale-ip> ./scripts/local-media.sh up
```

Then set the web app to use the same reachable host for signaling, for example `NEXT_PUBLIC_RTC_URL=ws://<host-lan-or-tailscale-ip>:7880`. If participants show as online but receiver/operator pages stay black, reload the site pages after restarting the media stack and confirm the advertised IP is reachable from the viewing browser.

Start the web app in another terminal:

```bash
./scripts/local-web.sh
```

Use this local-only password for every role:

```text
campus-connect-test-123
```

### Test Greeting Mode

1. Login as `operator` in one browser window and open `/operator`.
2. Login as `main` in a second browser profile or private window and open `/campus/main`.
3. Login as `campus-b` in a third browser profile or private window and open `/campus/b`.
4. Login as `campus-c` in a fourth browser profile or private window and open `/campus/c`.
5. In `/operator`, switch to Greeting Mode.
6. Allow camera and microphone permissions in each site window.
7. Confirm the operator grid sees all site feeds.
8. Open `/receiver/main?view=grid` and press full screen.

On one laptop, use different browser profiles, private windows, or different browsers. A normal tab shares the same session cookie, so it cannot be logged in as multiple roles at once.

### Test Program Streaming

Publish a test stream with OBS, FFmpeg, or the Rust Capture Bridge.

Rust Capture Bridge test pattern:

```bash
./scripts/capture-bridge-test-pattern-srt.sh
```

Rust Capture Bridge daemon API smoke test:

```bash
./scripts/capture-bridge-daemon.sh --bind 127.0.0.1:4768
curl http://127.0.0.1:4768/health
curl http://127.0.0.1:4768/profiles
```

OBS SRT server:

```text
srt://localhost:9999?streamid=default/broadcast/program
```

OBS RTMP server:

```text
rtmp://localhost:1936/broadcast
```

OBS RTMP stream key:

```text
program
```

FFmpeg test pattern:

```bash
./scripts/local-program-test-stream.sh
```

Open:

```text
http://localhost:3000/player/program
```

Expected result:

- The branded player shows the `Program` stream.
- Direct playlist URL returns HTTP 200 after the stream starts:

```bash
curl http://localhost:3333/broadcast/program/llhls.m3u8
```

The included FFmpeg test stream is 1080p30. For production bitrate and encoder guidance, see [MEDIA_QUALITY.md](MEDIA_QUALITY.md).

Stop local media:

```bash
./scripts/local-media.sh down
```

## Level 3: Production Config Validation

This validates the Docker Compose topology and web container build without needing AWS.

Run:

```bash
npm --prefix campus-connect-web run typecheck
npm --prefix campus-connect-web run build
npm --prefix campus-connect-web audit --audit-level=moderate
docker build -t campus-connect-web:local ./campus-connect-web
```

Validate Compose with dummy env values:

```bash
export PUBLIC_IP=203.0.113.10
export APP_DOMAIN=app.example.com
export RTC_DOMAIN=rtc.example.com
export TURN_DOMAIN=turn.example.com
export STREAM_DOMAIN=stream.example.com
export MONITOR_DOMAIN=monitor.example.com
export MONITOR_ALLOWED_CIDR=203.0.113.0/24
export APP_BASE_URL=https://app.example.com
export NEXT_PUBLIC_RTC_URL=wss://rtc.example.com
export RTC_API_URL=http://host.docker.internal:7880
export NEXT_PUBLIC_STREAM_LLHLS_BASE_URL=https://stream.example.com
export RELAY_SRT_PUBLISH_URL=srt://stream.example.com:9999?streamid=default/broadcast/program
export PLATFORM_NAME="Campus Connect"
export ORGANIZATION_NAME="Demo Organization"
export PRIMARY_SITE_NAME="Primary Site"
export REMOTE_SITE_A_NAME="Campus B"
export REMOTE_SITE_B_NAME="Campus C"
export GREETING_MODE_NAME="Greeting Mode"
export PROGRAM_MODE_NAME="Program Mode"
export SESSION_SECRET=dummy_session_secret_123456789_dummy_session_secret_123456789
export RTC_API_KEY=dummy_key
export RTC_API_SECRET=dummy_secret
export TURN_STATIC_AUTH_SECRET=dummy_turn_secret
export ADMIN_PASSWORD_HASH=dummy_hash
export OPERATOR_PASSWORD_HASH=dummy_hash
export MAIN_PASSWORD_HASH=dummy_hash
export CAMPUS_B_PASSWORD_HASH=dummy_hash
export CAMPUS_C_PASSWORD_HASH=dummy_hash
export GRAFANA_ADMIN_USER=admin
export GRAFANA_ADMIN_PASSWORD=dummy_password
export RTC_SERVER_IMAGE=livekit/livekit-server:v1.13.3
export RTC_INGRESS_IMAGE=livekit/ingress:v1.5.0
export RTC_EGRESS_IMAGE=livekit/egress:v1.13.0
export STREAMING_RELAY_IMAGE=ovenmedialabs/ovenmediaengine:v0.20.5

mkdir -p runtime/rtc runtime/streaming
envsubst < rtc.yaml > runtime/rtc/server.yaml
envsubst < config/rtc/ingress.yaml > runtime/rtc/ingress.yaml
envsubst < config/rtc/egress.yaml > runtime/rtc/egress.yaml
envsubst < config/streaming/Server.xml > runtime/streaming/Server.xml
docker compose config --quiet
```

Expected result:

- TypeScript passes.
- Production build passes.
- Audit reports zero moderate-or-higher vulnerabilities.
- Docker image builds.
- Compose config validates.

## Remote No-AWS Test Options

If the sites are not on the same LAN, you have three no-AWS choices:

- Tailscale: easiest private network test. Install Tailscale on each test machine and use the host's Tailscale IP for app, signaling, and stream relay. This avoids public DNS and AWS, but every test machine needs Tailscale.
- Cloudflare Tunnel or ngrok: good for HTTPS web app and signaling, but WebRTC UDP and SRT UDP are limited or awkward. It is useful for UI testing, not a faithful media rehearsal.
- A spare local machine with port forwarding: closest to AWS behavior if your router can forward RTC UDP, SRT UDP, and HTTP ports. You still need dynamic DNS or a temporary domain.

For real remote-site confidence, Tailscale is usually the least painful pre-AWS option. For final sign-off, use EC2 because TURN, public UDP, DNS, and TLS are part of the real system.

## Full AWS Media Test

Do the full media test on the EC2 instance before production because the final stack expects:

- Public DNS
- Valid TLS certificates
- Public WebRTC UDP ports
- TURN with a reachable public IP
- SRT UDP ingress

The full AWS media test is covered in [REHEARSAL_CHECKLIST.md](REHEARSAL_CHECKLIST.md).
