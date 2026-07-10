#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR/campus-connect-web"

# Local-only credentials. Do not use these values in production.
LOCAL_HASH='$2b$12$RuJf6ICJFxXjh9oXXbKX.uLkSUk1MP6pMsRcIYwPoBPZTu8tUuY86'

export SESSION_SECRET="${SESSION_SECRET:-local_dev_session_secret_campus_connect_please_replace}"
export RTC_API_KEY="${RTC_API_KEY:-local_dev_key}"
export RTC_API_SECRET="${RTC_API_SECRET:-local_dev_secret}"
export RTC_API_URL="${RTC_API_URL:-http://localhost:7880}"
export NEXT_PUBLIC_RTC_URL="${NEXT_PUBLIC_RTC_URL:-ws://localhost:7880}"
export NEXT_PUBLIC_STREAM_LLHLS_BASE_URL="${NEXT_PUBLIC_STREAM_LLHLS_BASE_URL:-http://localhost:3333}"
export RELAY_SRT_PUBLISH_URL="${RELAY_SRT_PUBLISH_URL:-srt://localhost:9999?streamid=default/broadcast/program}"
export PLATFORM_NAME="${PLATFORM_NAME:-Campus Connect}"
export ORGANIZATION_NAME="${ORGANIZATION_NAME:-Demo Organization}"
export PRIMARY_SITE_NAME="${PRIMARY_SITE_NAME:-Primary Site}"
export REMOTE_SITE_A_NAME="${REMOTE_SITE_A_NAME:-Remote Site A}"
export REMOTE_SITE_B_NAME="${REMOTE_SITE_B_NAME:-Remote Site B}"
export GREETING_MODE_NAME="${GREETING_MODE_NAME:-Greeting Mode}"
export PROGRAM_MODE_NAME="${PROGRAM_MODE_NAME:-Program Mode}"
export ADMIN_PASSWORD_HASH="${ADMIN_PASSWORD_HASH:-$LOCAL_HASH}"
export OPERATOR_PASSWORD_HASH="${OPERATOR_PASSWORD_HASH:-$LOCAL_HASH}"
export MAIN_PASSWORD_HASH="${MAIN_PASSWORD_HASH:-$LOCAL_HASH}"
export CAMPUS_B_PASSWORD_HASH="${CAMPUS_B_PASSWORD_HASH:-$LOCAL_HASH}"
export CAMPUS_C_PASSWORD_HASH="${CAMPUS_C_PASSWORD_HASH:-$LOCAL_HASH}"

echo "Starting Campus Connect local web app"
echo "URL: http://localhost:3000/login"
echo "Local test password for every role: campus-connect-test-123"
npm run dev
