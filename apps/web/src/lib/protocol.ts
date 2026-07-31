import { ed25519 } from "@noble/curves/ed25519";
import type { DemoIdentity, ProtocolEvent } from "../types";

function base64ToBytes(value: string): Uint8Array {
  return Uint8Array.from(atob(value), (char) => char.charCodeAt(0));
}

function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  bytes.forEach((byte) => {
    binary += String.fromCharCode(byte);
  });
  return btoa(binary);
}

export function createUnsignedMessageEvent(
  identity: DemoIdentity,
  roomId: string,
  sequence: number,
  originalText: string,
): ProtocolEvent {
  return {
    protocol: "babel/1",
    id: crypto.randomUUID(),
    schema: "babel.message.created/1",
    version: 1,
    author_id: identity.participant_id,
    device_id: identity.device_id,
    room_id: roomId,
    created_at: new Date().toISOString(),
    client_sequence: sequence,
    parent_ids: [],
    scope: "room",
    expires_at: null,
    content: { original_text: originalText },
    attachments: [],
    signature: "",
  };
}

export function signingPayload(event: ProtocolEvent): Record<string, unknown> {
  return {
    protocol: event.protocol,
    id: event.id,
    schema: event.schema,
    version: event.version,
    author_id: event.author_id,
    device_id: event.device_id,
    room_id: event.room_id,
    created_at: event.created_at,
    client_sequence: event.client_sequence,
    parent_ids: event.parent_ids,
    scope: event.scope,
    expires_at: event.expires_at,
    content: event.content,
    attachments: event.attachments,
  };
}

export async function signProtocolEvent(
  event: ProtocolEvent,
  identity: DemoIdentity,
): Promise<ProtocolEvent> {
  const payload = new TextEncoder().encode(JSON.stringify(signingPayload(event)));
  const signature = ed25519.sign(payload, base64ToBytes(identity.private_key));
  return {
    ...event,
    signature: bytesToBase64(signature),
  };
}
