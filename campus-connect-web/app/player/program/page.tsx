import { AppShell } from "@/components/AppShell";
import { OvenPlayerClient } from "@/components/OvenPlayerClient";
import { publicPlaybackBaseUrl } from "@/lib/env";
import { requireUser } from "@/lib/page-auth";
import { modeLabel } from "@/lib/tenant";
import { getTenantSettings } from "@/lib/tenant-store";

export const metadata = {
  title: "Player"
};

export default async function ProgramPlayerPage() {
  const [user, tenant] = await Promise.all([requireUser(["operator", "main", "campus-b", "campus-c"]), getTenantSettings()]);
  const programModeName = modeLabel("program", tenant);
  return (
    <AppShell user={user} eyebrow={`${programModeName} playback`} title={`${tenant.platformName} ${programModeName} player`}>
      <OvenPlayerClient baseUrl={publicPlaybackBaseUrl()} tenant={tenant} />
    </AppShell>
  );
}
