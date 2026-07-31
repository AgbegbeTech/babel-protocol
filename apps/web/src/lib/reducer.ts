import type {
  CulturalContextNote,
  FacilitationResponse,
  Message,
  PublicArtifactSummary,
  RepairThread,
  RoomSnapshot,
  ServerEvent,
  TranslationResult,
} from "../types";

export function applyServerEvent(snapshot: RoomSnapshot, event: ServerEvent): RoomSnapshot {
  switch (event.type) {
    case "room.snapshot":
      return event.payload;
    case "presence.updated":
      return {
        ...snapshot,
        room: {
          ...snapshot.room,
          participants: snapshot.room.participants.map((participant) =>
            participant.id === event.payload.participant_id
              ? { ...participant, present: event.payload.present }
              : participant,
          ),
        },
      };
    case "typing.updated":
      return {
        ...snapshot,
        room: {
          ...snapshot.room,
          participants: snapshot.room.participants.map((participant) =>
            participant.id === event.payload.participant_id
              ? { ...participant, typing: event.payload.typing }
              : participant,
          ),
        },
      };
    case "message.accepted":
      return upsertMessage(snapshot, event.payload);
    case "message.delivered":
      return upsertMessageState(snapshot, event.payload.message_id, "delivered");
    case "translation.started":
      return upsertMessageState(snapshot, event.payload.message_id, "translation_pending");
    case "translation.completed":
      return upsertTranslation(snapshot, event.payload);
    case "message.context_added":
      return upsertContext(snapshot, event.payload);
    case "repair.updated":
      return upsertRepair(snapshot, event.payload);
    case "facilitator.response":
      return upsertFacilitation(snapshot, event.payload);
    case "artifact.proposal_created":
    case "artifact.updated":
      return { ...snapshot, artifact: event.payload };
    case "commons.published":
      return {
        ...snapshot,
        commons_publications: upsertById(snapshot.commons_publications, event.payload),
        artifact: snapshot.artifact ? { ...snapshot.artifact, lifecycle: "published" } : snapshot.artifact,
      };
    case "project.created":
      return { ...snapshot, projects: upsertById(snapshot.projects, event.payload) };
    default:
      return snapshot;
  }
}

function upsertMessage(snapshot: RoomSnapshot, message: Message): RoomSnapshot {
  return {
    ...snapshot,
    messages: upsertById(snapshot.messages, message),
  };
}

function upsertMessageState(
  snapshot: RoomSnapshot,
  messageId: string,
  deliveryState: Message["delivery_state"],
): RoomSnapshot {
  return {
    ...snapshot,
    messages: snapshot.messages.map((message) =>
      message.id === messageId ? { ...message, delivery_state: deliveryState } : message,
    ),
  };
}

function upsertTranslation(snapshot: RoomSnapshot, translation: TranslationResult): RoomSnapshot {
  return {
    ...snapshot,
    messages: snapshot.messages.map((message) =>
      message.id === translation.message_id
        ? {
            ...message,
            delivery_state: "translated",
            translations: upsertByMessageId(message.translations, translation),
          }
        : message,
    ),
  };
}

function upsertContext(snapshot: RoomSnapshot, note: CulturalContextNote): RoomSnapshot {
  return {
    ...snapshot,
    messages: snapshot.messages.map((message) =>
      message.id === note.message_id
        ? { ...message, context_notes: upsertById(message.context_notes, note) }
        : message,
    ),
  };
}

function upsertRepair(snapshot: RoomSnapshot, repair: RepairThread): RoomSnapshot {
  return { ...snapshot, repairs: upsertById(snapshot.repairs, repair) };
}

function upsertFacilitation(
  snapshot: RoomSnapshot,
  response: FacilitationResponse,
): RoomSnapshot {
  return {
    ...snapshot,
    facilitator_responses: upsertById(snapshot.facilitator_responses, response),
  };
}

function upsertById<T extends { id: string }>(items: T[], item: T): T[] {
  const exists = items.some((existing) => existing.id === item.id);
  return exists
    ? items.map((existing) => (existing.id === item.id ? item : existing))
    : [...items, item];
}

function upsertByMessageId(
  items: TranslationResult[],
  item: TranslationResult,
): TranslationResult[] {
  const exists = items.some((existing) => existing.message_id === item.message_id);
  return exists
    ? items.map((existing) => (existing.message_id === item.message_id ? item : existing))
    : [...items, item];
}

export function publicCommonsHasNoTranscript(publications: PublicArtifactSummary[]): boolean {
  return publications.every((publication) => publication.transcript_exposed === false);
}
