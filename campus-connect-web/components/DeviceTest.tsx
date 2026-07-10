"use client";

import { Mic, Play, RefreshCcw, Video } from "lucide-react";
import { useEffect, useRef, useState } from "react";

type MediaDevice = {
  deviceId: string;
  kind: MediaDeviceKind;
  label: string;
};

export function DeviceTest() {
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const streamRef = useRef<MediaStream | null>(null);
  const [devices, setDevices] = useState<MediaDevice[]>([]);
  const [cameraId, setCameraId] = useState("");
  const [micId, setMicId] = useState("");
  const [speakerId, setSpeakerId] = useState("");
  const [level, setLevel] = useState(0);
  const [error, setError] = useState<string | null>(null);

  async function stopStream() {
    streamRef.current?.getTracks().forEach((track) => track.stop());
    streamRef.current = null;
  }

  async function loadDevices() {
    const list = await navigator.mediaDevices.enumerateDevices();
    setDevices(
      list.map((device, index) => ({
        deviceId: device.deviceId,
        kind: device.kind,
        label: device.label || `${device.kind.replace("input", "")} ${index + 1}`
      }))
    );
  }

  async function startPreview() {
    setError(null);
    await stopStream();
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        video: cameraId ? { deviceId: { exact: cameraId } } : true,
        audio: micId ? { deviceId: { exact: micId } } : true
      });
      streamRef.current = stream;
      if (videoRef.current) videoRef.current.srcObject = stream;
      await loadDevices();

      const audioContext = new AudioContext();
      const analyser = audioContext.createAnalyser();
      const source = audioContext.createMediaStreamSource(stream);
      const data = new Uint8Array(analyser.frequencyBinCount);
      source.connect(analyser);

      function tick() {
        if (!streamRef.current) {
          audioContext.close().catch(() => undefined);
          return;
        }
        analyser.getByteFrequencyData(data);
        const peak = data.reduce((max, value) => Math.max(max, value), 0);
        setLevel(Math.round((peak / 255) * 100));
        window.requestAnimationFrame(tick);
      }
      tick();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unable to open camera or microphone.");
    }
  }

  async function playSpeakerTone() {
    const audioContext = new AudioContext();
    const oscillator = audioContext.createOscillator();
    const gain = audioContext.createGain();
    const destination = audioContext.createMediaStreamDestination();
    const audio = new Audio();
    oscillator.frequency.value = 880;
    gain.gain.value = 0.08;
    oscillator.connect(gain);
    gain.connect(destination);
    audio.srcObject = destination.stream;
    if (speakerId && "setSinkId" in audio) {
      await (audio as HTMLAudioElement & { setSinkId: (sinkId: string) => Promise<void> }).setSinkId(speakerId);
    }
    await audio.play();
    oscillator.start();
    window.setTimeout(() => {
      oscillator.stop();
      audioContext.close().catch(() => undefined);
      audio.pause();
    }, 600);
  }

  useEffect(() => {
    loadDevices().catch(() => undefined);
    startPreview().catch(() => undefined);
    return () => {
      stopStream().catch(() => undefined);
    };
    // The first preview intentionally starts with browser defaults.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const cameras = devices.filter((device) => device.kind === "videoinput");
  const microphones = devices.filter((device) => device.kind === "audioinput");
  const speakers = devices.filter((device) => device.kind === "audiooutput");

  return (
    <div className="grid gap-5 lg:grid-cols-[1fr_360px]">
      <section className="overflow-hidden rounded-lg border border-[#10252f]/10 bg-white shadow-sm">
        <div className="border-b border-[#10252f]/10 px-4 py-3">
          <h3 className="text-lg font-bold text-[#10252f]">Camera preview</h3>
        </div>
        <video ref={videoRef} autoPlay playsInline muted className="aspect-video w-full bg-[#0d1b22] object-cover" />
        <div className="px-4 py-3">
          <div className="h-3 overflow-hidden rounded-full bg-[#dfe7e3]">
            <div className="h-full bg-[#2f9b91] transition-all" style={{ width: `${level}%` }} />
          </div>
          <p className="mt-2 text-sm text-[#10252f]/60">Microphone level: {level}%</p>
        </div>
      </section>

      <section className="rounded-lg border border-[#10252f]/10 bg-white p-4 shadow-sm">
        <h3 className="text-lg font-bold text-[#10252f]">Device check</h3>
        <div className="mt-4 space-y-3">
          <label className="block">
            <span className="mb-1 flex items-center gap-2 text-sm font-semibold text-[#10252f]">
              <Video className="h-4 w-4 text-[#2f9b91]" aria-hidden="true" />
              Camera
            </span>
            <select value={cameraId} onChange={(event) => setCameraId(event.target.value)} className="w-full rounded-md border border-[#10252f]/15 bg-white px-3 py-2 text-sm">
              <option value="">Browser default camera</option>
              {cameras.map((device) => (
                <option key={device.deviceId} value={device.deviceId}>
                  {device.label}
                </option>
              ))}
            </select>
          </label>

          <label className="block">
            <span className="mb-1 flex items-center gap-2 text-sm font-semibold text-[#10252f]">
              <Mic className="h-4 w-4 text-[#e45f5f]" aria-hidden="true" />
              Microphone
            </span>
            <select value={micId} onChange={(event) => setMicId(event.target.value)} className="w-full rounded-md border border-[#10252f]/15 bg-white px-3 py-2 text-sm">
              <option value="">Browser default microphone</option>
              {microphones.map((device) => (
                <option key={device.deviceId} value={device.deviceId}>
                  {device.label}
                </option>
              ))}
            </select>
          </label>

          <label className="block">
            <span className="mb-1 text-sm font-semibold text-[#10252f]">Speaker</span>
            <select value={speakerId} onChange={(event) => setSpeakerId(event.target.value)} className="w-full rounded-md border border-[#10252f]/15 bg-white px-3 py-2 text-sm">
              <option value="">Browser default speaker</option>
              {speakers.map((device) => (
                <option key={device.deviceId} value={device.deviceId}>
                  {device.label}
                </option>
              ))}
            </select>
          </label>

          <button type="button" onClick={startPreview} className="inline-flex w-full items-center justify-center gap-2 rounded-md bg-[#10252f] px-3 py-2 text-sm font-bold text-white">
            <RefreshCcw className="h-4 w-4" aria-hidden="true" />
            Apply devices
          </button>
          <button type="button" onClick={playSpeakerTone} className="inline-flex w-full items-center justify-center gap-2 rounded-md border border-[#10252f]/15 px-3 py-2 text-sm font-bold text-[#10252f]">
            <Play className="h-4 w-4" aria-hidden="true" />
            Play speaker test
          </button>
        </div>
        {speakerId ? (
          <p className="mt-3 text-xs text-[#10252f]/55">Speaker selection depends on browser support for audio output devices.</p>
        ) : null}
        {error ? <p className="mt-3 text-sm text-[#b83f3f]">{error}</p> : null}
      </section>
    </div>
  );
}
