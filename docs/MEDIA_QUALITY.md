# Media Quality

Campus Connect targets visually lossless 1080p production, not 4K.

True mathematical lossless video is not realistic for browser WebRTC or public-internet LL-HLS. Use this platform target instead:

- Resolution: 1920x1080
- Frame rate: 30 fps
- Greeting Mode: high-quality WebRTC with an 8 Mbps camera target
- Program Mode: SRT ingest with a 12-20 Mbps H.264 program target
- Audio: 48 kHz source audio, stable clocking, no Bluetooth devices in production

## Greeting Mode

The web app publishes camera video with a 1080p30 target and disables adaptive stream and dynacast in the wrapper UI. This keeps receiver and operator screens from being served a lower-resolution layer just because a tile is small.

Production endpoint guidance:

- Use wired Ethernet for every site endpoint.
- Use Chrome or Edge on a dedicated production computer.
- Use a 1080p camera or a clean HDMI/SDI capture device set to 1080p30.
- Route camera and microphone from the same hardware chain when possible; separate USB camera plus separate USB audio can drift more easily.
- Avoid Bluetooth microphones, Bluetooth speakers, Wi-Fi, VPNs, and overloaded laptops.
- Budget about 8-10 Mbps uplink per publishing site and 16-30 Mbps downlink for grid/receiver views.

If a laptop cannot hold 1080p30 locally, the browser may still lower actual camera capture quality. Check browser camera settings and the quality line under the grid.

## Program Mode

Use the stream relay for the final program feed. The relay is configured to bypass audio/video encoding, so quality is mainly controlled by the Primary Site encoder.

Recommended OBS or hardware encoder settings:

- Output: 1920x1080, 30 fps
- Video codec: H.264 High Profile
- Rate control: CBR
- Video bitrate: 12 Mbps minimum, 16-20 Mbps preferred when bandwidth is stable
- Keyframe interval: 2 seconds
- B-frames: off for lowest latency, on only if latency is less important
- Audio codec: AAC
- Audio sample rate: 48 kHz
- Audio bitrate: 192-320 kbps
- SRT latency: 250-800 ms depending on WAN quality

LL-HLS playback has a few seconds of latency by design. If remote sites need sub-second program playback later, enable and test WebRTC playback separately from LL-HLS.

## Local 1080p Test

Start local media and web services:

```bash
./scripts/local-media.sh up
./scripts/local-web.sh
```

Publish the local 1080p program test stream:

```bash
./scripts/local-program-test-stream.sh
```

Then open:

```text
http://localhost:3000/player/program
```

For Greeting Mode, test with separate browser profiles or devices. Multiple roles in normal tabs share the same login cookie and will overwrite each other.

## When Quality Drops

Check these first:

- Browser camera permission is using the correct 1080p device.
- No endpoint is on Wi-Fi.
- CPU is not pinned on the publishing laptop.
- Uplink packet loss is below 1%.
- The same room is not opened many times on one laptop during testing.
- OBS or the hardware encoder is actually sending 1080p, not 720p.

Upgrade the EC2 host from c7i.2xlarge to c7i.4xlarge if server CPU, packet loss, or media worker pressure stays high during a full rehearsal.
