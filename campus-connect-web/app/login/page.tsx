import { redirect } from "next/navigation";
import { BrandText } from "@/components/BrandText";
import { LoginForm } from "@/components/LoginForm";
import { ThemeToggle } from "@/components/ThemeToggle";
import { getSessionUser, userHome } from "@/lib/auth";
import { getTenantSettings } from "@/lib/tenant-store";

export const metadata = {
  title: "Login"
};

export default async function LoginPage() {
  const [user, tenant] = await Promise.all([getSessionUser(), getTenantSettings()]);
  if (user) redirect(userHome(user.role));

  return (
    <main className="flex min-h-screen items-center justify-center bg-[#f8faf7] px-4 py-10">
      <section className="w-full max-w-md rounded-lg border border-[#10252f]/10 bg-[#fffdf8] p-6 shadow-sm">
        <div className="flex items-start justify-between gap-4">
          <BrandText tenant={tenant} />
          <ThemeToggle />
        </div>
        <div className="mt-6">
          <h2 className="text-2xl font-bold text-[#10252f]">{tenant.platformName} login</h2>
          <p className="mt-2 text-sm text-[#10252f]/65">Use the role password assigned for this site position.</p>
        </div>
        <LoginForm tenant={tenant} />
      </section>
    </main>
  );
}
