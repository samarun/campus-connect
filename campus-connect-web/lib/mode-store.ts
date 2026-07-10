import "server-only";

import { mkdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import type { Mode, Role } from "./types";

export type ModeState = {
  mode: Mode;
  updatedAt: string;
  updatedBy: Role | "system";
};

const DATA_DIR = join(process.cwd(), "data");
const MODE_FILE = join(DATA_DIR, "mode.json");

const DEFAULT_STATE: ModeState = {
  mode: "greeting",
  updatedAt: new Date(0).toISOString(),
  updatedBy: "system"
};

export async function getModeState(): Promise<ModeState> {
  try {
    const raw = await readFile(MODE_FILE, "utf8");
    const parsed = JSON.parse(raw) as ModeState | { mode?: "greeting" | "program"; updatedAt?: string; updatedBy?: Role | "system" };
    if (parsed.mode !== "greeting" && parsed.mode !== "program") return DEFAULT_STATE;
    return {
      mode: parsed.mode,
      updatedAt: parsed.updatedAt ?? DEFAULT_STATE.updatedAt,
      updatedBy: parsed.updatedBy ?? DEFAULT_STATE.updatedBy
    };
  } catch {
    return DEFAULT_STATE;
  }
}

export async function setModeState(mode: Mode, updatedBy: Role): Promise<ModeState> {
  const next: ModeState = {
    mode,
    updatedAt: new Date().toISOString(),
    updatedBy
  };
  await mkdir(DATA_DIR, { recursive: true });
  await writeFile(MODE_FILE, JSON.stringify(next, null, 2), "utf8");
  return next;
}
