import { NextResponse } from "next/server";
import { getSessionUser } from "@/lib/auth";
import { getTenantSettings, setTenantSettings } from "@/lib/tenant-store";
import type { TenantSettings } from "@/lib/types";

export const runtime = "nodejs";

export async function GET() {
  return NextResponse.json(await getTenantSettings());
}

export async function POST(request: Request) {
  const user = await getSessionUser();
  if (!user) return NextResponse.json({ error: "Login required." }, { status: 401 });
  if (user.role !== "admin" && user.role !== "operator") {
    return NextResponse.json({ error: "Only administrators and operators can update site labels." }, { status: 403 });
  }

  const body = (await request.json().catch(() => null)) as Partial<TenantSettings> | null;
  if (!body || typeof body !== "object") {
    return NextResponse.json({ error: "Invalid setup payload." }, { status: 400 });
  }

  return NextResponse.json(await setTenantSettings(body));
}
