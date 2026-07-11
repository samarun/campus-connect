import "server-only";

import { mkdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { DEFAULT_TENANT_SETTINGS } from "./tenant";
import type { TenantSettings } from "./types";

const DATA_DIR = join(process.cwd(), "data");
const TENANT_FILE = join(DATA_DIR, "tenant.json");
const MAX_LABEL_LENGTH = 80;

function cleanLabel(value: unknown, fallback: string) {
  if (typeof value !== "string") return fallback;
  const trimmed = value.trim().replace(/\s+/g, " ");
  if (!trimmed) return fallback;
  return trimmed.slice(0, MAX_LABEL_LENGTH);
}

function cleanSiteLabel(value: unknown, fallback: string, legacyValue: string) {
  const cleaned = cleanLabel(value, fallback);
  return cleaned === legacyValue ? fallback : cleaned;
}

function envDefault(name: string, fallback: string) {
  return cleanLabel(process.env[name], fallback);
}

function envSiteDefault(name: string, fallback: string, legacyValue: string) {
  return cleanSiteLabel(process.env[name], fallback, legacyValue);
}

export function defaultTenantSettings(): TenantSettings {
  return {
    platformName: envDefault("PLATFORM_NAME", DEFAULT_TENANT_SETTINGS.platformName),
    organizationName: envDefault("ORGANIZATION_NAME", DEFAULT_TENANT_SETTINGS.organizationName),
    primarySiteName: envDefault("PRIMARY_SITE_NAME", DEFAULT_TENANT_SETTINGS.primarySiteName),
    remoteSiteAName: envSiteDefault("REMOTE_SITE_A_NAME", DEFAULT_TENANT_SETTINGS.remoteSiteAName, "Remote Site A"),
    remoteSiteBName: envSiteDefault("REMOTE_SITE_B_NAME", DEFAULT_TENANT_SETTINGS.remoteSiteBName, "Remote Site B"),
    greetingModeName: envDefault("GREETING_MODE_NAME", DEFAULT_TENANT_SETTINGS.greetingModeName),
    programModeName: envDefault("PROGRAM_MODE_NAME", DEFAULT_TENANT_SETTINGS.programModeName)
  };
}

export function normalizeTenantSettings(value: Partial<TenantSettings>, fallback = defaultTenantSettings()): TenantSettings {
  return {
    platformName: cleanLabel(value.platformName, fallback.platformName),
    organizationName: cleanLabel(value.organizationName, fallback.organizationName),
    primarySiteName: cleanLabel(value.primarySiteName, fallback.primarySiteName),
    remoteSiteAName: cleanSiteLabel(value.remoteSiteAName, fallback.remoteSiteAName, "Remote Site A"),
    remoteSiteBName: cleanSiteLabel(value.remoteSiteBName, fallback.remoteSiteBName, "Remote Site B"),
    greetingModeName: cleanLabel(value.greetingModeName, fallback.greetingModeName),
    programModeName: cleanLabel(value.programModeName, fallback.programModeName)
  };
}

export async function getTenantSettings(): Promise<TenantSettings> {
  const fallback = defaultTenantSettings();
  try {
    const raw = await readFile(TENANT_FILE, "utf8");
    return normalizeTenantSettings(JSON.parse(raw) as Partial<TenantSettings>, fallback);
  } catch {
    return fallback;
  }
}

export async function setTenantSettings(value: Partial<TenantSettings>) {
  const next = normalizeTenantSettings(value, await getTenantSettings());
  await mkdir(DATA_DIR, { recursive: true });
  await writeFile(TENANT_FILE, JSON.stringify(next, null, 2), "utf8");
  return next;
}
