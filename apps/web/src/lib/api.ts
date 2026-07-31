import { API_BASE, DEMO_ROOM_ID } from "./config";
import type { DemoIdentity, RoomSnapshot } from "../types";

async function fetchJson<T>(path: string): Promise<T> {
  const response = await fetch(`${API_BASE}${path}`);
  if (!response.ok) {
    throw new Error(`Request failed: ${response.status}`);
  }
  return response.json() as Promise<T>;
}

export async function fetchDemoIdentities(): Promise<DemoIdentity[]> {
  return fetchJson<DemoIdentity[]>("/api/v1/demo-identities");
}

export async function fetchSnapshot(participantId: string): Promise<RoomSnapshot> {
  return fetchJson<RoomSnapshot>(
    `/api/v1/rooms/${DEMO_ROOM_ID}?participant_id=${encodeURIComponent(participantId)}`,
  );
}
