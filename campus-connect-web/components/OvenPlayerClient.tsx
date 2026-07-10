"use client";

import Script from "next/script";
import { Radio, RefreshCcw } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { PLAYBACK_STREAMS } from "@/lib/constants";
import { modeLabel } from "@/lib/tenant";
import type { TenantSettings } from "@/lib/types";

declare global {
  interface Window {
    OvenPlayer?: {
      create: (elementId: string, options: Record<string, unknown>) => { remove?: () => void };
    };
  }
}

export function OvenPlayerClient({ baseUrl, tenant }: { baseUrl: string; tenant: TenantSettings }) {
  const [ready, setReady] = useState(false);
  const [stream, setStream] = useState<(typeof PLAYBACK_STREAMS)[number]>(PLAYBACK_STREAMS[0]);
  const [reloadKey, setReloadKey] = useState(0);
  const playerRef = useRef<{ remove?: () => void } | null>(null);
  const programModeName = modeLabel("program", tenant);

  const source = useMemo(() => `${baseUrl.replace(/\/$/, "")}/${stream.app}/${stream.stream}/llhls.m3u8`, [baseUrl, stream]);

  useEffect(() => {
    if (!ready || !window.OvenPlayer) return;
    playerRef.current?.remove?.();
    playerRef.current = window.OvenPlayer.create("campus-connect-player", {
      autoStart: true,
      autoFallback: true,
      mute: false,
      controls: true,
      hlsConfig: {
        lowLatencyMode: true,
        liveSyncDuration: 1.5,
        liveMaxLatencyDuration: 6,
        maxLiveSyncPlaybackRate: 1.5
      },
      sources: [
        {
          label: stream.label,
          type: "llhls",
          file: source,
          lowLatency: true
        }
      ]
    });

    return () => {
      playerRef.current?.remove?.();
      playerRef.current = null;
    };
  }, [ready, reloadKey, source, stream.label]);

  return (
    <section className="space-y-4">
      <Script src="/vendor/ovenplayer/ovenplayer.js" strategy="afterInteractive" onLoad={() => setReady(true)} />
      <div className="rounded-lg border border-[#10252f]/10 bg-white p-4 shadow-sm">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <h3 className="text-lg font-bold text-[#10252f]">{tenant.platformName} {programModeName} player</h3>
            <p className="mt-1 text-sm text-[#10252f]/60">Branded playback for low-latency {programModeName.toLowerCase()} streams.</p>
          </div>
          <button
            type="button"
            onClick={() => setReloadKey((key) => key + 1)}
            className="inline-flex items-center gap-2 rounded-md border border-[#10252f]/10 px-3 py-2 text-sm font-semibold hover:border-[#e45f5f]/50"
          >
            <RefreshCcw className="h-4 w-4" aria-hidden="true" />
            Reload stream
          </button>
        </div>
        <div className="mt-4 flex flex-wrap gap-2">
          {PLAYBACK_STREAMS.map((item) => (
            <button
              type="button"
              key={item.stream}
              onClick={() => setStream(item)}
              className={
                item.stream === stream.stream
                  ? "rounded-md bg-[#10252f] px-3 py-2 text-sm font-semibold text-white"
                  : "rounded-md border border-[#10252f]/10 px-3 py-2 text-sm font-semibold text-[#10252f] hover:border-[#2f9b91]/50"
              }
            >
              {item.label}
            </button>
          ))}
        </div>
      </div>

      <div className="overflow-hidden rounded-lg border border-[#10252f]/10 bg-[#0d1b22] shadow-sm">
        <div className="flex items-center gap-2 border-b border-white/10 px-4 py-3 text-white">
          <Radio className="h-5 w-5 text-[#e45f5f]" aria-hidden="true" />
          <span className="font-bold">{stream.label}</span>
          <code className="ml-auto hidden rounded bg-white/10 px-2 py-1 text-xs text-white/80 sm:block">{source}</code>
        </div>
        <div id="campus-connect-player" className="aspect-video w-full bg-black" />
      </div>
    </section>
  );
}
