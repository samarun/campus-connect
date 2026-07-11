import type { CampusKey, ReceiverKey, Role, RoomName } from "./types";

export const DEFAULT_PLATFORM_NAME = "Campus Connect";
export const DEFAULT_ORGANIZATION_NAME = "Your Organization";
export const DEFAULT_GREETING_MODE_NAME = "Greeting Mode";
export const DEFAULT_PROGRAM_MODE_NAME = "Program Mode";

export const ROOMS: Record<string, RoomName> = {
  greeting: "campus-greeting",
  program: "campus-program",
  backstage: "campus-backstage"
};

export const DEFAULT_ROLE_LABELS: Record<Role, string> = {
  admin: "Administrator",
  operator: "Operator",
  main: "Primary Site",
  "campus-b": "Campus B",
  "campus-c": "Campus C"
};

export const DEFAULT_CAMPUS_LABELS: Record<CampusKey, string> = {
  main: "Primary Site",
  b: "Campus B",
  c: "Campus C"
};

export const DEFAULT_RECEIVER_LABELS: Record<ReceiverKey, string> = {
  main: "Primary Site Feed",
  "campus-b": "Campus B Feed",
  "campus-c": "Campus C Feed",
  grid: "3-Site Grid"
};

export const CAMPUS_ROLE_BY_KEY: Record<CampusKey, Role> = {
  main: "main",
  b: "campus-b",
  c: "campus-c"
};

export const PLAYBACK_STREAMS = [
  { label: "Program", app: "broadcast", stream: "program" },
  { label: "Backup", app: "broadcast", stream: "backup" },
  { label: "Test", app: "broadcast", stream: "test" }
] as const;

export function roleToCampus(role: Role): CampusKey | null {
  if (role === "main") return "main";
  if (role === "campus-b") return "b";
  if (role === "campus-c") return "c";
  return null;
}

export function receiverToRole(receiver: ReceiverKey): Role | null {
  if (receiver === "main") return "main";
  if (receiver === "campus-b") return "campus-b";
  if (receiver === "campus-c") return "campus-c";
  return null;
}
