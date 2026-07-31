CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS identities (
  id TEXT PRIMARY KEY,
  display_name TEXT NOT NULL,
  preferred_languages TEXT[] NOT NULL DEFAULT '{}',
  public_identity_key TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS devices (
  id TEXT PRIMARY KEY,
  identity_id TEXT NOT NULL REFERENCES identities(id) ON DELETE CASCADE,
  display_name TEXT NOT NULL,
  public_key TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS revoked_devices (
  device_id TEXT PRIMARY KEY REFERENCES devices(id) ON DELETE CASCADE,
  revoked_by TEXT NOT NULL,
  reason TEXT,
  revoked_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS rooms (
  id UUID PRIMARY KEY,
  title TEXT NOT NULL,
  lifecycle TEXT NOT NULL CHECK (lifecycle IN ('created', 'active', 'paused', 'closed', 'archived')),
  privacy TEXT NOT NULL,
  retention TEXT NOT NULL,
  artifact_proposal_allowed BOOLEAN NOT NULL DEFAULT true,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  closed_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS room_participants (
  room_id UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
  identity_id TEXT NOT NULL REFERENCES identities(id) ON DELETE CASCADE,
  role TEXT NOT NULL DEFAULT 'participant',
  joined_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  left_at TIMESTAMPTZ,
  PRIMARY KEY (room_id, identity_id)
);

CREATE TABLE IF NOT EXISTS room_language_preferences (
  room_id UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
  identity_id TEXT NOT NULL REFERENCES identities(id) ON DELETE CASCADE,
  preferred_language TEXT NOT NULL,
  translation_target TEXT NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (room_id, identity_id)
);

CREATE TABLE IF NOT EXISTS protocol_events (
  id TEXT PRIMARY KEY,
  protocol TEXT NOT NULL,
  schema TEXT NOT NULL,
  version INTEGER NOT NULL,
  author_id TEXT NOT NULL,
  device_id TEXT NOT NULL,
  room_id UUID REFERENCES rooms(id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL,
  client_sequence BIGINT NOT NULL,
  parent_ids TEXT[] NOT NULL DEFAULT '{}',
  scope TEXT NOT NULL,
  expires_at TIMESTAMPTZ,
  content JSONB NOT NULL,
  attachments JSONB NOT NULL DEFAULT '[]',
  signature TEXT NOT NULL,
  event_hash TEXT NOT NULL,
  inserted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE(device_id, client_sequence)
);

CREATE TABLE IF NOT EXISTS messages (
  id TEXT PRIMARY KEY REFERENCES protocol_events(id) ON DELETE CASCADE,
  room_id UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
  sender_id TEXT NOT NULL REFERENCES identities(id),
  sender_device_id TEXT NOT NULL REFERENCES devices(id),
  original_language TEXT NOT NULL,
  original_text TEXT NOT NULL,
  sent_at TIMESTAMPTZ NOT NULL,
  client_sequence BIGINT NOT NULL,
  reply_to TEXT REFERENCES messages(id),
  signature TEXT NOT NULL,
  event_hash TEXT NOT NULL,
  delivery_state TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS message_events (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
  event_type TEXT NOT NULL,
  actor_id TEXT NOT NULL,
  event_body JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS delivery_acknowledgments (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
  identity_id TEXT NOT NULL REFERENCES identities(id),
  state TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS translations (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
  source_language TEXT NOT NULL,
  target_language TEXT NOT NULL,
  translated_text TEXT NOT NULL,
  provider TEXT NOT NULL,
  confidence REAL,
  uncertain_phrases TEXT[] NOT NULL DEFAULT '{}',
  literal_alternative TEXT,
  cultural_notes TEXT[] NOT NULL DEFAULT '{}',
  review_status TEXT NOT NULL,
  stream_state TEXT NOT NULL,
  generated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS translation_reviews (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
  reviewer_id TEXT NOT NULL REFERENCES identities(id),
  note TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS cultural_context (
  id TEXT PRIMARY KEY,
  message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
  author_id TEXT NOT NULL REFERENCES identities(id),
  note_type TEXT NOT NULL,
  text TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS repair_threads (
  id UUID PRIMARY KEY,
  room_id UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
  target_id TEXT NOT NULL,
  opened_by TEXT NOT NULL REFERENCES identities(id),
  reason TEXT NOT NULL,
  note TEXT NOT NULL,
  state TEXT NOT NULL,
  opened_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS repair_events (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  repair_id UUID NOT NULL REFERENCES repair_threads(id) ON DELETE CASCADE,
  actor_id TEXT NOT NULL REFERENCES identities(id),
  state TEXT NOT NULL,
  note TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS facilitator_requests (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  room_id UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
  requested_by TEXT NOT NULL REFERENCES identities(id),
  prompt_hash TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS facilitator_responses (
  id TEXT PRIMARY KEY,
  room_id UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
  generated_by TEXT NOT NULL,
  disclosure TEXT NOT NULL,
  suggestion TEXT NOT NULL,
  uncertainty TEXT NOT NULL,
  accepted BOOLEAN,
  generated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS understanding_artifacts (
  id UUID PRIMARY KEY,
  room_id UUID NOT NULL REFERENCES rooms(id) ON DELETE RESTRICT,
  title TEXT NOT NULL,
  lifecycle TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS artifact_revisions (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  artifact_id UUID NOT NULL REFERENCES understanding_artifacts(id) ON DELETE CASCADE,
  revision_hash TEXT NOT NULL,
  exact_text TEXT NOT NULL,
  ai_disclosure TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (artifact_id, revision_hash)
);

CREATE TABLE IF NOT EXISTS consent_receipts (
  receipt_id UUID PRIMARY KEY,
  artifact_id UUID NOT NULL REFERENCES understanding_artifacts(id) ON DELETE CASCADE,
  exact_revision_hash TEXT NOT NULL,
  approving_participant TEXT NOT NULL REFERENCES identities(id),
  approving_device TEXT NOT NULL REFERENCES devices(id),
  approved_publication_scope TEXT NOT NULL,
  attribution_preference TEXT NOT NULL,
  approved_translations TEXT[] NOT NULL DEFAULT '{}',
  ai_processing_permissions TEXT[] NOT NULL DEFAULT '{}',
  timestamp TIMESTAMPTZ NOT NULL,
  optional_review_date TIMESTAMPTZ,
  consent_statement_version TEXT NOT NULL,
  signature TEXT NOT NULL,
  UNIQUE (artifact_id, exact_revision_hash, approving_participant)
);

CREATE TABLE IF NOT EXISTS commons_publications (
  id TEXT PRIMARY KEY,
  artifact_id UUID NOT NULL REFERENCES understanding_artifacts(id) ON DELETE RESTRICT,
  title TEXT NOT NULL,
  summary TEXT NOT NULL,
  revision_hash TEXT NOT NULL,
  consent_verified BOOLEAN NOT NULL,
  transcript_exposed BOOLEAN NOT NULL DEFAULT false,
  published_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS collaboration_projects (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  problem TEXT NOT NULL,
  status TEXT NOT NULL,
  source_artifact_id TEXT NOT NULL REFERENCES commons_publications(id),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS project_contributors (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  project_id TEXT NOT NULL REFERENCES collaboration_projects(id) ON DELETE CASCADE,
  contributor_id TEXT NOT NULL,
  contribution_type TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS contribution_needs (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  project_id TEXT NOT NULL REFERENCES collaboration_projects(id) ON DELETE CASCADE,
  contribution_type TEXT NOT NULL,
  description TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS milestones (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  project_id TEXT NOT NULL REFERENCES collaboration_projects(id) ON DELETE CASCADE,
  title TEXT NOT NULL,
  status TEXT NOT NULL,
  due_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS audit_records (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  actor_id TEXT NOT NULL,
  action TEXT NOT NULL,
  resource_type TEXT NOT NULL,
  resource_id TEXT NOT NULL,
  privacy_classification TEXT NOT NULL,
  content_hash TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS outbox_events (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  aggregate_type TEXT NOT NULL,
  aggregate_id TEXT NOT NULL,
  event_type TEXT NOT NULL,
  payload JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  processed_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS asynchronous_jobs (
  id UUID PRIMARY KEY,
  idempotency_key TEXT NOT NULL UNIQUE,
  job_type TEXT NOT NULL,
  schema_version INTEGER NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  attempt_count INTEGER NOT NULL,
  privacy_classification TEXT NOT NULL,
  authorized_references TEXT[] NOT NULL DEFAULT '{}'
);

CREATE TABLE IF NOT EXISTS object_references (
  id TEXT PRIMARY KEY,
  provider TEXT NOT NULL,
  bucket TEXT NOT NULL,
  object_key TEXT NOT NULL,
  content_hash TEXT NOT NULL,
  privacy_classification TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_messages_room_sent_at ON messages(room_id, sent_at);
CREATE INDEX IF NOT EXISTS idx_translations_message ON translations(message_id);
CREATE INDEX IF NOT EXISTS idx_repairs_room_state ON repair_threads(room_id, state);
CREATE INDEX IF NOT EXISTS idx_protocol_events_device_sequence ON protocol_events(device_id, client_sequence);
CREATE INDEX IF NOT EXISTS idx_commons_publications_revision ON commons_publications(revision_hash);
