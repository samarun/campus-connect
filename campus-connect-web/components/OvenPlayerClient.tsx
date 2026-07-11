"use client";

import Script from "next/script";
import { AlertTriangle, Radio, RefreshCcw } from "lucide-react";
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

type StreamStatus = "checking" | "available" | "waiting" | "error";

export function OvenPlayerClient({ baseUrl, relayPublishUrl, tenant }: { baseUrl: string; relayPublishUrl: string; tenant: TenantSettings }) {
  const [ready, setReady] = useState(false);
  const [stream, setStream] = useState<(typeof PLAYBACK_STREAMS)[number]>(PLAYBACK_STREAMS[0]);
  const [reloadKey, setReloadKey] = useState(0);
  const [streamStatus, setStreamStatus] = useState<StreamStatus>("checking");
  const [streamMessage, setStreamMessage] = useState("Checking stream ingest.");
  const playerRef = useRef<{ remove?: () => void } | null>(null);
  const programModeName = modeLabel("program", tenant);

  const source = useMemo(() => `${baseUrl.replace(/\/$/, "")}/${stream.app}/${stream.stream}/llhls.m3u8`, [baseUrl, stream]);

  useEffect(() => {
    let active = true;
    setStreamStatus("checking");
    setStreamMessage("Checking stream ingest.");

    async function checkStream() {
      try {
        const response = await fetch(source, { cache: "no-store" });
        const body = await response.text().catch(() => "");
        if (!active) return;

        if (response.ok && body.includes("#EXTM3U")) {
          setStreamStatus("available");
          setStreamMessage("Stream is available.");
          return;
        }

        setStreamStatus("waiting");
        setStreamMessage(`${stream.label} stream is not active yet. Start the encoder or send SRT to the relay.`);
      } catch (error) {
        if (!active) return;
        setStreamStatus("error");
        setStreamMessage(error instanceof Error ? error.message : "Unable to check stream ingest.");
      }
    }

    checkStream();
    const timer = window.setInterval(checkStream, 5000);
    return () => {
      active = false;
      window.clearInterval(timer);
    };
  }, [reloadKey, source, stream.label]);

  useEffect(() => {
    if (!ready || !window.OvenPlayer || streamStatus !== "available") return;
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
  }, [ready, source, stream.label, streamStatus]);

  useEffect(() => {
    if (streamStatus !== "available") {
      playerRef.current?.remove?.();
      playerRef.current = null;
    }
  }, [streamStatus]);

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
              onClick={() => {
                setStream(item);
                setReloadKey((key) => key + 1);
              }}
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
        <div className="relative aspect-video w-full bg-black">
          <div id="campus-connect-player" className="h-full w-full" />
          {streamStatus !== "available" ? (
            <div className="absolute inset-0 flex items-center justify-center p-6 text-center text-white">
              <div className="max-w-2xl rounded-lg border border-white/10 bg-black/75 p-5 shadow-lg">
                <AlertTriangle className="mx-auto h-10 w-10 text-[#e8a93e]" aria-hidden="true" />
                <h4 className="mt-3 text-lg font-bold">
                  {streamStatus === "checking" ? "Checking stream" : streamStatus === "waiting" ? "Waiting for encoder" : "Stream check failed"}
                </h4>
                <p className="mt-2 text-sm text-white/75">{streamMessage}</p>
                <div className="mt-4 grid gap-2 text-left text-xs text-white/70">
                  <div>
                    Playback URL: <code className="rounded bg-white/10 px-1 py-0.5 text-white">{source}</code>
                  </div>
                  <div>
                    SRT publish URL: <code className="rounded bg-white/10 px-1 py-0.5 text-white">{relayPublishUrl}</code>
                  </div>
                </div>
                <button
                  type="button"
                  onClick={() => setReloadKey((key) => key + 1)}
                  className="mt-4 inline-flex items-center gap-2 rounded-md bg-white px-3 py-2 text-sm font-bold text-[#10252f]"
                >
                  <RefreshCcw className="h-4 w-4" aria-hidden="true" />
                  Check again
                </button>
              </div>
            </div>
          ) : null}
        </div>
      </div>
    </section>
  );
}
