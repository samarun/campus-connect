# Backup and Restore

Backups protect configuration, TLS certificates, app state, Redis state, and monitoring data. They do not replace AWS EBS snapshots; use both.

## Backup Schedule

- Before every production change: run `./scripts/backup.sh`.
- Weekly before a production event: run `./scripts/backup.sh` and copy the archive off the EC2 instance.
- Monthly: create an AWS EBS snapshot after a successful event.

## Create Backup

```bash
cd /opt/campus-connect
./scripts/backup.sh
ls -lh backups/
```

Copy the resulting archive to S3 or another trusted storage location:

```bash
aws s3 cp backups/campus-connect-*.tgz s3://YOUR-BUCKET/campus-connect/
```

## Restore on Same Host

1. Stop services:

```bash
docker compose down
```

2. Extract the backup:

```bash
mkdir -p restore
tar xzf backups/campus-connect-YYYYMMDDTHHMMSSZ.tgz -C restore
```

3. Restore `.env`:

```bash
cp restore/campus-connect-YYYYMMDDTHHMMSSZ/env.secure .env
chmod 600 .env
```

4. Restore Docker volumes as needed:

```bash
docker volume create campus-connect_redis-data
docker run --rm \
  -v campus-connect_redis-data:/volume \
  -v "$PWD/restore/campus-connect-YYYYMMDDTHHMMSSZ:/backup" \
  alpine:3.21 sh -c "cd /volume && tar xzf /backup/campus-connect_redis-data.tar.gz"
```

Repeat for:

- `campus-connect_app-data`
- `campus-connect_letsencrypt`
- `campus-connect_prometheus-data`
- `campus-connect_grafana-data`
- `campus-connect_uptime-kuma-data`

5. Redeploy:

```bash
./scripts/deploy.sh
./scripts/healthcheck.sh
```

## Restore on New EC2 Host

1. Launch Ubuntu 24.04 LTS on the replacement instance.
2. Attach the same security group.
3. Associate the Elastic IP with the new instance.
4. Install Docker with `sudo ./scripts/install.sh`.
5. Copy the backup archive to the new host.
6. Restore `.env` and volumes.
7. Run `./scripts/deploy.sh`.
8. Rehearse Greeting Mode and Program Mode before production use.

## Verification

Run:

```bash
./scripts/healthcheck.sh
docker compose ps
```

Then verify:

- `https://app.example.com/health`
- `https://app.example.com/login`
- `https://monitor.example.com/grafana/`
- `https://app.example.com/player/program`
