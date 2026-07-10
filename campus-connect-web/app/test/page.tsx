import { AppShell } from "@/components/AppShell";
import { DeviceTest } from "@/components/DeviceTest";
import { requireUser } from "@/lib/page-auth";
import { getTenantSettings } from "@/lib/tenant-store";

export const metadata = {
  title: "Device Test"
};

export default async function TestPage() {
  const [user, tenant] = await Promise.all([requireUser(["operator", "main", "campus-b", "campus-c"]), getTenantSettings()]);
  return (
    <AppShell user={user} eyebrow="Readiness" title={`${tenant.platformName} camera, microphone, and speaker test`}>
      <DeviceTest />
    </AppShell>
  );
}
