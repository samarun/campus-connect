# AWS Deployment

Target: Ubuntu 24.04 LTS on `c7i.2xlarge`, Docker Compose, one Elastic IP. Upgrade path: `c7i.4xlarge`.

## 1. Launch EC2

1. Create an Ubuntu 24.04 LTS EC2 instance.
2. Start with `c7i.2xlarge`.
3. Attach gp3 storage. Use at least 100 GB for logs, monitoring data, and backups.
4. Attach the security group from [SECURITY_GROUPS.md](SECURITY_GROUPS.md).
5. Allocate and associate an Elastic IP.

## 2. DNS

Create `A` records pointing to the Elastic IP:

- `app.example.com`
- `rtc.example.com`
- `turn.example.com`
- `stream.example.com`
- `monitor.example.com`

Use your real customer/service domain names in `.env`:

- `APP_DOMAIN`
- `RTC_DOMAIN`
- `TURN_DOMAIN`
- `STREAM_DOMAIN`
- `MONITOR_DOMAIN`

Verify:

```bash
dig +short app.example.com
dig +short rtc.example.com
dig +short stream.example.com
```

## 3. Install Host Dependencies

```bash
sudo apt-get update
sudo apt-get install -y git
sudo mkdir -p /opt/campus-connect
sudo chown "$USER:$USER" /opt/campus-connect
cd /opt/campus-connect
git clone <your-repo-url> .
sudo ./scripts/install.sh
```

## 4. Configure Secrets

```bash
cp .env.example .env
chmod 600 .env
openssl rand -base64 48
```

Generate each role hash:

```bash
npm --prefix campus-connect-web install
npm --prefix campus-connect-web run hash-password -- "strong role password"
```

Put hashes and random secrets in `.env`. Do not commit `.env`.

Bcrypt hashes contain `$`, so wrap each password hash in single quotes:

```dotenv
ADMIN_PASSWORD_HASH='$2b$12$replace-with-generated-hash'
```

Set customer defaults before first deploy, or set them later from `/operator`:

```dotenv
PLATFORM_NAME='Campus Connect'
ORGANIZATION_NAME='Customer Name'
PRIMARY_SITE_NAME='Primary Site'
REMOTE_SITE_A_NAME='Remote Site A'
REMOTE_SITE_B_NAME='Remote Site B'
GREETING_MODE_NAME='Greeting Mode'
PROGRAM_MODE_NAME='Program Mode'
```

Confirm production configuration before starting containers:

```bash
./scripts/preflight.sh
```

## 5. TLS Certificates

After DNS resolves to the Elastic IP, bootstrap nginx for ACME challenge or run certbot with standalone mode during a maintenance window:

```bash
docker compose run --rm --service-ports certbot certonly \
  --standalone \
  -d app.example.com \
  -d rtc.example.com \
  -d turn.example.com \
  -d stream.example.com \
  -d monitor.example.com \
  --email admin@example.com \
  --agree-tos \
  --no-eff-email
```

Renew monthly:

```bash
docker compose run --rm certbot renew
docker compose restart nginx coturn
```

## 6. Deploy

```bash
./scripts/deploy.sh
./scripts/healthcheck.sh
```

Use `./scripts/deploy.sh` instead of raw `docker compose up`; it runs production preflight checks and renders `runtime/rtc/server.yaml`, `runtime/rtc/ingress.yaml`, `runtime/rtc/egress.yaml`, and `runtime/streaming/Server.xml`.

## 7. Customer Setup

1. Open `https://app.example.com/login`.
2. Log in as `admin` or `operator`.
3. Open `/operator`.
4. Use **Customer setup** to set the platform, organization, Primary Site, Remote Site A, Remote Site B, and mode names.
5. Refresh any open receiver/player windows.

## 8. Upgrade to c7i.4xlarge

Use [scripts/upgrade-instance-notes.md](../scripts/upgrade-instance-notes.md). Always run `./scripts/backup.sh` first.

## 9. Backup and Restore

The dedicated runbook is [BACKUP_RESTORE.md](BACKUP_RESTORE.md). Short version:

```bash
./scripts/backup.sh
aws s3 cp backups/campus-connect-*.tgz s3://YOUR-BUCKET/campus-connect/
```

Restore:

1. Stop containers: `docker compose down`.
2. Extract the backup.
3. Restore `.env` and config files.
4. Restore Docker volume tarballs with an Alpine container.
5. Run `./scripts/deploy.sh`.

Example volume restore:

```bash
docker volume create campus-connect_redis-data
docker run --rm \
  -v campus-connect_redis-data:/volume \
  -v "$PWD/restore:/backup" \
  alpine:3.21 sh -c "cd /volume && tar xzf /backup/campus-connect_redis-data.tar.gz"
```
