import { describe, expect, it } from "vitest";
import { participantFromUrl } from "./index";

describe("realtime room durable object", () => {
  it("extracts participant identity from websocket URL", () => {
    expect(participantFromUrl("https://edge/rooms/x/ws?participant_id=did%3Ababel%3Aamara")).toBe(
      "did:babel:amara",
    );
  });
});
