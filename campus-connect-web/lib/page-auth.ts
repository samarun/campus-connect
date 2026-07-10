import { redirect } from "next/navigation";
import { canAccessRole, getSessionUser } from "./auth";
import type { Role } from "./types";

export async function requireUser(allowed?: Role[]) {
  const user = await getSessionUser();
  if (!user) redirect("/login");
  if (allowed && !canAccessRole(user, allowed)) redirect("/");
  return user;
}

