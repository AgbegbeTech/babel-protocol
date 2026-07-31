import { describe, expect, it } from "vitest";
import { isMinimumNecessaryJob } from "./index";

describe("queue consumer", () => {
  it("rejects jobs that include a transcript", () => {
    expect(
      isMinimumNecessaryJob({
        job_id: "job-1",
        idempotency_key: "job-1",
        job_type: "translation",
        privacy_classification: "room_private",
        authorized_references: ["message-1"],
        transcript: "private text",
      }),
    ).toBe(false);
  });
});
