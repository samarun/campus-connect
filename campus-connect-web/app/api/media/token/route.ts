import { NextResponse } from "next/server";
import { getSessionUser } from "@/lib/auth";
import { createMediaToken, roleCanJoinRoom } from "@/lib/media";
import { checkRateLimit, clientIp } from "@/lib/rate-limit";
import type { RoomName } from "@/lib/types";

export const runtime = "nodejs";

const VALID_ROOMS: RoomName[] = ["campus-greeting", "campus-program", "campus-backstage"];

export async function POST(request: Request) {
  const user = await getSessionUser();
  if (!user) return NextResponse.json({ error: "Login required." }, { status: 401 });

  const ip = clientIp(request.headers);
  const limit = checkRateLimit(`token:${user.role}:${ip}`, 60, 60_000);
  if (!limit.allowed) {
    return NextResponse.json({ error: "Too many media token requests." }, { status: 429 });
  }

  const body = (await request.json().catch(() => null)) as { room_name?: RoomName } | null;
  const room = body?.room_name;
  if (!room || !VALID_ROOMS.includes(room)) {
    return NextResponse.json({ error: "Unsupported media room." }, { status: 400 });
  }

  if (!roleCanJoinRoom(user.role, room)) {
    return NextResponse.json({ error: "This role cannot join that room." }, { status: 403 });
  }

  try {
    const token = await createMediaToken(user, room);
    return NextResponse.json(token, { status: 201 });
  } catch (error) {
    return NextResponse.json(
      { error: error instanceof Error ? error.message : "Media token generation failed." },
      { status: 500 }
    );
  }
}
