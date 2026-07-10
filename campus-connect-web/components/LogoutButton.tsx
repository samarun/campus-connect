"use client";

import { LogOut } from "lucide-react";
import { useRouter } from "next/navigation";
import { useState } from "react";

export function LogoutButton() {
  const router = useRouter();
  const [busy, setBusy] = useState(false);

  async function logout() {
    setBusy(true);
    await fetch("/api/auth/logout", { method: "POST" });
    router.push("/login");
    router.refresh();
  }

  return (
    <button
      type="button"
      onClick={logout}
      disabled={busy}
      title="Sign out"
      className="inline-flex h-10 items-center gap-2 rounded-md border border-[#10252f]/15 bg-white px-3 text-sm font-semibold text-[#10252f] shadow-sm transition hover:border-[#e45f5f]/50 hover:text-[#b83f3f] disabled:opacity-60"
    >
      <LogOut className="h-4 w-4" aria-hidden="true" />
      Sign out
    </button>
  );
}
