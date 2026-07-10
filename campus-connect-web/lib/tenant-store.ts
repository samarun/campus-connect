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

function envDefault(name: string, fallback: string) {
  return cleanLabel(process.env[name], fallback);
}

export function defaultTenantSettings(): TenantSettings {
  return {
    platformName: envDefault("PLATFORM_NAME", DEFAULT_TENANT_SETTINGS.platformName),
    organizationName: envDefault("ORGANIZATION_NAME", DEFAULT_TENANT_SETTINGS.organizationName),
    primarySiteName: envDefault("PRIMARY_SITE_NAME", DEFAULT_TENANT_SETTINGS.primarySiteName),
    remoteSiteAName: envDefault("REMOTE_SITE_A_NAME", DEFAULT_TENANT_SETTINGS.remoteSiteAName),
    remoteSiteBName: envDefault("REMOTE_SITE_B_NAME", DEFAULT_TENANT_SETTINGS.remoteSiteBName),
    greetingModeName: envDefault("GREETING_MODE_NAME", DEFAULT_TENANT_SETTINGS.greetingModeName),
    programModeName: envDefault("PROGRAM_MODE_NAME", DEFAULT_TENANT_SETTINGS.programModeName)
  };
}

export function normalizeTenantSettings(value: Partial<TenantSettings>, fallback = defaultTenantSettings()): TenantSettings {
  return {
    platformName: cleanLabel(value.platformName, fallback.platformName),
    organizationName: cleanLabel(value.organizationName, fallback.organizationName),
    primarySiteName: cleanLabel(value.primarySiteName, fallback.primarySiteName),
    remoteSiteAName: cleanLabel(value.remoteSiteAName, fallback.remoteSiteAName),
    remoteSiteBName: cleanLabel(value.remoteSiteBName, fallback.remoteSiteBName),
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
