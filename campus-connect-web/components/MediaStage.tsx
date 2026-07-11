"use client";

import {
  ConnectionStateToast,
  ControlBar,
  LiveKitRoom,
  ParticipantTile,
  RoomAudioRenderer,
  TrackLoop,
  useConnectionState,
  useParticipants,
  useTracks
} from "@livekit/components-react";
import { Camera, Mic, RefreshCcw, Signal, VideoOff } from "lucide-react";
import { AudioPresets, RoomEvent, Track, VideoPresets, type AudioCaptureOptions, type Participant, type RoomOptions, type VideoCaptureOptions } from "livekit-client";
import { useEffect, useMemo, useState, type CSSProperties } from "react";
import { apiError, clientErrorMessage, readJsonResponse } from "@/lib/client-response";
import { roleLabel } from "@/lib/tenant";
import type { Role, RoomName, TenantSettings } from "@/lib/types";

type TokenResponse = {
  server_url: string;
  participant_token: string;
  can_publish: boolean;
  identity: string;
};

type DeviceSelection = {
  cameraId: string;
  micId: string;
};

type BrowserDevice = {
  deviceId: string;
  kind: MediaDeviceKind;
  label: string;
};

const PLATFORM_VIDEO_FRAMERATE = 30;
const PLATFORM_VIDEO_MAX_BITRATE = 8_000_000;
const CAMPUS_ROLES: Role[] = ["main", "campus-b", "campus-c"];
const CAMERA_STORAGE_KEY = "campus-connect.camera-id";
const MIC_STORAGE_KEY = "campus-connect.mic-id";

const platformVideoCapture: VideoCaptureOptions = {
  frameRate: PLATFORM_VIDEO_FRAMERATE,
  resolution: {
    ...VideoPresets.h1080.resolution,
    frameRate: PLATFORM_VIDEO_FRAMERATE
  }
};

const platformAudioCapture: AudioCaptureOptions = {
  autoGainControl: true,
  channelCount: 1,
  echoCancellation: true,
  noiseSuppression: true,
  sampleRate: 48_000
};

const platformRoomOptions: RoomOptions = {
  adaptiveStream: true,
  dynacast: true,
  audioCaptureDefaults: platformAudioCapture,
  videoCaptureDefaults: platformVideoCapture,
  publishDefaults: {
    audioPreset: AudioPresets.musicHighQuality,
    degradationPreference: "maintain-resolution",
    dtx: false,
    red: true,
    screenShareEncoding: {
      maxBitrate: PLATFORM_VIDEO_MAX_BITRATE,
      maxFramerate: PLATFORM_VIDEO_FRAMERATE,
      priority: "high"
    },
    simulcast: true,
    videoEncoding: {
      ...VideoPresets.h1080.encoding,
      maxBitrate: PLATFORM_VIDEO_MAX_BITRATE,
      maxFramerate: PLATFORM_VIDEO_FRAMERATE,
      priority: "high"
    }
  }
};

const defaultDeviceSelection: DeviceSelection = {
  cameraId: "",
  micId: ""
};

function validateMediaServerUrl(value: string) {
  try {
    const url = new URL(value);
    if (url.protocol !== "wss:" && url.protocol !== "ws:") {
      throw new Error("Real-time media URL must start with wss:// or ws://.");
    }
  } catch {
    throw new Error("Real-time media URL is invalid. Check NEXT_PUBLIC_RTC_URL in the server .env file.");
  }
}

function storedDeviceSelection(): DeviceSelection {
  if (typeof window === "undefined") return defaultDeviceSelection;
  return {
    cameraId: window.localStorage.getItem(CAMERA_STORAGE_KEY) ?? "",
    micId: window.localStorage.getItem(MIC_STORAGE_KEY) ?? ""
  };
}

function deviceCaptureOptions(selection: DeviceSelection) {
  const video: VideoCaptureOptions = {
    ...platformVideoCapture,
    deviceId: selection.cameraId || undefined
  };
  const audio: AudioCaptureOptions = {
    ...platformAudioCapture,
    deviceId: selection.micId || undefined
  };

  return { audio, video };
}

function roleName(role: Role | null, tenant: TenantSettings) {
  return role ? roleLabel(role, tenant) : "All sites";
}

function roleFromIdentity(identity: string): Role | null {
  if (identity.startsWith("main-")) return "main";
  if (identity.startsWith("campus-b-")) return "campus-b";
  if (identity.startsWith("campus-c-")) return "campus-c";
  if (identity.startsWith("operator-")) return "operator";
  if (identity.startsWith("admin-")) return "admin";
  return null;
}

