"use client";

import { Maximize2 } from "lucide-react";
import Link from "next/link";
import { useEffect, useState } from "react";
import { ROOMS, receiverToRole } from "@/lib/constants";
import { receiverLabel } from "@/lib/tenant";
import type { ReceiverKey, TenantSettings } from "@/lib/types";
import { MediaStage } from "./MediaStage";

export function ReceiverView({ receiver, tenant }: { receiver: ReceiverKey; tenant: TenantSettings }) {
  const targetRole = receiverToRole(receiver);
  const currentLabel = receiverLabel(receiver, tenant);
  const [isFullscreen, setIsFullscreen] = useState(false);

  useEffect(() => {
    function syncFullscreenState() {
      setIsFullscreen(Boolean(document.fullscreenElement));
    }

    syncFullscreenState();
    document.addEventListener("fullscreenchange", syncFullscreenState);
    return () => document.removeEventListener("fullscreenchange", syncFullscreenState);
  }, []);

  async function fullscreen() {
    if (!document.fullscreenElement) {
      await document.documentElement.requestFullscreen().catch(() => undefined);
    }
  }

  return (
    <main className="h-screen overflow-hidden bg-black text-white">
      {!isFullscreen ? (
        <div className="absolute left-3 top-3 z-10 flex max-w-[calc(100vw-1.5rem)] flex-wrap items-center gap-2 rounded-md bg-black/55 p-2 backdrop-blur">
          <span className="px-2 text-sm font-bold">{currentLabel}</span>
          {[
            ["/receiver/main", tenant.primarySiteName],
            ["/receiver/campus-b", tenant.remoteSiteAName],
            ["/receiver/campus-c", tenant.remoteSiteBName],
            ["/receiver/main?view=grid", "Grid"]
          ].map(([href, label]) => (
            <Link key={href} href={href} className="rounded-md bg-white/10 px-2 py-1 text-xs font-semibold hover:bg-white/20">
              {label}
            </Link>
          ))}
          <button
            type="button"
            onClick={fullscreen}
            title={`Full screen ${tenant.platformName} receiver`}
            className="inline-flex items-center gap-1 rounded-md bg-white/10 px-2 py-1 text-xs font-semibold hover:bg-white/20"
          >
            <Maximize2 className="h-3.5 w-3.5" aria-hidden="true" />
            Full screen
          </button>
        </div>
      ) : null}
      <MediaStage room={ROOMS.greeting} title={currentLabel} publish={false} targetRole={targetRole} receiverMode showControls={false} tenant={tenant} />
    </main>
  );
}
