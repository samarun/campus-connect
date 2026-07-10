"use client";

import { Radio, Video } from "lucide-react";
import { useEffect, useState } from "react";
import { modeLabel } from "@/lib/tenant";
import type { Mode, TenantSettings } from "@/lib/types";

type ModeState = {
  mode: Mode;
  updatedAt: string;
  updatedBy: string;
};

export function useModeState() {
  const [state, setState] = useState<ModeState | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;

    async function load() {
      try {
        const response = await fetch("/api/mode", { cache: "no-store" });
        if (!response.ok) throw new Error("Mode unavailable");
        const json = (await response.json()) as ModeState;
        if (active) {
          setState(json);
          setError(null);
        }
      } catch (err) {
        if (active) setError(err instanceof Error ? err.message : "Mode unavailable");
      }
    }

    load();
    const timer = window.setInterval(load, 5000);
    return () => {
      active = false;
      window.clearInterval(timer);
    };
  }, []);

  return { state, setState, error };
}

export function ModeBadge({ tenant }: { tenant: TenantSettings }) {
  const { state, error } = useModeState();
  const mode = state?.mode ?? "greeting";
  const Icon = mode === "greeting" ? Video : Radio;

  return (
    <div className="inline-flex items-center gap-2 rounded-md border border-[#10252f]/10 bg-white px-3 py-2 text-sm font-semibold text-[#10252f] shadow-sm">
      <Icon className={mode === "greeting" ? "h-4 w-4 text-[#2f9b91]" : "h-4 w-4 text-[#e45f5f]"} aria-hidden="true" />
      {error ? "Mode status unavailable" : modeLabel(mode, tenant)}
    </div>
  );
}
