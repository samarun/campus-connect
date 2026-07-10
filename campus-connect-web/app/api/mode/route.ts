import { NextResponse } from "next/server";
import { getSessionUser } from "@/lib/auth";
import { getModeState, setModeState } from "@/lib/mode-store";
import type { Mode } from "@/lib/types";

export const runtime = "nodejs";

export async function GET() {
  const user = await getSessionUser();
  if (!user) return NextResponse.json({ error: "Login required." }, { status: 401 });
  return NextResponse.json(await getModeState());
}

export async function POST(request: Request) {
  const user = await getSessionUser();
  if (!user) return NextResponse.json({ error: "Login required." }, { status: 401 });
  if (user.role !== "admin" && user.role !== "operator") {
    return NextResponse.json({ error: "Only administrators and operators can switch mode." }, { status: 403 });
  }

  const body = (await request.json().catch(() => null)) as { mode?: Mode } | null;
  if (body?.mode !== "greeting" && body?.mode !== "program") {
    return NextResponse.json({ error: "Mode must be greeting or program." }, { status: 400 });
  }

  return NextResponse.json(await setModeState(body.mode, user.role));
}
