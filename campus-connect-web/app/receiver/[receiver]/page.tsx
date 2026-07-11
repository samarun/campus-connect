import { notFound, redirect } from "next/navigation";
import { ReceiverView } from "@/components/ReceiverView";
import { requireUser } from "@/lib/page-auth";
import { receiverHref, resolveReceiverParam } from "@/lib/site-routes";
import { getTenantSettings } from "@/lib/tenant-store";

export const metadata = {
  title: "Receiver"
};

export default async function ReceiverPage({
  params,
  searchParams
}: {
  params: Promise<{ receiver: string }>;
  searchParams: Promise<{ view?: string }>;
}) {
  const [, tenant, { receiver: receiverParam }, query] = await Promise.all([
    requireUser(["operator", "main", "campus-b", "campus-c"]),
    getTenantSettings(),
    params,
    searchParams
  ]);
  const receiver = query.view === "grid" ? "grid" : resolveReceiverParam(receiverParam, tenant);
  if (!receiver) notFound();

  const canonicalPath = receiverHref(receiver, tenant);
  if (`/receiver/${receiverParam}` !== canonicalPath || query.view) redirect(canonicalPath);

  return <ReceiverView receiver={receiver} tenant={tenant} />;
}
