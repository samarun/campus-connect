"use client";

import { AlertTriangle, Radio, RefreshCcw } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import type Hls from "hls.js";
import type { ErrorData } from "hls.js";
import { PLAYBACK_STREAMS } from "@/lib/constants";
import { modeLabel } from "@/lib/tenant";
import type { TenantSettings } from "@/lib/types";

type StreamStatus = "checking" | "available" | "waiting" | "error";

type QualityLevel = {
  index: number;
  label: string;
};

export function OvenPlayerClient({ baseUrl, relayPublishUrl, tenant }: { baseUrl: string; relayPublishUrl: string; tenant: TenantSettings }) {
  const [stream, setStream] = useState<(typeof PLAYBACK_STREAMS)[number]>(PLAYBACK_STREAMS[0]);
  const [reloadKey, setReloadKey] = useState(0);
  const [streamStatus, setStreamStatus] = useState<StreamStatus>("checking");
  const [streamMessage, setStreamMessage] = useState("Checking stream ingest.");
  const [playerError, setPlayerError] = useState<string | null>(null);
  const [qualityLevels, setQualityLevels] = useState<QualityLevel[]>([]);
  const [selectedQuality, setSelectedQuality] = useState(-1);
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const hlsRef = useRef<Hls | null>(null);
  const programModeName = modeLabel("program", tenant);

  const source = useMemo(() => `${baseUrl.replace(/\/$/, "")}/${stream.app}/${stream.stream}/llhls.m3u8`, [baseUrl, stream]);
  const statusLabel =
    streamStatus === "available" ? "Active" : streamStatus === "checking" ? "Checking" : streamStatus === "waiting" ? "Inactive" : "Check failed";
  const statusClass =
    streamStatus === "available"
      ? "bg-[#2f9b91]/15 text-[#2f9b91]"
      : streamStatus === "checking"
        ? "bg-[#e8a93e]/15 text-[#e8a93e]"
        : streamStatus === "waiting"
          ? "bg-[#dfe7e3] text-[#10252f]/70"
          : "bg-[#e45f5f]/15 text-[#e45f5f]";
  const overlayTitle =
    streamStatus === "checking" ? "Checking stream" : streamStatus === "waiting" ? "Stream inactive" : "Stream check failed";
  const bitrateControlText =
    streamStatus !== "available"
      ? "Bitrates load after the stream is active."
      : qualityLevels.length > 1
        ? "Auto is selected by default. Choose a fixed bitrate only for testing."
        : "Only one bitrate is available from the relay right now.";

  function describeHlsError(data: ErrorData) {
    const detail = data.details ? ` (${data.details})` : "";
    const responseCode = data.response?.code ? ` HTTP ${data.response.code}` : "";
    return `Playback error${responseCode}${detail}.`;
  }

  function formatQualityLabel(level: Hls["levels"][number]) {
    const resolution = level.height ? `${level.height}p` : level.width && level.height ? `${level.width}x${level.height}` : "";
    const bitrate = level.bitrate ? `${Math.round(level.bitrate / 1000).toLocaleString()} kbps` : "";
    const name = level.name && level.name !== resolution ? level.name : "";
    return [name, resolution, bitrate].filter(Boolean).join(" - ") || "Source";
  }

  function changeQuality(value: number) {
    setSelectedQuality(value);
    const hls = hlsRef.current;
    if (!hls) return;
    hls.currentLevel = value;
  }

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
        setStreamMessage(`${stream.label} is not receiving an encoder signal yet. Start the encoder or send SRT/RTMP to the relay.`);
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
    const video = videoRef.current;
    if (!video || streamStatus !== "available") return;

    let cancelled = false;
    setPlayerError(null);
    setQualityLevels([]);
    setSelectedQuality(-1);
    hlsRef.current?.destroy();
    hlsRef.current = null;

    async function attachPlayer() {
      if (!video) return;

      if (video.canPlayType("application/vnd.apple.mpegurl")) {
        video.src = source;
        video.load();
        await video.play().catch(() => {
          if (!cancelled) setPlayerError("Playback is ready. Press play if the browser blocks autoplay.");
        });
        return;
      }

      const { default: Hls } = await import("hls.js");
      if (cancelled) return;

      if (!Hls.isSupported()) {
        setPlayerError("This browser cannot play HLS. Use Chrome, Edge, Safari, or a supported hardware player.");
        return;
      }

      const hls = new Hls({
        lowLatencyMode: true,
        backBufferLength: 30,
        liveSyncDurationCount: 2,
        liveMaxLatencyDurationCount: 8,
        maxLiveSyncPlaybackRate: 1.5
      });
      hlsRef.current = hls;

      hls.on(Hls.Events.ERROR, (_event, data) => {
        if (!data.fatal) return;

        const message = describeHlsError(data);
        setPlayerError(`${message} Retrying playback.`);

        if (data.type === Hls.ErrorTypes.NETWORK_ERROR) {
          hls.startLoad();
          return;
        }

        if (data.type === Hls.ErrorTypes.MEDIA_ERROR) {
          hls.recoverMediaError();
          return;
        }

        hls.destroy();
        hlsRef.current = null;
        setStreamStatus("error");
        setStreamMessage(message);
      });

      hls.on(Hls.Events.MANIFEST_PARSED, () => {
        setQualityLevels(
          hls.levels.map((level, index) => ({
            index,
            label: formatQualityLabel(level)
          }))
        );
        changeQuality(-1);
        video.play().catch(() => setPlayerError("Playback is ready. Press play if the browser blocks autoplay."));
      });

      hls.attachMedia(video);
      hls.loadSource(source);
    }

    attachPlayer();

    return () => {
      cancelled = true;
      hlsRef.current?.destroy();
      hlsRef.current = null;
      video.removeAttribute("src");
      video.load();
    };
  }, [source, streamStatus]);

  useEffect(() => {
    if (streamStatus !== "available") {
      hlsRef.current?.destroy();
      hlsRef.current = null;
      setPlayerError(null);
      setQualityLevels([]);
      setSelectedQuality(-1);
    }
  }, [streamStatus]);

  return (
    <section className="space-y-4">
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
                  ? "rounded-md border border-[#2f9b91] bg-[#10252f] px-3 py-2 text-sm font-semibold text-white shadow-sm ring-2 ring-[#2f9b91]/25"
                  : "rounded-md border border-[#10252f]/10 px-3 py-2 text-sm font-semibold text-[#10252f] hover:border-[#2f9b91]/50 hover:bg-[#f8faf7]"
              }
            >
              <span className="inline-flex items-center gap-2">
                <span>{item.label}</span>
                {item.stream === stream.stream ? <span className="rounded-full bg-white/15 px-1.5 py-0.5 text-[10px] uppercase tracking-[0.1em]">Selected</span> : null}
              </span>
            </button>
          ))}
        </div>
      </div>

      <div className="overflow-hidden rounded-lg border border-[#10252f]/10 bg-[#0d1b22] shadow-sm">
        <div className="flex flex-wrap items-center gap-3 border-b border-white/10 px-4 py-3 text-white">
          <div className="flex min-w-0 items-center gap-2">
            <Radio className="h-5 w-5 shrink-0 text-[#e45f5f]" aria-hidden="true" />
            <span className="font-bold">{stream.label}</span>
            <span className={`rounded-full px-2 py-0.5 text-xs font-bold uppercase tracking-[0.1em] ${statusClass}`}>{statusLabel}</span>
          </div>
          <div className="ml-auto flex flex-wrap items-center justify-end gap-2">
            <label className="inline-flex items-center gap-2 text-xs font-bold uppercase tracking-[0.1em] text-white/70" title={bitrateControlText}>
              Bitrate
              <select
                value={selectedQuality}
                onChange={(event) => changeQuality(Number(event.target.value))}
                disabled={streamStatus !== "available" || qualityLevels.length <= 1}
                className="rounded-md border border-white/10 bg-white/10 px-2 py-1 text-sm normal-case tracking-normal text-white outline-none disabled:cursor-not-allowed disabled:opacity-55"
              >
                <option value={-1} className="text-[#10252f]">
                  Auto bitrate
                </option>
                {qualityLevels.map((level) => (
                  <option key={level.index} value={level.index} className="text-[#10252f]">
                    {level.label}
                  </option>
                ))}
              </select>
            </label>
            <code className="hidden rounded bg-white/10 px-2 py-1 text-xs text-white/80 xl:block">{source}</code>
          </div>
        </div>
        <div className="relative aspect-[16/10] min-h-[360px] w-full bg-black md:min-h-[520px] lg:min-h-[620px]">
          <video
            ref={videoRef}
            className="h-full w-full bg-black object-contain"
            controls
            playsInline
            autoPlay
            crossOrigin="anonymous"
            aria-label={`${stream.label} ${programModeName} stream`}
          />
          {playerError && streamStatus === "available" ? (
            <div className="absolute left-4 top-4 max-w-xl rounded-md border border-[#e8a93e]/35 bg-black/80 px-3 py-2 text-sm font-semibold text-white shadow-lg">
              {playerError}
            </div>
          ) : null}
          {streamStatus !== "available" ? (
            <div className="absolute inset-0 flex items-center justify-center p-6 text-center text-white">
              <div className="max-w-2xl rounded-lg border border-white/10 bg-black/75 p-5 shadow-lg">
                <AlertTriangle className="mx-auto h-10 w-10 text-[#e8a93e]" aria-hidden="true" />
                <h4 className="mt-3 text-lg font-bold">{overlayTitle}</h4>
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
