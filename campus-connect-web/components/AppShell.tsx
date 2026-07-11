import Link from "next/link";
import type { SessionUser } from "@/lib/types";
import { getTenantSettings } from "@/lib/tenant-store";
import { roleLabel } from "@/lib/tenant";
import { homeHrefForRole } from "@/lib/site-routes";
import { BrandText } from "./BrandText";
import { LogoutButton } from "./LogoutButton";
import { ThemeToggle } from "./ThemeToggle";

export async function AppShell({
  user,
  eyebrow,
  title,
  children
}: {
  user: SessionUser;
  eyebrow?: string;
  title: string;
  children: React.ReactNode;
}) {
  const tenant = await getTenantSettings();
  const homeHref = homeHrefForRole(user.role, tenant);

  return (
    <main className="min-h-screen bg-[#f8faf7]">
      <header className="border-b border-[#10252f]/10 bg-[#fffdf8] px-4 py-3 backdrop-blur sm:px-6">
        <div className="mx-auto flex max-w-7xl flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          <Link
            href={homeHref}
            className="block min-w-0 rounded-md outline-none transition hover:opacity-85 focus-visible:ring-2 focus-visible:ring-[#2f9b91] focus-visible:ring-offset-2 focus-visible:ring-offset-[#fffdf8]"
            aria-label={`Go to ${tenant.platformName} home`}
          >
            <BrandText compact tenant={tenant} />
          </Link>
          <div className="flex flex-wrap items-center gap-3">
            <ThemeToggle />
            <div className="rounded-md border border-[#10252f]/10 bg-white px-3 py-2 text-sm text-[#10252f]/80">
              <span className="font-semibold text-[#10252f]">{roleLabel(user.role, tenant)}</span>
            </div>
            <LogoutButton />
          </div>
        </div>
      </header>

      <section className="mx-auto max-w-7xl px-4 py-6 sm:px-6">
        <div className="mb-5">
          {eyebrow ? <p className="text-sm font-semibold uppercase tracking-[0.16em] text-[#2f9b91]">{eyebrow}</p> : null}
          <h2 className="mt-1 text-2xl font-bold text-[#10252f] sm:text-3xl">{title}</h2>
        </div>
        {children}
      </section>
    </main>
  );
}
