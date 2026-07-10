import { NextResponse } from "next/server";
import { getModeState } from "@/lib/mode-store";

export const runtime = "nodejs";

const startedAt = Date.now();

function metricLine(name: string, value: number, labels?: Record<string, string>) {
  const labelText = labels
    ? `{${Object.entries(labels)
        .map(([key, item]) => `${key}="${item.replaceAll("\\", "\\\\").replaceAll('"', '\\"')}"`)
        .join(",")}}`
    : "";
  return `${name}${labelText} ${value}`;
}

export async function GET() {
  const modeState = await getModeState();
  const uptimeSeconds = Math.floor((Date.now() - startedAt) / 1000);
  const updatedAtSeconds = Math.floor(Date.parse(modeState.updatedAt) / 1000);

  const body = [
    "# HELP campus_connect_health Web application health status.",
    "# TYPE campus_connect_health gauge",
    metricLine("campus_connect_health", 1),
    "# HELP campus_connect_uptime_seconds Web application uptime in seconds.",
    "# TYPE campus_connect_uptime_seconds gauge",
    metricLine("campus_connect_uptime_seconds", uptimeSeconds),
    "# HELP campus_connect_mode Current operator-selected site mode.",
    "# TYPE campus_connect_mode gauge",
    metricLine("campus_connect_mode", modeState.mode === "greeting" ? 1 : 0, { mode: "greeting" }),
    metricLine("campus_connect_mode", modeState.mode === "program" ? 1 : 0, { mode: "program" }),
    "# HELP campus_connect_mode_updated_at_seconds Last mode update timestamp.",
    "# TYPE campus_connect_mode_updated_at_seconds gauge",
    metricLine("campus_connect_mode_updated_at_seconds", Number.isFinite(updatedAtSeconds) ? updatedAtSeconds : 0, {
      updated_by: modeState.updatedBy
    }),
    ""
  ].join("\n");

  return new NextResponse(body, {
    headers: {
      "Content-Type": "text/plain; version=0.0.4; charset=utf-8",
      "Cache-Control": "no-store"
    }
  });
}
