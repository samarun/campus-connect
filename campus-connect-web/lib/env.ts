const REQUIRED_ENV = [
  "SESSION_SECRET",
  "RTC_API_KEY",
  "RTC_API_SECRET",
  "RTC_API_URL",
  "NEXT_PUBLIC_RTC_URL"
] as const;

export function getEnv(name: string): string {
  const value = process.env[name];
  if (!value || value.startsWith("CHANGE_ME")) {
    throw new Error(`Missing required environment variable: ${name}`);
  }
  return value;
}

export function validateCoreEnv() {
  for (const name of REQUIRED_ENV) {
    getEnv(name);
  }
}

export function publicRtcUrl() {
  return getEnv("NEXT_PUBLIC_RTC_URL");
}

export function publicPlaybackBaseUrl() {
  return process.env.NEXT_PUBLIC_STREAM_LLHLS_BASE_URL || "https://stream.example.com";
}

export function relayPublishUrl() {
  return process.env.RELAY_SRT_PUBLISH_URL || "srt://stream.example.com:9999?streamid=default/broadcast/program";
}
