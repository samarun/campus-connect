import "server-only";

import { AccessToken, RoomServiceClient } from "livekit-server-sdk";
import { randomUUID } from "node:crypto";
import { ROOMS } from "./constants";
import { getEnv, publicRtcUrl } from "./env";
import { getTenantSettings } from "./tenant-store";
import { roleLabel, roomLabel } from "./tenant";
import type { Role, RoomName, SessionUser } from "./types";

export function roomForMode(mode: "greeting" | "program" | "backstage"): RoomName {
  return ROOMS[mode];
}

export function roleCanJoinRoom(role: Role, room: RoomName) {
  if (role === "admin" || role === "operator") return true;
  if (room === "campus-greeting") return role === "main" || role === "campus-b" || role === "campus-c";
  if (room === "campus-program") return role === "main" || role === "campus-b" || role === "campus-c";
  if (room === "campus-backstage") return role === "main";
  return false;
}

export function roleCanPublish(role: Role, room: RoomName) {
  if (role === "admin") return true;
  if (role === "operator") return room === "campus-backstage";
  if (room === "campus-greeting") return role === "main" || role === "campus-b" || role === "campus-c";
  if (room === "campus-program") return role === "main";
  if (room === "campus-backstage") return role === "main";
  return false;
}

export async function createMediaToken(user: SessionUser, room: RoomName) {
  if (!roleCanJoinRoom(user.role, room)) {
    throw new Error("Role is not allowed to join this media room.");
  }

  const tenant = await getTenantSettings();
  const canPublish = roleCanPublish(user.role, room);
  const identity = `${user.role}-${randomUUID()}`;
  const token = new AccessToken(getEnv("RTC_API_KEY"), getEnv("RTC_API_SECRET"), {
    identity,
    name: roleLabel(user.role, tenant),
    ttl: "2h",
    metadata: JSON.stringify({
      role: user.role,
      roomLabel: roomLabel(room, tenant),
      brand: tenant.platformName,
      organization: tenant.organizationName
    })
  });

  token.addGrant({
    room,
    roomJoin: true,
    canPublish,
    canSubscribe: true,
    canPublishData: true
  });

  return {
    server_url: publicRtcUrl(),
    participant_token: await token.toJwt(),
    can_publish: canPublish,
    identity
  };
}

export function roomServiceClient() {
  return new RoomServiceClient(getEnv("RTC_API_URL"), getEnv("RTC_API_KEY"), getEnv("RTC_API_SECRET"));
}
