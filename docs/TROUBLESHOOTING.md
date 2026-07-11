# Troubleshooting

## Containers

```bash
docker compose ps
docker compose logs -f campus-connect-web rtc-server streaming-relay nginx
./scripts/healthcheck.sh
```

## Login Fails

- Confirm `.env` contains bcrypt hashes, not plaintext passwords.
- Confirm bcrypt hashes are wrapped in single quotes so shell scripts do not expand `$` characters.
- Regenerate a hash:

```bash
npm --prefix campus-connect-web run hash-password -- "new strong password"
```

- Restart the web container:

```bash
docker compose restart campus-connect-web
```

## RTC Connection Fails

- Confirm DNS for `rtc.example.com` points to the Elastic IP.
- Confirm security group allows TCP `443`, TCP `7881`, and UDP `50000-60000`.
- Confirm the host firewall allows Docker bridge containers to reach the RTC signaling port on the host:

```bash
sudo ufw allow from 172.16.0.0/12 to any port 7880 proto tcp
sudo ufw reload
docker compose exec nginx wget -S -O- http://host.docker.internal:7880/ || true
```

- Check `runtime/rtc/server.yaml` was rendered.
- Check `docker compose logs rtc-server`.

## Participants Are Online but Video Is Blank Locally

- Confirm the receiver or operator browser can reach the media IP advertised by `./scripts/local-media.sh up`.
- For same-machine testing, use the default `LOCAL_RTC_NODE_IP=127.0.0.1`.
- For LAN or Tailscale testing, restart media with `LOCAL_RTC_NODE_IP=<host-lan-or-tailscale-ip> ./scripts/local-media.sh up`.
- Set `NEXT_PUBLIC_RTC_URL=ws://<same-reachable-host>:7880` when viewing from another device.
- Reload the site publisher pages after restarting media so they publish against the current candidates.

## Feeds Disappear While Testing Multiple Roles

Do not test `admin`, `operator`, `Primary Site`, `Remote Site A`, and `Remote Site B` in multiple tabs of the same browser profile. They share the same session cookie, so the most recent login replaces the earlier role for every tab on the domain.

Use separate devices, separate Chrome profiles, Incognito plus normal browser windows, or different browsers such as Chrome, Safari, and Firefox.

Also confirm the current mode:

- In `Greeting Mode`, all site pages can publish camera and microphone.
- In `Program Mode`, only `Primary Site` publishes to the RTC room; remote sites are intentionally receive-only.
- `/receiver/main`, `/receiver/campus-b`, `/receiver/campus-c`, and `/receiver/main?view=grid` are for the greeting RTC room.
- Program playback uses `/player/program` and requires an active SRT/RTMP stream into the streaming relay.

## TURN Fails

- Confirm `turn.example.com` certificate exists.
- Confirm security group allows UDP/TCP `3478` and TCP `5349`.
- Check coturn logs:

```bash
docker compose logs -f coturn
```

## SRT Ingest Fails

- Confirm the encoder sends:

```text
srt://stream.example.com:9999?streamid=default/broadcast/program
```

- Confirm security group allows UDP `9999` from the encoder source IP.
- Check:

```bash
docker compose logs -f streaming-relay
```

## LL-HLS Player Is Blank

- Confirm the stream exists:

```bash
curl https://stream.example.com/broadcast/program/llhls.m3u8
```

- Confirm `https://app.example.com/player/program` loads `/vendor/ovenplayer/ovenplayer.js`.
- Confirm the encoder is sending H.264 video and AAC audio.

## Monitoring Is Inaccessible

- Confirm your public IP is inside `MONITOR_ALLOWED_CIDR`.
- Confirm nginx rendered the monitor server block:

```bash
docker compose exec nginx nginx -T | grep monitor.example.com
```
