import { campusLabel } from "./tenant";
import type { CampusKey, ReceiverKey, TenantSettings } from "./types";

const CAMPUS_ORDER: CampusKey[] = ["main", "b", "c"];
const RECEIVER_ORDER: ReceiverKey[] = ["main", "campus-b", "campus-c", "grid"];

const CAMPUS_FALLBACKS: Record<CampusKey, string> = {
  main: "primary-site",
  b: "remote-site-a",
  c: "remote-site-b"
};

const RECEIVER_FALLBACKS: Record<ReceiverKey, string> = {
  main: "primary-site",
  "campus-b": "remote-site-a",
  "campus-c": "remote-site-b",
  grid: "grid"
};

const CAMPUS_ALIASES: Record<string, CampusKey> = {
  main: "main",
  primary: "main",
  "primary-site": "main",
  b: "b",
  "campus-b": "b",
  "remote-site-a": "b",
  c: "c",
  "campus-c": "c",
  "remote-site-b": "c"
};

const RECEIVER_ALIASES: Record<string, ReceiverKey> = {
  main: "main",
  primary: "main",
  "primary-site": "main",
  "primary-site-feed": "main",
  b: "campus-b",
  "campus-b": "campus-b",
  "remote-site-a": "campus-b",
  "remote-site-a-feed": "campus-b",
  c: "campus-c",
  "campus-c": "campus-c",
  "remote-site-b": "campus-c",
  "remote-site-b-feed": "campus-c",
  grid: "grid",
  "3-site-grid": "grid",
  "three-site-grid": "grid"
};

function normalizeSlug(value: string) {
  return slugifyLabel(value, "");
}

export function slugifyLabel(value: string, fallback: string) {
  const slug = value
    .normalize("NFKD")
    .replace(/[\u0300-\u036f]/g, "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .replace(/-{2,}/g, "-");

  return slug || fallback;
}

function uniqueSlugs<T extends string>(items: Array<[T, string]>, fallbacks: Record<T, string>, reserved = new Set<string>()) {
  const used = new Set(reserved);
  const slugs = {} as Record<T, string>;

  for (const [key, label] of items) {
    const base = slugifyLabel(label, fallbacks[key]);
    let candidate = base;

    if (used.has(candidate)) {
      candidate = `${base}-${fallbacks[key]}`;
    }

    let index = 2;
    while (used.has(candidate)) {
      candidate = `${base}-${fallbacks[key]}-${index}`;
      index += 1;
    }

    used.add(candidate);
    slugs[key] = candidate;
  }

  return slugs;
}

export function campusSlugMap(settings: TenantSettings): Record<CampusKey, string> {
  return uniqueSlugs(
    CAMPUS_ORDER.map((campus) => [campus, campusLabel(campus, settings)]),
    CAMPUS_FALLBACKS
  );
}

export function receiverSlugMap(settings: TenantSettings): Record<ReceiverKey, string> {
  return uniqueSlugs(
    [
      ["main", settings.primarySiteName],
      ["campus-b", settings.remoteSiteAName],
      ["campus-c", settings.remoteSiteBName],
      ["grid", "grid"]
    ],
    RECEIVER_FALLBACKS
  );
}

export function campusHref(campus: CampusKey, settings: TenantSettings) {
  return `/campus/${campusSlugMap(settings)[campus]}`;
}

export function receiverHref(receiver: ReceiverKey, settings: TenantSettings) {
  return `/receiver/${receiverSlugMap(settings)[receiver]}`;
}

export function resolveCampusParam(value: string, settings: TenantSettings): CampusKey | null {
  const slug = normalizeSlug(value);
  const dynamicEntry = Object.entries(campusSlugMap(settings)).find(([, dynamicSlug]) => dynamicSlug === slug);
  if (dynamicEntry) return dynamicEntry[0] as CampusKey;
  return CAMPUS_ALIASES[slug] ?? null;
}

export function resolveReceiverParam(value: string, settings: TenantSettings): ReceiverKey | null {
  const slug = normalizeSlug(value);
  const dynamicEntry = Object.entries(receiverSlugMap(settings)).find(([, dynamicSlug]) => dynamicSlug === slug);
  if (dynamicEntry) return dynamicEntry[0] as ReceiverKey;
  return RECEIVER_ALIASES[slug] ?? null;
}

export function receiverLinks(settings: TenantSettings) {
  return RECEIVER_ORDER.map((receiver) => ({
    receiver,
    href: receiverHref(receiver, settings)
  }));
}
