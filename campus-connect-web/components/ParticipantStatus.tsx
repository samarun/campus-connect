"use client";

import { Circle } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { roleLabel, roomLabel } from "@/lib/tenant";
import type { Role, RoomName, TenantSettings } from "@/lib/types";

type Participant = {
  identity: string;
  name: string;
  tracks: number;
  metadata?: {
    role?: Role;
  };
};

const CAMPUS_ROLES: Role[] = ["main", "campus-b", "campus-c"];

function roleFromParticipant(participant: Participant): Role | null {
  if (participant.metadata?.role) return participant.metadata.role;
  const found = CAMPUS_ROLES.find((role) => participant.identity.startsWith(`${role}-`));
  return found ?? null;
}

export function ParticipantStatus({ room, tenant }: { room: RoomName; tenant: TenantSettings }) {
  const [participants, setParticipants] = useState<Participant[]>([]);
  const [warning, setWarning] = useState<string | null>(null);

  useEffect(() => {
    let active = true;

    async function load() {
      try {
        const response = await fetch(`/api/media/participants?room=${room}`, { cache: "no-store" });
        const json = (await response.json()) as { participants?: Participant[]; warning?: string };
        if (!active) return;
        setParticipants(json.participants ?? []);
        setWarning(json.warning ?? null);
      } catch {
        if (active) setWarning("Participant status unavailable");
      }
    }

    load();
    const timer = window.setInterval(load, 5000);
    return () => {
      active = false;
      window.clearInterval(timer);
    };
  }, [room]);

  const online = useMemo(() => {
    const set = new Set<Role>();
    for (const participant of participants) {
      const role = roleFromParticipant(participant);
      if (role) set.add(role);
    }
    return set;
  }, [participants]);

  return (
    <section className="rounded-lg border border-[#10252f]/10 bg-white p-4 shadow-sm">
      <div className="mb-3 flex items-center justify-between gap-3">
        <h3 className="text-base font-bold text-[#10252f]">Site status</h3>
        <span className="text-xs font-semibold uppercase tracking-[0.16em] text-[#10252f]/50">{roomLabel(room, tenant)}</span>
      </div>
      <div className="grid gap-2 sm:grid-cols-3">
        {CAMPUS_ROLES.map((role) => {
          const isOnline = online.has(role);
          return (
            <div key={role} className="flex items-center gap-2 rounded-md border border-[#10252f]/10 bg-[#f8faf7] px-3 py-2">
              <Circle className={isOnline ? "h-3 w-3 fill-[#2f9b91] text-[#2f9b91]" : "h-3 w-3 fill-[#d8ddd7] text-[#d8ddd7]"} />
              <span className="text-sm font-semibold text-[#10252f]">{roleLabel(role, tenant)}</span>
              <span className="ml-auto text-xs text-[#10252f]/60">{isOnline ? "Online" : "Offline"}</span>
            </div>
          );
        })}
      </div>
      {warning ? <p className="mt-3 text-sm text-[#b83f3f]">{warning}</p> : null}
    </section>
  );
}
