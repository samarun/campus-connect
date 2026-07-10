import { DEFAULT_TENANT_SETTINGS } from "@/lib/tenant";
import type { TenantSettings } from "@/lib/types";

export function BrandText({ compact = false, tenant = DEFAULT_TENANT_SETTINGS }: { compact?: boolean; tenant?: TenantSettings }) {
  return (
    <div className="flex min-w-0 items-center">
      <div className="min-w-0">
        <p className={compact ? "truncate text-xs font-semibold uppercase tracking-[0.16em] text-[#2f9b91]" : "truncate text-sm font-semibold uppercase tracking-[0.16em] text-[#2f9b91]"}>
          {tenant.organizationName}
        </p>
        <h1 className={compact ? "truncate text-lg font-bold text-[#10252f] sm:text-xl" : "truncate text-3xl font-bold text-[#10252f]"}>{tenant.platformName}</h1>
      </div>
    </div>
  );
}
