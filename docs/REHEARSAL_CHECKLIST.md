# Rehearsal Checklist

Run this before any production event that depends on Campus Connect.

## One Day Before

- Confirm EC2 instance is `running`.
- Confirm Elastic IP is associated.
- Confirm DNS resolves for all domains.
- Confirm certificates are not close to expiry.
- Run `./scripts/backup.sh`.
- Run `./scripts/healthcheck.sh`.
- Confirm Grafana and Uptime Kuma are accessible from the allowed office or VPN IP.
- Confirm customer setup labels are correct in `/operator`.

## Primary Site

- Login as `Primary Site`.
- Open `/test` and confirm camera, microphone, and speaker.
- Join Greeting Mode and verify Primary Site video/audio.
- Open `/receiver/campus-b` and `/receiver/campus-c` on the production output path if needed.
- Send a test SRT stream:

```text
srt://stream.example.com:9999?streamid=default/broadcast/test
```

- Send program stream:

```text
srt://stream.example.com:9999?streamid=default/broadcast/program
```

## Remote Site A

- Login as `Remote Site A`.
- Open `/test` and confirm camera, microphone, and speaker.
- Join Greeting Mode and verify bidirectional audio/video.
- Switch to `/player/program` and confirm program playback.
- Confirm local display audio is routed correctly.

## Remote Site B

- Login as `Remote Site B`.
- Open `/test` and confirm camera, microphone, and speaker.
- Join Greeting Mode and verify bidirectional audio/video.
- Switch to `/player/program` and confirm program playback.
- Confirm local display audio is routed correctly.

## Operator

- Login as `Operator`.
- Switch to Greeting Mode.
- Confirm Primary Site, Remote Site A, and Remote Site B are online.
- Verify 3-way grid.
- Verify each full-screen receiver page.
- Switch to Program Mode.
- Confirm remote site publish controls are unavailable or muted.
- Confirm the program player receives `broadcast/program`.

## Go / No-Go

Go when:

- All three sites appear online in Greeting Mode.
- Primary Site receives Remote Site A and Remote Site B feeds.
- Remote sites receive Primary Site return feed.
- Program playback works on both remote sites.
- Monitoring is green.

No-go when:

- RTC media cannot connect from a site browser.
- SRT ingest is not visible in the player.
- TURN is required by a site network and is failing.
- CPU or network saturation appears before the event starts.
