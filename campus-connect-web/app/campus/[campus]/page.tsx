import { notFound, redirect } from "next/navigation";
import { AppShell } from "@/components/AppShell";
import { CampusConsole } from "@/components/CampusConsole";
import { CAMPUS_ROLE_BY_KEY } from "@/lib/constants";
import { relayPublishUrl } from "@/lib/env";
import { requireUser } from "@/lib/page-auth";
import { campusHref, resolveCampusParam } from "@/lib/site-routes";
import { campusLabel } from "@/lib/tenant";
import { getTenantSettings } from "@/lib/tenant-store";

export const metadata = {
  title: "Campus"
};

export default async function CampusPage({ params }: { params: Promise<{ campus: string }> }) {
  const [{ campus: campusParam }, tenant] = await Promise.all([params, getTenantSettings()]);
  const campus = resolveCampusParam(campusParam, tenant);
  if (!campus) notFound();

  const canonicalPath = campusHref(campus, tenant);
  if (`/campus/${campusParam}` !== canonicalPath) redirect(canonicalPath);

  const campusRole = CAMPUS_ROLE_BY_KEY[campus];
  const user = await requireUser([campusRole, "operator"]);
  const siteName = campusLabel(campus, tenant);

  return (
    <AppShell user={user} eyebrow="Site" title={`${siteName} console`}>
      <CampusConsole campus={campus} role={campusRole} tenant={tenant} relayPublishUrl={relayPublishUrl()} />
    </AppShell>
  );
}
