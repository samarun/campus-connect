import {
  DEFAULT_CAMPUS_LABELS,
  DEFAULT_GREETING_MODE_NAME,
  DEFAULT_ORGANIZATION_NAME,
  DEFAULT_PLATFORM_NAME,
  DEFAULT_PROGRAM_MODE_NAME,
  DEFAULT_RECEIVER_LABELS,
  DEFAULT_ROLE_LABELS
} from "./constants";
import type { CampusKey, Mode, ReceiverKey, Role, RoomName, TenantSettings } from "./types";

export const DEFAULT_TENANT_SETTINGS: TenantSettings = {
  platformName: DEFAULT_PLATFORM_NAME,
  organizationName: DEFAULT_ORGANIZATION_NAME,
  primarySiteName: DEFAULT_CAMPUS_LABELS.main,
  remoteSiteAName: DEFAULT_CAMPUS_LABELS.b,
  remoteSiteBName: DEFAULT_CAMPUS_LABELS.c,
  greetingModeName: DEFAULT_GREETING_MODE_NAME,
  programModeName: DEFAULT_PROGRAM_MODE_NAME
};

export function tenantCampusLabels(settings: TenantSettings): Record<CampusKey, string> {
  return {
    main: settings.primarySiteName,
    b: settings.remoteSiteAName,
    c: settings.remoteSiteBName
  };
}

export function tenantRoleLabels(settings: TenantSettings): Record<Role, string> {
  return {
    ...DEFAULT_ROLE_LABELS,
    main: settings.primarySiteName,
    "campus-b": settings.remoteSiteAName,
    "campus-c": settings.remoteSiteBName
  };
}

export function tenantReceiverLabels(settings: TenantSettings): Record<ReceiverKey, string> {
  return {
    ...DEFAULT_RECEIVER_LABELS,
    main: `${settings.primarySiteName} Feed`,
    "campus-b": `${settings.remoteSiteAName} Feed`,
    "campus-c": `${settings.remoteSiteBName} Feed`
  };
}

export function roleLabel(role: Role, settings: TenantSettings) {
  return tenantRoleLabels(settings)[role];
}

export function campusLabel(campus: CampusKey, settings: TenantSettings) {
  return tenantCampusLabels(settings)[campus];
}

export function receiverLabel(receiver: ReceiverKey, settings: TenantSettings) {
  return tenantReceiverLabels(settings)[receiver];
}

export function modeLabel(mode: Mode, settings: TenantSettings) {
  return mode === "greeting" ? settings.greetingModeName : settings.programModeName;
}

export function roomLabel(room: RoomName, settings: TenantSettings) {
  if (room === "campus-greeting") return settings.greetingModeName;
  if (room === "campus-program") return settings.programModeName;
  return "Backstage";
}

export function campusList(settings: TenantSettings) {
  const labels = tenantCampusLabels(settings);
  return [labels.main, labels.b, labels.c];
}
