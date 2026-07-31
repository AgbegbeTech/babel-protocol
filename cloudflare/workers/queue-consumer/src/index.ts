export interface BabelJob {
  job_id: string;
  idempotency_key: string;
  job_type: string;
  schema_version: number;
  created_at: string;
  attempt_count: number;
  privacy_classification: string;
  authorized_references: string[];
}

export function isMinimumNecessaryJob(value: unknown): value is BabelJob {
  if (!value || typeof value !== "object") return false;
  const job = value as Record<string, unknown>;
  return (
    typeof job.job_id === "string" &&
    typeof job.idempotency_key === "string" &&
    typeof job.job_type === "string" &&
    typeof job.privacy_classification === "string" &&
    Array.isArray(job.authorized_references) &&
    !("message_body" in job) &&
    !("transcript" in job)
  );
}

export default {
  async queue(batch: MessageBatch<unknown>) {
    for (const message of batch.messages) {
      if (!isMinimumNecessaryJob(message.body)) {
        message.retry();
        continue;
      }
      message.ack();
    }
  },
};
