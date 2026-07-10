# Campus Connect Capture Bridge

The Campus Connect Capture Bridge is the Rust-native companion for capture-card workflows. It does not replace the Next.js operator app. The web app remains the volunteer-facing control surface; the bridge runs on a capture machine connected to SDI/HDMI devices.

## Purpose

- Select Blackmagic, AJA, USB, virtual, or test-pattern capture sources.
- Lock the production format, such as `1080p30`.
- Choose alternate formats and frame rates for real-world hardware needs.
- Monitor the exact encoder command before running it.
- Send the final program to the stream relay over SRT.
- Provide a Rust/Tauri desktop UI and a daemon-first rack mode.

## Current Implementation

The bridge is now a Rust workspace in [capture-bridge](../capture-bridge):

- `capture-common`: shared API/status/config types
- `capture-core`: device discovery, validation, preflight, and media plan generation
- `capture-cli`: production fallback CLI
- `capture-daemon`: headless macOS/Linux rack service
- `capture-gui`: Tauri desktop app with a Rust/WASM Leptos frontend

Supported runnable paths now:

- macOS AVFoundation input through FFmpeg
- Linux v4l2 input through FFmpeg
- FFmpeg `lavfi` test pattern
- GStreamer v4l2/DeckLink SRT plan generation for Linux rack hosts
- SRT output to stream relay
- RTMP/file output command generation

Production extension points:

- Blackmagic DeckLink through FFmpeg when FFmpeg is built with the DeckLink SDK
- Blackmagic DeckLink through GStreamer on Linux rack hosts
- AJA through native AJA SDK integration
- WHIP/WebRTC publishing to RTC

## Format and Frame Rate Selection

List recommended formats:

```bash
cd capture-bridge
cargo run -p capture-cli -- list-formats
```

Common production choices:

| Format | Use |
| --- | --- |
| `1080p30` | Default Campus Connect production target |
| `1080p29.97` | Common broadcast/camera-chain format |
| `1080p25` | PAL-region camera-chain format |
| `720p30` | Lower-bandwidth fallback |
| `1080p50` / `1080p59.94` / `1080p60` | Only after hardware, upload, and receiver decode tests pass |

Override format and frame rate:

```bash
cargo run -p capture-cli -- plan --profile program-local-srt --format 1080p29.97
cargo run -p capture-cli -- plan --profile program-local-srt --format 1080p30 --frame-rate 25
cargo run -p capture-cli -- plan --profile program-local-srt --format 720p30 --video-bitrate-kbps 4500
```

The bridge will warn when selecting frame rates above 30 fps because they increase encoder load, network bandwidth, and receiver decode load.

## CLI and Local Stream Relay Test

Start local media:

```bash
./scripts/local-media.sh up
```

Run a test pattern from the bridge:

```bash
cd capture-bridge
cargo run -p capture-cli -- run --profile program-test-pattern
```

Open:

```text
http://localhost:3000/player/program
```

For the separate `relay-test-pattern` profile, use:

```text
http://localhost:3333/broadcast/test/llhls.m3u8
```

Stop the bridge with `Ctrl+C`.

## Capture Device Discovery

macOS:

```bash
cd capture-bridge
cargo run -p capture-cli -- list-devices avfoundation
```

Blackmagic DeckLink:

```bash
cargo run -p capture-cli -- list-devices decklink
```

If DeckLink does not appear, install Blackmagic Desktop Video and use an FFmpeg build compiled with DeckLink SDK support. Homebrew FFmpeg usually does not include DeckLink because the SDK is separately licensed.

Linux v4l2:

```bash
cargo run -p capture-cli -- list-devices v4l2
v4l2-ctl --list-devices
```

AJA:

Native AJA SDK support is planned. For now, use AJA Control Room, OBS with AJA support, or a hardware encoder for production output.

## Daemon API

Run the daemon locally:

```bash
./scripts/capture-bridge-daemon.sh --bind 127.0.0.1:4768
```

