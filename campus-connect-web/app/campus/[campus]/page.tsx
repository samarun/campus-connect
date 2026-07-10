import { notFound } from "next/navigation";
import { AppShell } from "@/components/AppShell";
import { CampusConsole } from "@/components/CampusConsole";
import { CAMPUS_ROLE_BY_KEY } from "@/lib/constants";
import { relayPublishUrl } from "@/lib/env";
import { requireUser } from "@/lib/page-auth";
import { campusLabel } from "@/lib/tenant";
import { getTenantSettings } from "@/lib/tenant-store";
import type { CampusKey } from "@/lib/types";

export const metadata = {
  title: "Campus"
};

const CAMPUS_KEYS: CampusKey[] = ["main", "b", "c"];

export default async function CampusPage({ params }: { params: Promise<{ campus: string }> }) {
  const { campus: campusParam } = await params;
  if (!CAMPUS_KEYS.includes(campusParam as CampusKey)) notFound();
  const campus = campusParam as CampusKey;
  const campusRole = CAMPUS_ROLE_BY_KEY[campus];
  const [user, tenant] = await Promise.all([requireUser([campusRole, "operator"]), getTenantSettings()]);
  const siteName = campusLabel(campus, tenant);

  return (
    <AppShell user={user} eyebrow="Site" title={`${siteName} console`}>
      <CampusConsole campus={campus} role={campusRole} tenant={tenant} relayPublishUrl={relayPublishUrl()} />
    </AppShell>
  );
}
