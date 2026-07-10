import { AppShell } from "@/components/AppShell";
import { OperatorDashboard } from "@/components/OperatorDashboard";
import { requireUser } from "@/lib/page-auth";
import { getTenantSettings } from "@/lib/tenant-store";

export const metadata = {
  title: "Operator"
};

export default async function OperatorPage() {
  const [user, tenant] = await Promise.all([requireUser(["operator"]), getTenantSettings()]);
  return (
    <AppShell user={user} eyebrow="Control room" title={`${tenant.platformName} operator console`}>
      <OperatorDashboard tenant={tenant} />
    </AppShell>
  );
}