function roleFromMetadata(metadata?: string): Role | null {
  if (!metadata) return null;
  try {
    const parsed = JSON.parse(metadata) as { role?: unknown };
    return typeof parsed.role === "string" && (["admin", "operator", ...CAMPUS_ROLES] as string[]).includes(parsed.role)
      ? (parsed.role as Role)
      : null;
  } catch {
    return null;
  }
}

function roleFromParticipant(participant: Pick<Participant, "identity" | "metadata">): Role | null {
  return roleFromMetadata(participant.metadata) ?? roleFromIdentity(participant.identity);
}

function isCampusRole(role: Role | null) {
  return role ? CAMPUS_ROLES.includes(role) : false;
}

function QualityStrip() {
  const participants = useParticipants();
  const connectionState = useConnectionState();

  return (
    <div className="flex flex-wrap items-center gap-2 border-t border-[#10252f]/10 bg-white px-3 py-2 text-xs text-[#10252f]/70">
      <span className="inline-flex items-center gap-1 font-semibold text-[#10252f]">
        <Signal className="h-3.5 w-3.5 text-[#2f9b91]" aria-hidden="true" />
        {String(connectionState)}
      </span>
      {participants.map((participant) => {
        const quality = String((participant as unknown as { connectionQuality?: string }).connectionQuality ?? "unknown");
        return (
          <span key={participant.identity} className="rounded-md bg-[#f8faf7] px-2 py-1">
            {participant.name || participant.identity}: {quality}
          </span>
        );
      })}
    </div>
  );
}

function PublisherDeviceControls({
  livePublishing,
  hasPublishPermission,
  publishRequested,
  selection,
  onApply,
  tenant
}: {
  livePublishing: boolean;
  hasPublishPermission: boolean;
  publishRequested: boolean;
  selection: DeviceSelection;
  onApply: (selection: DeviceSelection) => void;
  tenant: TenantSettings;
}) {
  const [devices, setDevices] = useState<BrowserDevice[]>([]);
  const [draft, setDraft] = useState<DeviceSelection>(selection);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setDraft(selection);
  }, [selection]);

  async function loadDevices(requestPermission = false) {
    setError(null);
    if (!navigator.mediaDevices?.enumerateDevices) {
      setError("This browser does not expose camera and microphone device selection.");
      return;
    }

    let permissionStream: MediaStream | null = null;
    try {
      if (requestPermission) {
        permissionStream = await navigator.mediaDevices.getUserMedia({ audio: true, video: true });
      }
      const list = await navigator.mediaDevices.enumerateDevices();
      setDevices(
        list.map((device, index) => ({
          deviceId: device.deviceId,
          kind: device.kind,
          label: device.label || `${device.kind === "audioinput" ? "Microphone" : device.kind === "videoinput" ? "Camera" : "Device"} ${index + 1}`
        }))
      );
    } catch (err) {
      setError(clientErrorMessage(err, "Unable to read camera and microphone devices."));
    } finally {
      permissionStream?.getTracks().forEach((track) => track.stop());
    }
  }

  useEffect(() => {
    function handleDeviceChange() {
      loadDevices(false).catch(() => undefined);
    }

    loadDevices(false).catch(() => undefined);
    navigator.mediaDevices?.addEventListener?.("devicechange", handleDeviceChange);
    return () => navigator.mediaDevices?.removeEventListener?.("devicechange", handleDeviceChange);
  }, []);

  function apply() {
    window.localStorage.setItem(CAMERA_STORAGE_KEY, draft.cameraId);
    window.localStorage.setItem(MIC_STORAGE_KEY, draft.micId);
    onApply(draft);
  }

  const cameras = devices.filter((device) => device.kind === "videoinput");
  const microphones = devices.filter((device) => device.kind === "audioinput");
  const status = livePublishing
    ? "Publishing enabled. These devices are used for the live room."
    : hasPublishPermission
      ? "Camera and microphone controls are available below. This page starts muted/off in the current mode."
    : !publishRequested
      ? "Receive-only right now. Device choices are saved and will apply when this site is allowed to publish."
      : "Receive-only for this login. Device choices are saved, but this role cannot publish from this page.";

  return (
    <div className="border-t border-[#10252f]/10 bg-[#f8faf7] px-4 py-3">
      <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
        <p className="text-sm font-bold text-[#10252f]">Camera and microphone setup</p>
        <span className={livePublishing ? "rounded-md bg-[#2f9b91]/15 px-2 py-1 text-xs font-bold text-[#2f9b91]" : "rounded-md bg-[#e8a93e]/15 px-2 py-1 text-xs font-bold text-[#e8a93e]"}>
          {livePublishing ? "Live publishing" : hasPublishPermission ? "Muted by default" : "Saved for publishing"}
        </span>
      </div>
      <div className="grid gap-3 lg:grid-cols-[1fr_1fr_auto_auto]">
        <label className="grid gap-1">
          <span className="flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.12em] text-[#10252f]/60">
            <Camera className="h-4 w-4 text-[#2f9b91]" aria-hidden="true" />
            Camera
          </span>
          <select
            value={draft.cameraId}
            onChange={(event) => setDraft((current) => ({ ...current, cameraId: event.target.value }))}
            className="h-10 rounded-md border border-[#10252f]/15 bg-white px-3 text-sm font-semibold text-[#10252f]"
          >
            <option value="">Browser default camera</option>
            {cameras.map((device) => (
              <option key={device.deviceId} value={device.deviceId}>
                {device.label}
              </option>
            ))}
          </select>
        </label>

        <label className="grid gap-1">
          <span className="flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.12em] text-[#10252f]/60">
            <Mic className="h-4 w-4 text-[#e45f5f]" aria-hidden="true" />
            Microphone
          </span>
          <select
            value={draft.micId}
            onChange={(event) => setDraft((current) => ({ ...current, micId: event.target.value }))}
            className="h-10 rounded-md border border-[#10252f]/15 bg-white px-3 text-sm font-semibold text-[#10252f]"
          >
            <option value="">Browser default microphone</option>
            {microphones.map((device) => (
              <option key={device.deviceId} value={device.deviceId}>
                {device.label}
              </option>
            ))}
          </select>
        </label>

        <button
          type="button"
          onClick={() => loadDevices(true)}
          className="self-end rounded-md border border-[#10252f]/15 px-3 py-2 text-sm font-bold text-[#10252f] hover:border-[#2f9b91]/50"
        >
          Refresh devices
        </button>
        <button
          type="button"
          onClick={apply}
          className="self-end rounded-md bg-[#10252f] px-3 py-2 text-sm font-bold text-white hover:bg-[#173746]"
        >
          Apply camera/mic
        </button>
      </div>
      <p className="mt-2 text-xs text-[#10252f]/55">{status}</p>
      <p className="mt-1 text-xs text-[#10252f]/55">If labels are blank, click Refresh devices and allow browser access for {tenant.platformName}.</p>
      {error ? <p className="mt-2 text-sm text-[#b83f3f]">{error}</p> : null}
    </div>
  );
}

