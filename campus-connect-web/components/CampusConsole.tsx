"use client";

import { ExternalLink, LayoutGrid, MonitorUp, Radio, Video } from "lucide-react";
import Link from "next/link";
import { ROOMS } from "@/lib/constants";
import { receiverHref, receiverLinks as buildReceiverLinks } from "@/lib/site-routes";
import { campusLabel, modeLabel, receiverLabel } from "@/lib/tenant";
import type { CampusKey, Role, TenantSettings } from "@/lib/types";
import { MediaStage } from "./MediaStage";
import { ModeBadge, useModeState } from "./ModeBadge";
import { ParticipantStatus } from "./ParticipantStatus";

export function CampusConsole({ campus, role, tenant, relayPublishUrl }: { campus: CampusKey; role: Role; tenant: TenantSettings; relayPublishUrl: string }) {
  const { state } = useModeState();
  const mode = state?.mode ?? "greeting";
  const isMain = role === "main";
  const siteName = campusLabel(campus, tenant);
  const room = mode === "greeting" ? ROOMS.greeting : ROOMS.program;
  const publish = mode === "greeting" || (mode === "program" && isMain);
  const greetingModeName = modeLabel("greeting", tenant);
  const programModeName = modeLabel("program", tenant);
  const receiverLinks = buildReceiverLinks(tenant).map(({ receiver, href }) => ({
    href,
    label: receiverLabel(receiver, tenant),
    icon: receiver === "grid" ? LayoutGrid : Video
  }));
  const title =
    mode === "greeting"
      ? `${siteName} ${greetingModeName}`
      : isMain
        ? `${siteName} ${programModeName} Feed`
        : `${siteName} ${programModeName} Receive`;

  return (
    <div className="space-y-5">
      <section className="rounded-lg border border-[#10252f]/10 bg-white p-4 shadow-sm">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <h3 className="text-lg font-bold text-[#10252f]">{siteName} console</h3>
            <p className="mt-1 text-sm text-[#10252f]/60">
              {mode === "greeting"
                ? "Camera and microphone are available for near real-time site greeting."
                : isMain
                  ? "Publish the program feed here for return monitoring. Send the final program to the streaming relay over SRT for remote playback."
                  : `${programModeName} starts this site microphone and camera muted by default. Use the player controls only when the operator asks for a live return.`}
            </p>
          </div>
          <ModeBadge tenant={tenant} />
        </div>

        <div className="mt-4 grid gap-3 sm:grid-cols-3">
          <Link href="/test" className="flex items-center gap-3 rounded-md border border-[#10252f]/10 px-3 py-3 text-sm font-semibold hover:border-[#2f9b91]/50">
            <MonitorUp className="h-5 w-5 text-[#2f9b91]" aria-hidden="true" />
            Camera, mic, speaker test
          </Link>
          <Link href={receiverHref("main", tenant)} className="flex items-center gap-3 rounded-md border border-[#10252f]/10 px-3 py-3 text-sm font-semibold hover:border-[#e8a93e]/60">
            <Video className="h-5 w-5 text-[#e8a93e]" aria-hidden="true" />
            {tenant.primarySiteName} return feed
          </Link>
          <Link href="/player/program" className="flex items-center gap-3 rounded-md border border-[#10252f]/10 px-3 py-3 text-sm font-semibold hover:border-[#e45f5f]/50">
            <Radio className="h-5 w-5 text-[#e45f5f]" aria-hidden="true" />
            {programModeName} player
          </Link>
        </div>

        {isMain ? (
          <div className="mt-4 rounded-md border border-[#e8a93e]/30 bg-[#fff8e9] p-3 text-sm text-[#10252f]">
            SRT publish URL: <code className="font-semibold">{relayPublishUrl}</code>
          </div>
        ) : null}
      </section>

      <section className="rounded-lg border border-[#10252f]/10 bg-white p-4 shadow-sm">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <h3 className="text-base font-bold text-[#10252f]">External display output</h3>
            <p className="mt-1 text-sm text-[#10252f]/60">Open one feed or the grid on the display connected to this site video mixer.</p>
          </div>
          <span className="rounded-md bg-[#f8faf7] px-2 py-1 text-xs font-bold uppercase tracking-[0.12em] text-[#10252f]/60">Receiver pages</span>
        </div>
        <div className="mt-4 grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
          {receiverLinks.map((item) => {
            const Icon = item.icon;
            return (
              <Link
                key={item.href}
                href={item.href}
                target="_blank"
                rel="noreferrer"
                className="flex items-center gap-3 rounded-md border border-[#10252f]/10 px-3 py-3 text-sm font-semibold hover:border-[#2f9b91]/50"
              >
                <Icon className="h-5 w-5 text-[#2f9b91]" aria-hidden="true" />
                <span>
                  <span className="block text-[#10252f]">{item.label}</span>
                  <span className="block text-xs text-[#10252f]/55">Open for fullscreen output</span>
                </span>
              </Link>
            );
          })}
        </div>
      </section>

      <ParticipantStatus room={room} tenant={tenant} />
      <MediaStage room={room} title={title} publish={publish} tenant={tenant} />

      {mode === "program" && !isMain ? (
        <Link
          href="/player/program"
          className="inline-flex items-center gap-2 rounded-md bg-[#10252f] px-4 py-3 text-sm font-bold text-white"
        >
          Open {programModeName} player
          <ExternalLink className="h-4 w-4" aria-hidden="true" />
        </Link>
      ) : null}
    </div>
  );
}
