import { NextResponse } from "next/server";
import { getSessionUser } from "@/lib/auth";
import { roleCanJoinRoom, roomServiceClient } from "@/lib/media";
import type { RoomName } from "@/lib/types";

export const runtime = "nodejs";

const VALID_ROOMS: RoomName[] = ["campus-greeting", "campus-program", "campus-backstage"];

function parseMetadata(metadata?: string) {
  if (!metadata) return {};
  try {
    return JSON.parse(metadata) as Record<string, unknown>;
  } catch {
    return {};
  }
}

function jsonSafe(value: unknown): unknown {
  if (typeof value === "bigint") return value.toString();
  if (Array.isArray(value)) return value.map(jsonSafe);
  if (value && typeof value === "object") {
    return Object.fromEntries(Object.entries(value as Record<string, unknown>).map(([key, item]) => [key, jsonSafe(item)]));
  }
  return value;
}

function participantStatusWarning(error: unknown) {
  const message = error instanceof Error ? error.message : "Unable to read media participants.";
  if (/fetch failed|ECONNREFUSED|ENOTFOUND|ETIMEDOUT|EHOSTUNREACH|network/i.test(message)) {
    return "Participant status unavailable. Check RTC_API_URL and confirm the real-time server is reachable from the web container.";
  }
  return message;
}

export async function GET(request: Request) {
  const user = await getSessionUser();
  if (!user) return NextResponse.json({ error: "Login required." }, { status: 401 });

  const { searchParams } = new URL(request.url);
  const room = searchParams.get("room") as RoomName | null;
  if (!room || !VALID_ROOMS.includes(room)) {
    return NextResponse.json({ error: "Unsupported room." }, { status: 400 });
  }

  if (!roleCanJoinRoom(user.role, room)) {
    return NextResponse.json({ error: "This role cannot inspect that room." }, { status: 403 });
  }

  try {
    const participants = await roomServiceClient().listParticipants(room);
    return NextResponse.json({
      room,
      participants: participants.map((participant) => ({
        identity: participant.identity,
        name: participant.name,
        metadata: jsonSafe(parseMetadata(participant.metadata)),
        state: String(participant.state),
        joinedAt: jsonSafe(participant.joinedAt),
        tracks: participant.tracks.length
      }))
    });
  } catch (error) {
    return NextResponse.json(
      { room, participants: [], warning: participantStatusWarning(error) },
      { status: 200 }
    );
  }
}
