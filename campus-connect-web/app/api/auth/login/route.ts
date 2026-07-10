import { NextResponse } from "next/server";
import { createSession, ROLES, verifyRolePassword, userHome } from "@/lib/auth";
import { checkRateLimit, clientIp } from "@/lib/rate-limit";
import { getTenantSettings } from "@/lib/tenant-store";
import { roleLabel } from "@/lib/tenant";
import type { Role } from "@/lib/types";

export const runtime = "nodejs";

export async function POST(request: Request) {
  const ip = clientIp(request.headers);
  const limit = checkRateLimit(`login:${ip}`, 10, 60_000);
  if (!limit.allowed) {
    return NextResponse.json({ error: "Too many login attempts. Wait a minute and try again." }, { status: 429 });
  }

  const body = (await request.json().catch(() => null)) as { role?: Role; password?: string } | null;
  if (!body?.role || !body.password || !ROLES.includes(body.role)) {
    return NextResponse.json({ error: "Choose a valid role and enter the password." }, { status: 400 });
  }

  const valid = await verifyRolePassword(body.role, body.password);
  if (!valid) {
    return NextResponse.json({ error: "Login failed." }, { status: 401 });
  }

  const tenant = await getTenantSettings();
  const user = { role: body.role, name: roleLabel(body.role, tenant) };
  await createSession(user);
  return NextResponse.json({ ok: true, user, redirectTo: userHome(body.role) });
}
