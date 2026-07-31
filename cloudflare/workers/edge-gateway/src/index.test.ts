import { describe, expect, it } from "vitest";
import { hasBasicEnvelopeShape, securityHeaders } from "./index";

describe("edge gateway", () => {
  it("rejects events without a signature field", () => {
    expect(
      hasBasicEnvelopeShape({
        protocol: "babel/1",
        id: "event",
        schema: "babel.message.created/1",
        author_id: "did:babel:amara",
        device_id: "device",
        client_sequence: 1,
      }),
    ).toBe(false);
  });

  it("sets no-store headers for private API responses", () => {
    expect(securityHeaders("request-1").get("cache-control")).toBe("no-store");
  });
});
