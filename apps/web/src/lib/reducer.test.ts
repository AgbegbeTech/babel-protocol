import { describe, expect, it } from "vitest";
import { applyServerEvent, publicCommonsHasNoTranscript } from "./reducer";
import type { RoomSnapshot } from "../types";

const snapshot: RoomSnapshot = {
  room: {
    id: "11111111-1111-4111-8111-111111111111",
    title: "Demo",
    lifecycle: "active",
    privacy: "private_by_default",
    retention: "participant_controlled",
    artifact_proposal_allowed: true,
    created_at: new Date().toISOString(),
    participants: [
      {
        id: "did:babel:amara",
        display_name: "Amara",
        preferred_language: "English",
        translation_target: "es",
        location_label: "Lagos, Nigeria",
        present: false,
        typing: false,
      },
    ],
  },
  messages: [],
  repairs: [],
  facilitator_responses: [],
  artifact: null,
  approvals: [],
  consent_receipt_ids: [],
  commons_publications: [],
  projects: [],
  server_time: new Date().toISOString(),
};

describe("room reducer", () => {
  it("keeps presence ephemeral in room state only", () => {
    const updated = applyServerEvent(snapshot, {
      type: "presence.updated",
      payload: { participant_id: "did:babel:amara", present: true },
    });

    expect(updated.room.participants[0].present).toBe(true);
    expect(updated.messages).toHaveLength(0);
  });

  it("verifies public Commons entries do not expose transcripts", () => {
    expect(
      publicCommonsHasNoTranscript([
        {
          id: "commons-1",
          title: "Shared insight",
          summary: "Approved summary",
          revision_hash: "hash",
          consent_verified: true,
          transcript_exposed: false,
        },
      ]),
    ).toBe(true);
  });
});
