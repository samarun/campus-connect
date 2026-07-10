# Campus Connect Capture Bridge

Rust-first capture-station product for Campus Connect.

The Campus Connect web app remains the browser operator/receiver surface. This bridge is the native capture layer for SDI/HDMI devices, format locking, stream relay SRT handoff, and future RTC hardware ingest.

## Workspace

- `crates/capture-common`: shared API/status/config types
- `crates/capture-core`: device discovery, profile resolution, validation, preflight, command planning
- `crates/capture-cli`: CLI fallback, binary `capture-bridge`
- `crates/capture-daemon`: headless rack daemon, binary `capture-daemon`
- `apps/capture-gui/src-tauri`: Tauri desktop backend with local session control
- `apps/capture-gui/frontend`: Rust/WASM Leptos GUI for profile save/load, format, source, output, preflight, and start/stop control

## CLI

```bash
cargo run -p capture-cli -- list-devices avfoundation
cargo run -p capture-cli -- list-formats
cargo run -p capture-cli -- validate
cargo run -p capture-cli -- preflight --profile program-test-pattern
cargo run -p capture-cli -- plan --profile program-local-srt
cargo run -p capture-cli -- run --profile program-test-pattern --dry-run
```

Helper scripts from repo root:

```bash
./scripts/capture-bridge-list-devices.sh avfoundation
./scripts/capture-bridge-plan.sh --profile program-local-srt --format 1080p30
./scripts/capture-bridge-test-pattern-srt.sh --dry-run
```

Actual local stream relay test stream:

```bash
./scripts/local-media.sh up
./scripts/capture-bridge-test-pattern-srt.sh
```

Stop with `Ctrl+C`.

## Daemon

Local daemon:

```bash
./scripts/capture-bridge-daemon.sh --bind 127.0.0.1:4768
```

API:

```bash
curl http://127.0.0.1:4768/health
curl http://127.0.0.1:4768/profiles
curl http://127.0.0.1:4768/metrics
```

LAN bind requires a token:

```bash
CAPTURE_BRIDGE_API_TOKEN=replace-me \
  ./scripts/capture-bridge-daemon.sh --bind 0.0.0.0:4768
```

Then call protected endpoints with:

```bash
curl -H 'Authorization: Bearer replace-me' http://rack-host:4768/profiles
```

## Tauri GUI

Simple launcher from repo root:

```bash
./scripts/capture-bridge-gui.sh
```

If this is the first time on a machine with rustup already installed:

```bash
./scripts/capture-bridge-gui.sh --install-tools
```

Rust backend check:

```bash
cargo check -p capture-gui
```

Rust/WASM frontend check:

```bash
cargo check --manifest-path apps/capture-gui/frontend/Cargo.toml
```

Desktop dev requires Tauri CLI, Trunk, and a Rust toolchain with WASM target support:

```bash
cargo install tauri-cli --locked
cargo install trunk --locked
rustup target add wasm32-unknown-unknown
cargo tauri dev --config apps/capture-gui/src-tauri/tauri.conf.json
```

This machine uses Homebrew Rust without `rustup`, so the GUI code is checked natively here. For real GUI bundling, install Rust through rustup on the build machine.

## Linux Rack

The Linux rack path is designed to run without a desktop session:

```bash
sudo cp packaging/systemd/capture-daemon.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now capture-daemon
```

Recommended packages:

- FFmpeg with `libsrt`, `libx264`, and optional `h264_nvenc`/`h264_vaapi`
- GStreamer core tools and plugins
- Blackmagic Desktop Video for DeckLink
- `v4l2-ctl` for USB/v4l2 capture inspection

## Production Defaults

- Default format: `1080p30`
- Fallback format: `720p30`
- Program path: SRT to stream relay
- RTC hardware ingest: future WHIP/Ingress phase
- AJA: native SDK adapter after hardware/driver target is confirmed