Endpoints:

- `GET /health`
- `GET /devices?backend=...`
- `GET /formats`
- `GET /profiles`
- `POST /profiles`
- `POST /validate`
- `POST /plan`
- `POST /sessions/start`
- `POST /sessions/stop`
- `GET /sessions/current`
- `GET /metrics`

LAN bind requires an API token:

```bash
CAPTURE_BRIDGE_API_TOKEN=replace-me ./scripts/capture-bridge-daemon.sh --bind 0.0.0.0:4768
```

Protected requests accept `Authorization: Bearer <token>` or `x-capture-bridge-token`.

## Tauri GUI

The GUI lives under `capture-bridge/apps/capture-gui`.

Simple launcher from repo root:

```bash
./scripts/capture-bridge-gui.sh
```

First-time setup on a machine with rustup:

```bash
./scripts/capture-bridge-gui.sh --install-tools
```

Validated pieces:

- Tauri Rust backend commands compile
- Leptos Rust/WASM frontend type-checks
- Shared Rust core powers CLI, daemon, and GUI commands
- GUI controls are wired for profile selection/saving, format/frame-rate selection, backend/device/encoder/output overrides, validation, live plan preview, dry run, start, stop, warnings, preflight checks, device scan, and logs

Build machine requirements for real desktop bundling:

```bash
cargo install tauri-cli --locked
cargo install trunk --locked
rustup target add wasm32-unknown-unknown
```

Run from `capture-bridge`:

```bash
cargo tauri dev --config apps/capture-gui/src-tauri/tauri.conf.json
```

## Linux Rack Mode

The rack path is daemon-first and does not require a desktop session.

Systemd service template:

```text
capture-bridge/packaging/systemd/capture-bridge-daemon.service
```

Recommended rack dependencies:

- FFmpeg with `libsrt`, `libx264`, and optional `h264_nvenc`/`h264_vaapi`
- GStreamer tools/plugins for v4l2 and DeckLink pipelines
- Blackmagic Desktop Video for DeckLink devices
- `v4l2-ctl` for USB capture inspection

## Profiles

Profiles live in [capture-bridge/config/capture.local.toml](../capture-bridge/config/capture.local.toml).

Current profiles:

- `program-local-srt`: local macOS camera/audio to local stream relay SRT
- `program-test-pattern`: synthetic 1080p test source to local stream relay SRT
- `relay-test-pattern`: synthetic 1080p test source to the `broadcast/test` stream relay stream
- `program-decklink-srt`: production-shaped Blackmagic DeckLink to production stream relay SRT
- `linux-rack-v4l2-srt`: Linux rack FFmpeg/v4l2 to production stream relay SRT
- `linux-rack-gstreamer-srt`: Linux rack GStreamer/v4l2 to production stream relay SRT
- `greeting-whip-placeholder`: future RTC WHIP capture profile

Plan before running:

```bash
cd capture-bridge
cargo run -p capture-cli -- plan --profile program-local-srt
```

Run with a dry check:

```bash
cargo run -p capture-cli -- run --profile program-local-srt --dry-run
```

Run for real:

```bash
cargo run -p capture-cli -- run --profile program-local-srt
```

## Production Recommendation

Use the bridge for Program Mode first. SRT to stream relay is the strongest path for a final program feed.

For Greeting Mode, keep the browser RTC flow for now unless the capture machine has a dedicated WHIP/WebRTC path tested end to end. The future desktop bridge should use either a GStreamer WebRTC/WHIP path or RTC-supported ingest path, with explicit stats for dropped frames, audio drift, bitrate, and reconnects.

## Desktop UI Roadmap

Next phase:

- Add live log streaming from the daemon/session process.
- Add dropped-frame and audio-drift parsing from FFmpeg/GStreamer stderr.
- Add operator dashboard status heartbeat.
- Add signed macOS and Linux release packages.

Keep the CLI core even after the desktop UI exists. It gives production teams a reliable fallback and makes rehearsal automation easier.
