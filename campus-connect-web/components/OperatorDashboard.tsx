"use client";

import { LayoutGrid, Radio, Video } from "lucide-react";
import Link from "next/link";
import { useState } from "react";
import { apiError, clientErrorMessage, readJsonResponse } from "@/lib/client-response";
import { ROOMS } from "@/lib/constants";
import { receiverLinks as buildReceiverLinks } from "@/lib/site-routes";
import { campusList, modeLabel, receiverLabel } from "@/lib/tenant";
import type { Mode, TenantSettings } from "@/lib/types";
import { MediaStage } from "./MediaStage";
import { ModeBadge, type ModeState, useModeState } from "./ModeBadge";
import { ParticipantStatus } from "./ParticipantStatus";
import { TenantSettingsPanel } from "./TenantSettingsPanel";

export function OperatorDashboard({ tenant }: { tenant: TenantSettings }) {
  const { state, setState, error } = useModeState();
  const [busy, setBusy] = useState<Mode | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const mode = state?.mode ?? "greeting";
  const siteNames = campusList(tenant);
  const greetingModeName = modeLabel("greeting", tenant);
  const programModeName = modeLabel("program", tenant);
  const modeButtonClass = (targetMode: Mode) => {
    const active = mode === targetMode;
    const isGreeting = targetMode === "greeting";

    if (active) {
      return isGreeting
        ? "flex min-h-28 items-center gap-3 rounded-lg border border-[#2f9b91] bg-[#f2fbf8] p-4 text-left shadow-sm ring-2 ring-[#2f9b91]/30 transition disabled:cursor-default"
        : "flex min-h-28 items-center gap-3 rounded-lg border border-[#e45f5f] bg-[#fff6f2] p-4 text-left shadow-sm ring-2 ring-[#e45f5f]/30 transition disabled:cursor-default";
    }

    return isGreeting
      ? "flex min-h-28 items-center gap-3 rounded-lg border border-[#10252f]/10 bg-white p-4 text-left transition hover:border-[#2f9b91]/60 hover:bg-[#f8faf7] disabled:cursor-wait disabled:opacity-70"
      : "flex min-h-28 items-center gap-3 rounded-lg border border-[#10252f]/10 bg-white p-4 text-left transition hover:border-[#e45f5f]/60 hover:bg-[#f8faf7] disabled:cursor-wait disabled:opacity-70";
  };
  const modeStateLabel = (targetMode: Mode) => {
    if (busy === targetMode) return "Switching";
    return mode === targetMode ? "Active" : "Inactive";
  };
  const modePillClass = (targetMode: Mode) => {
    if (busy === targetMode) return "bg-[#e8a93e]/20 text-[#e8a93e]";
    if (mode !== targetMode) return "bg-[#dfe7e3] text-[#10252f]/70";
    return targetMode === "greeting" ? "bg-[#2f9b91]/15 text-[#2f9b91]" : "bg-[#e45f5f]/15 text-[#e45f5f]";
  };

  async function switchMode(nextMode: Mode) {
    setBusy(nextMode);
    setActionError(null);

    try {
      const response = await fetch("/api/mode", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ mode: nextMode })
      });
      const json = await readJsonResponse<ModeState | { error?: string }>(response);
      if (!response.ok) throw new Error(apiError(json, "Unable to switch mode."));
      setState(json as ModeState);
    } catch (err) {
      setActionError(clientErrorMessage(err, "Unable to switch mode."));
    } finally {
      setBusy(null);
    }
  }

  return (
    <div className="space-y-5">
      <div className="grid gap-4 lg:grid-cols-[1fr_360px]">
        <section className="rounded-lg border border-[#10252f]/10 bg-white p-4 shadow-sm">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div>
              <h3 className="text-lg font-bold text-[#10252f]">{tenant.platformName} operator dashboard</h3>
              <p className="mt-1 text-sm text-[#10252f]/60">Switch site mode and watch all live site feeds.</p>
            </div>
            <ModeBadge tenant={tenant} />
          </div>

          <div className="mt-4 grid gap-3 sm:grid-cols-2">
            <button
              type="button"
              onClick={() => switchMode("greeting")}
              aria-pressed={mode === "greeting"}
              disabled={busy !== null || mode === "greeting"}
              className={modeButtonClass("greeting")}
            >
              <Video className="h-8 w-8 shrink-0 text-[#2f9b91]" aria-hidden="true" />
              <span className="min-w-0 flex-1">
                <span className="flex flex-wrap items-center gap-2">
                  <span className="block text-base font-bold text-[#10252f]">{greetingModeName}</span>
                  <span className={`rounded-full px-2 py-0.5 text-xs font-bold uppercase tracking-[0.1em] ${modePillClass("greeting")}`}>
                    {modeStateLabel("greeting")}
                  </span>
                </span>
                <span className="mt-1 block text-sm text-[#10252f]/65">{siteNames.join(", ")} publish audio/video.</span>
                {mode !== "greeting" ? <span className="mt-2 block text-xs font-bold uppercase tracking-[0.12em] text-[#2f9b91]">Click to activate</span> : null}
              </span>
            </button>
            <button
              type="button"
              onClick={() => switchMode("program")}
              aria-pressed={mode === "program"}
              disabled={busy !== null || mode === "program"}
              className={modeButtonClass("program")}
            >
              <Radio className="h-8 w-8 shrink-0 text-[#e45f5f]" aria-hidden="true" />
              <span className="min-w-0 flex-1">
                <span className="flex flex-wrap items-center gap-2">
                  <span className="block text-base font-bold text-[#10252f]">{programModeName}</span>
                  <span className={`rounded-full px-2 py-0.5 text-xs font-bold uppercase tracking-[0.1em] ${modePillClass("program")}`}>
                    {modeStateLabel("program")}
                  </span>
                </span>
                <span className="mt-1 block text-sm text-[#10252f]/65">{tenant.primarySiteName} publishes the program feed; remote sites start muted.</span>
                {mode !== "program" ? <span className="mt-2 block text-xs font-bold uppercase tracking-[0.12em] text-[#e45f5f]">Click to activate</span> : null}
              </span>
            </button>
          </div>
          {error || actionError ? <p className="mt-3 text-sm text-[#b83f3f]">{actionError ?? error}</p> : null}
        </section>

        <section className="rounded-lg border border-[#10252f]/10 bg-white p-4 shadow-sm">
          <div className="mb-3 flex items-center gap-2">
            <LayoutGrid className="h-5 w-5 text-[#e8a93e]" aria-hidden="true" />
            <h3 className="text-base font-bold text-[#10252f]">Receiver shortcuts</h3>
          </div>
          <div className="grid gap-2">
            {buildReceiverLinks(tenant).map(({ receiver, href }) => (
              <Link key={href} href={href} className="rounded-md border border-[#10252f]/10 px-3 py-2 text-sm font-semibold text-[#10252f] hover:border-[#2f9b91]/50">
                {receiver === "main"
                  ? `${receiverLabel(receiver, tenant)} return`
                  : receiver === "grid"
                    ? receiverLabel(receiver, tenant)
                    : `${receiverLabel(receiver, tenant)} full screen`}
              </Link>
            ))}
          </div>
        </section>
      </div>

      <TenantSettingsPanel tenant={tenant} />
      <ParticipantStatus room={mode === "greeting" ? ROOMS.greeting : ROOMS.program} tenant={tenant} />
      <MediaStage room={mode === "greeting" ? ROOMS.greeting : ROOMS.program} title={`${tenant.platformName} live site grid`} publish={false} tenant={tenant} />
    </div>
  );
}
