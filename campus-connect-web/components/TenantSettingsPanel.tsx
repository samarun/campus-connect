"use client";

import { Save } from "lucide-react";
import { useRouter } from "next/navigation";
import { FormEvent, useEffect, useState } from "react";
import { apiError, clientErrorMessage, readJsonResponse } from "@/lib/client-response";
import type { TenantSettings } from "@/lib/types";

const FIELDS: Array<{ key: keyof TenantSettings; label: string }> = [
  { key: "platformName", label: "Platform name" },
  { key: "organizationName", label: "Customer / organization name" },
  { key: "primarySiteName", label: "Primary site name" },
  { key: "remoteSiteAName", label: "Second site display name" },
  { key: "remoteSiteBName", label: "Third site display name" },
  { key: "greetingModeName", label: "First mode name" },
  { key: "programModeName", label: "Second mode name" }
];

export function TenantSettingsPanel({ tenant }: { tenant: TenantSettings }) {
  const router = useRouter();
  const [form, setForm] = useState<TenantSettings>(tenant);
  const [status, setStatus] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    setForm(tenant);
  }, [tenant]);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setBusy(true);
    setStatus(null);

    try {
      const response = await fetch("/api/tenant", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(form)
      });
      const json = await readJsonResponse<TenantSettings | { error?: string }>(response);

      if (!response.ok) {
        setStatus(apiError(json, "Unable to save setup."));
        return;
      }

      setForm(json as TenantSettings);
      setStatus("Setup saved. Refresh open receiver windows to show the new names.");
      router.refresh();
    } catch (err) {
      setStatus(clientErrorMessage(err, "Unable to save setup."));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="rounded-lg border border-[#10252f]/10 bg-white p-4 shadow-sm">
      <div className="mb-4">
        <h3 className="text-base font-bold text-[#10252f]">Customer setup</h3>
        <p className="mt-1 text-sm text-[#10252f]/60">Set the customer-facing product, site, and mode names before production use.</p>
      </div>
      <form onSubmit={submit} className="grid gap-3">
        {FIELDS.map((field) => (
          <label key={field.key} className="grid gap-1">
            <span className="text-xs font-semibold uppercase tracking-[0.12em] text-[#10252f]/55">{field.label}</span>
            <input
              value={form[field.key]}
              maxLength={80}
              onChange={(event) => setForm((current) => ({ ...current, [field.key]: event.target.value }))}
              className="h-10 rounded-md border border-[#10252f]/15 bg-white px-3 text-sm font-semibold text-[#10252f] outline-none focus:border-[#2f9b91]"
            />
          </label>
        ))}
        <button
          type="submit"
          disabled={busy}
          className="inline-flex h-10 items-center justify-center gap-2 rounded-md bg-[#10252f] px-4 text-sm font-bold text-white transition hover:bg-[#173746] disabled:opacity-60"
        >
          <Save className="h-4 w-4" aria-hidden="true" />
          Save setup
        </button>
        {status ? <p className="text-sm text-[#10252f]/70">{status}</p> : null}
      </form>
    </section>
  );
}
