# Operator Guide

## Login

1. Open `https://app.example.com/login`.
2. Choose `Operator`.
3. Enter the operator password.

## Customer Setup

Before the first production event, open `/operator` and set the customer/platform labels. Receiver windows should be refreshed after saving so they show the latest names.

## Greeting Mode

1. Open `/operator`.
2. Select `Greeting Mode`.
3. Confirm Primary Site, Campus B, and Campus C show Online.
4. Use the live grid to verify camera framing and audio presence.
5. Open receiver outputs as needed:
   - `/receiver/main`
   - `/receiver/campus-b`
   - `/receiver/campus-c`
   - `/receiver/main?view=grid`

## Program Mode

1. Select `Program Mode` before the program handoff.
2. Confirm remote site microphones and cameras start muted/off.
3. Confirm the Primary Site encoder is sending SRT to:

```text
srt://stream.example.com:9999?streamid=default/broadcast/program
```

4. Open `/player/program` and confirm the branded player receives the feed.
5. Leave Program player quality on `Auto adaptive` unless troubleshooting.
6. Ask remote site volunteers to confirm stable playback.

## Rehearsal Checklist

- Login works for every role.
- Customer setup labels are correct.
- Device test passes at each site.
- Greeting Mode shows all three sites in under a few seconds.
- Receiver full screen works on the production output machine.
- Program SRT ingest is stable for 10 minutes.
- LL-HLS player reload works.
- Monitoring dashboard shows services healthy.
- Backup completes before the production event.
