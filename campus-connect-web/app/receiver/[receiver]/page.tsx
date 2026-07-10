import { notFound } from "next/navigation";
import { ReceiverView } from "@/components/ReceiverView";
import { requireUser } from "@/lib/page-auth";
import { getTenantSettings } from "@/lib/tenant-store";
import type { ReceiverKey } from "@/lib/types";

export const metadata = {
  title: "Receiver"
};

const RECEIVER_KEYS: ReceiverKey[] = ["main", "campus-b", "campus-c"];

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
  if (!RECEIVER_KEYS.includes(receiverParam as ReceiverKey)) notFound();
  const receiver = query.view === "grid" ? "grid" : (receiverParam as ReceiverKey);
  return <ReceiverView receiver={receiver} tenant={tenant} />;
}
