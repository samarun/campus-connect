import "server-only";

import bcrypt from "bcryptjs";
import { jwtVerify, SignJWT } from "jose";
import { cookies } from "next/headers";
import type { Role, SessionUser } from "./types";
import { getEnv } from "./env";
import { DEFAULT_TENANT_SETTINGS, roleLabel } from "./tenant";

export const SESSION_COOKIE = "campus_connect_session";
const SESSION_TTL_SECONDS = 12 * 60 * 60;

const encoder = new TextEncoder();

function sessionSecret() {
  return encoder.encode(getEnv("SESSION_SECRET"));
}

export function passwordHashEnv(role: Role) {
  const envByRole: Record<Role, string> = {
    admin: "ADMIN_PASSWORD_HASH",
    operator: "OPERATOR_PASSWORD_HASH",
    main: "MAIN_PASSWORD_HASH",
    "campus-b": "CAMPUS_B_PASSWORD_HASH",
    "campus-c": "CAMPUS_C_PASSWORD_HASH"
  };
  return envByRole[role];
}

export const ROLES: Role[] = ["admin", "operator", "main", "campus-b", "campus-c"];

export async function verifyRolePassword(role: Role, password: string) {
  const envName = passwordHashEnv(role);
  const hash = process.env[envName];
  if (!hash || hash.startsWith("CHANGE_ME") || !hash.startsWith("$2")) {
    return false;
  }
  return bcrypt.compare(password, hash);
}

export async function createSession(user: SessionUser) {
  const token = await new SignJWT({ role: user.role, name: user.name })
    .setProtectedHeader({ alg: "HS256", typ: "JWT" })
    .setSubject(user.role)
    .setIssuedAt()
    .setExpirationTime(`${SESSION_TTL_SECONDS}s`)
    .sign(sessionSecret());

  const cookieStore = await cookies();
  cookieStore.set({
    name: SESSION_COOKIE,
    value: token,
    httpOnly: true,
    secure: process.env.NODE_ENV === "production",
    sameSite: "strict",
    path: "/",
    maxAge: SESSION_TTL_SECONDS
  });
}

export async function clearSession() {
  const cookieStore = await cookies();
  cookieStore.set({
    name: SESSION_COOKIE,
    value: "",
    httpOnly: true,
    secure: process.env.NODE_ENV === "production",
    sameSite: "strict",
    path: "/",
    maxAge: 0
  });
}

export async function getSessionUser(): Promise<SessionUser | null> {
  const cookieStore = await cookies();
  const token = cookieStore.get(SESSION_COOKIE)?.value;
  if (!token) return null;

  try {
    const { payload } = await jwtVerify(token, sessionSecret());
    const role = payload.role;
    if (typeof role !== "string" || !ROLES.includes(role as Role)) return null;
    const name = typeof payload.name === "string" ? payload.name : roleLabel(role as Role, DEFAULT_TENANT_SETTINGS);
    return { role: role as Role, name };
  } catch {
    return null;
  }
}

export function canAccessRole(user: SessionUser, allowed: Role[]) {
  return user.role === "admin" || allowed.includes(user.role);
}

export function userHome(role: Role) {
  if (role === "admin" || role === "operator") return "/operator";
  if (role === "main") return "/campus/main";
  if (role === "campus-b") return "/campus/b";
  return "/campus/c";
}
