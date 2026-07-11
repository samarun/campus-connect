# Branding and Customer Setup

Campus Connect is now generic by default. The customer-facing brand and site labels live in the wrapper app and persisted tenant settings, not in upstream media-server source code.

## Runtime Setup

Before production use, log in as `admin` or `operator`, open `/operator`, and fill **Customer setup**:

- Platform name
- Customer / organization name
- Primary site name
- Campus B name
- Campus C name
- First mode name
- Second mode name

These values are stored in `campus-connect-web/data/tenant.json` inside the `campus-connect_app-data` Docker volume.

## Environment Defaults

Set these in `.env` for first boot defaults:

```dotenv
PLATFORM_NAME='Campus Connect'
ORGANIZATION_NAME='Customer Name'
PRIMARY_SITE_NAME='Primary Site'
REMOTE_SITE_A_NAME='Campus B'
REMOTE_SITE_B_NAME='Campus C'
GREETING_MODE_NAME='Greeting Mode'
PROGRAM_MODE_NAME='Program Mode'
```

Saved tenant settings override the `.env` defaults until the app data volume is reset or restored.

## Logo Policy

The default SaaS build uses text-only branding. There is no logo image in the app header, login page, metadata icon, or web manifest. Add a logo only for a customer-specific fork or paid customization where the asset and usage rights are confirmed.

## Do Not Brand by Editing Upstream Servers

Keep branding in:

- Next.js components
- Tenant settings
- Reverse proxy routing
- Documentation

Do not modify upstream RTC media server, stream relay, or player vendor source code for customer branding.