function TrackGrid({ targetRole, compact, tenant }: { targetRole: Role | null; compact?: boolean; tenant: TenantSettings }) {
  const tracks = useTracks(
    [Track.Source.Camera, Track.Source.ScreenShare],
    {
      onlySubscribed: true,
      updateOnlyOn: [
        RoomEvent.ParticipantConnected,
        RoomEvent.ParticipantDisconnected,
        RoomEvent.ParticipantMetadataChanged,
        RoomEvent.TrackMuted,
        RoomEvent.TrackPublished,
        RoomEvent.TrackStreamStateChanged,
        RoomEvent.TrackSubscribed,
        RoomEvent.TrackUnmuted,
        RoomEvent.TrackUnpublished,
        RoomEvent.TrackUnsubscribed
      ]
    }
  );

  const filteredTracks = useMemo(() => {
    return tracks.filter((trackRef) => {
      const role = roleFromParticipant(trackRef.participant);
      if (!isCampusRole(role)) return false;
      if (targetRole && role !== targetRole) return false;
      return trackRef.publication.kind === Track.Kind.Video && !trackRef.publication.isMuted;
    });
  }, [targetRole, tracks]);
  const gridStyle = useMemo<CSSProperties>(
    () => ({
      gridTemplateColumns:
        filteredTracks.length <= 1
          ? "minmax(0, 1fr)"
          : `repeat(${Math.min(filteredTracks.length, 3)}, minmax(0, 1fr))`
    }),
    [filteredTracks.length]
  );

  if (filteredTracks.length === 0) {
    return (
      <div className="flex h-full min-h-[320px] flex-col items-center justify-center gap-3 bg-[#0d1b22] p-6 text-center text-white">
        <VideoOff className="h-10 w-10 text-[#e8a93e]" aria-hidden="true" />
        <div>
          <p className="text-lg font-bold">{roleName(targetRole, tenant)} is not on video yet</p>
          <p className="mt-1 text-sm text-white/70">{tenant.platformName} will show the feed here as soon as it is published.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="grid h-full gap-2 p-2" style={gridStyle}>
      <TrackLoop tracks={filteredTracks}>
        <ParticipantTile className={compact ? "min-h-[180px]" : "min-h-[280px]"} />
      </TrackLoop>
    </div>
  );
}

export function MediaStage({
  room,
  title,
  publish,
  targetRole = null,
  receiverMode = false,
  showControls = true,
  compact = false,
  tenant
}: {
  room: RoomName;
  title: string;
  publish: boolean;
  targetRole?: Role | null;
  receiverMode?: boolean;
  showControls?: boolean;
  compact?: boolean;
  tenant: TenantSettings;
}) {
  const [tokenResponse, setTokenResponse] = useState<TokenResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);
  const [deviceSelection, setDeviceSelection] = useState<DeviceSelection>(defaultDeviceSelection);
  const [deviceApplyKey, setDeviceApplyKey] = useState(0);

  useEffect(() => {
    setDeviceSelection(storedDeviceSelection());
  }, []);

  const captureOptions = useMemo(() => deviceCaptureOptions(deviceSelection), [deviceSelection]);

  function applyDevices(selection: DeviceSelection) {
    setDeviceSelection(selection);
    if (tokenResponse?.can_publish) {
      setDeviceApplyKey((key) => key + 1);
    }
  }

  useEffect(() => {
    let active = true;
    setTokenResponse(null);
    setError(null);

    async function loadToken() {
      try {
        const response = await fetch("/api/media/token", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ room_name: room })
        });
        const json = await readJsonResponse<TokenResponse | { error?: string }>(response);
        if (!response.ok) throw new Error(apiError(json, "Media token failed."));
        validateMediaServerUrl((json as TokenResponse).server_url);
        if (active) setTokenResponse(json as TokenResponse);
      } catch (err) {
        if (active) setError(clientErrorMessage(err, "Unable to connect to real-time media."));
      }
    }

    loadToken();
    return () => {
      active = false;
    };
  }, [room, refreshKey]);

  if (error) {
    return (
      <section className="rounded-lg border border-[#e45f5f]/30 bg-white p-5 text-[#10252f] shadow-sm">
        <h3 className="text-lg font-bold">{title}</h3>
        <p className="mt-2 text-sm text-[#b83f3f]">{error}</p>
        <button
          type="button"
          onClick={() => setRefreshKey((key) => key + 1)}
          className="mt-4 inline-flex items-center gap-2 rounded-md bg-[#10252f] px-3 py-2 text-sm font-semibold text-white"
        >
          <RefreshCcw className="h-4 w-4" aria-hidden="true" />
          Reconnect
        </button>
      </section>
    );
  }

  if (!tokenResponse) {
    return (
      <section className="flex min-h-[360px] items-center justify-center rounded-lg border border-[#10252f]/10 bg-white p-5 text-center shadow-sm">
        <div>
          <p className="text-lg font-bold text-[#10252f]">Loading {tenant.platformName}</p>
          <p className="mt-1 text-sm text-[#10252f]/60">Preparing {title}.</p>
        </div>
      </section>
    );
  }

  const hasPublishPermission = tokenResponse.can_publish;
  const canPublish = publish && hasPublishPermission;

  return (
    <section className={receiverMode ? "receiver-mode h-full bg-black" : "rounded-lg border border-[#10252f]/10 bg-white shadow-sm"}>
      {!receiverMode ? (
        <div className="flex flex-wrap items-center justify-between gap-3 border-b border-[#10252f]/10 px-4 py-3">
          <div>
            <h3 className="text-base font-bold text-[#10252f]">{title}</h3>
            <p className="text-sm text-[#10252f]/60">{room}</p>
          </div>
          <button
            type="button"
            onClick={() => setRefreshKey((key) => key + 1)}
            title={`Reconnect to ${tenant.platformName}`}
            className="inline-flex h-9 items-center gap-2 rounded-md border border-[#10252f]/10 px-3 text-sm font-semibold text-[#10252f] hover:border-[#2f9b91]/50"
          >
            <RefreshCcw className="h-4 w-4" aria-hidden="true" />
            Reconnect
          </button>
        </div>
      ) : null}
      <div className={receiverMode ? "video-stage h-full" : compact ? "video-stage h-[420px]" : "video-stage h-[620px]"}>
        <LiveKitRoom
          key={`${tokenResponse.identity}-${deviceApplyKey}`}
          token={tokenResponse.participant_token}
          serverUrl={tokenResponse.server_url}
          connect
          video={canPublish ? captureOptions.video : false}
          audio={canPublish ? captureOptions.audio : false}
          options={platformRoomOptions}
        >
          <div className="flex h-full flex-col">
            <div className="min-h-0 flex-1">
              <TrackGrid targetRole={targetRole} compact={compact} tenant={tenant} />
            </div>
            <RoomAudioRenderer />
            {showControls && !receiverMode ? <ControlBar /> : null}
            {!receiverMode ? <QualityStrip /> : null}
            {!receiverMode ? (
              <PublisherDeviceControls
                livePublishing={canPublish}
                hasPublishPermission={hasPublishPermission}
                publishRequested={publish}
                selection={deviceSelection}
                onApply={applyDevices}
                tenant={tenant}
              />
            ) : null}
            <ConnectionStateToast />
          </div>
        </LiveKitRoom>
      </div>
    </section>
  );
}
