"use client";

import { LogIn } from "lucide-react";
import { useRouter } from "next/navigation";
import { FormEvent, useState } from "react";
import { apiError, clientErrorMessage, readJsonResponse } from "@/lib/client-response";
import { roleLabel } from "@/lib/tenant";
import type { Role, TenantSettings } from "@/lib/types";

const LOGIN_ROLES: Role[] = ["admin", "operator", "main", "campus-b", "campus-c"];

export function LoginForm({ tenant }: { tenant: TenantSettings }) {
  const router = useRouter();
  const [role, setRole] = useState<Role>("operator");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setBusy(true);
    setError(null);

    try {
      const response = await fetch("/api/auth/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ role, password })
      });
      const json = await readJsonResponse<{ error?: string; redirectTo?: string }>(response);
      if (!response.ok) {
        setError(apiError(json, `${tenant.platformName} login failed.`));
        return;
      }
      router.push(json.redirectTo ?? "/operator");
      router.refresh();
    } catch (err) {
      setError(clientErrorMessage(err, `${tenant.platformName} login failed.`));
    } finally {
      setBusy(false);
    }
  }

  return (
    <form onSubmit={submit} className="mt-6 space-y-4">
      <label className="block">
        <span className="mb-1 block text-sm font-semibold text-[#10252f]">Role</span>
        <select
          value={role}
          onChange={(event) => setRole(event.target.value as Role)}
          className="h-11 w-full rounded-md border border-[#10252f]/15 bg-white px-3 text-[#10252f] outline-none focus:border-[#2f9b91]"
        >
          {LOGIN_ROLES.map((item) => (
            <option key={item} value={item}>
              {roleLabel(item, tenant)}
            </option>
          ))}
        </select>
      </label>

      <label className="block">
        <span className="mb-1 block text-sm font-semibold text-[#10252f]">Password</span>
        <input
          type="password"
          value={password}
          onChange={(event) => setPassword(event.target.value)}
          autoComplete="current-password"
          className="h-11 w-full rounded-md border border-[#10252f]/15 bg-white px-3 text-[#10252f] outline-none focus:border-[#2f9b91]"
        />
      </label>

      {error ? <p className="rounded-md border border-[#e45f5f]/30 bg-[#fff6f2] px-3 py-2 text-sm text-[#b83f3f]">{error}</p> : null}

      <button
        type="submit"
        disabled={busy}
        className="inline-flex h-11 w-full items-center justify-center gap-2 rounded-md bg-[#10252f] px-4 text-sm font-bold text-white transition hover:bg-[#173746] disabled:opacity-60"
      >
        <LogIn className="h-4 w-4" aria-hidden="true" />
        Enter {tenant.platformName}
      </button>
    </form>
  );
}
