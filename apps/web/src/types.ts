export type DeliveryState =
  | "local_pending"
  | "received_by_edge"
  | "validating"
  | "persisted"
  | "delivered"
  | "translation_pending"
  | "translated"
  | "translation_reviewed"
  | "failed";

export type ArtifactLifecycle =
  | "draft"
  | "proposed"
  | "under_review"
  | "approved"
  | "published"
  | "withdrawn";

export interface Participant {
  id: string;
  display_name: string;
  preferred_language: string;
  translation_target: string;
  location_label: string;
  present: boolean;
  typing: boolean;
}

export interface Room {
  id: string;
  title: string;
  lifecycle: "created" | "active" | "paused" | "closed" | "archived";
  privacy: string;
  retention: string;
  participants: Participant[];
  artifact_proposal_allowed: boolean;
  created_at: string;
}

export interface TranslationResult {
  message_id: string;
  translated_text: string;
  source_language: string;
  target_language: string;
  provider: string;
  confidence: number;
  uncertain_phrases: string[];
  literal_alternative?: string | null;
  cultural_notes: string[];
  generated_at: string;
  review_status: "unreviewed" | "challenged" | "corrected" | "reviewed";
  stream_state:
    | "translation_started"
    | "translation_partial"
    | "translation_completed"
    | "translation_failed";
}

export interface CulturalContextNote {
  id: string;
  message_id: string;
  author_id: string;
  note_type: string;
  text: string;
  created_at: string;
}

export interface Message {
  id: string;
  room_id: string;
  sender_id: string;
  sender_device_id: string;
  original_language: string;
  original_text: string;
  sent_at: string;
  client_sequence: number;
  reply_to?: string | null;
  signature: string;
  event_hash: string;
  delivery_state: DeliveryState;
  translations: TranslationResult[];
  context_notes: CulturalContextNote[];
}

export interface RepairThread {
  id: string;
  room_id: string;
  target_id: string;
  opened_by: string;
  reason:
    | "misunderstood"
    | "translation_inaccurate"
    | "missing_context"
    | "harmful_wording"
    | "disagree_with_interpretation"
    | "needs_clarification"
    | "preserve_original_wording";
  note: string;
  state:
    | "open"
    | "acknowledged"
    | "clarifying"
    | "resolved"
    | "unresolved"
    | "withdrawn";
  opened_at: string;
  updated_at: string;
}

export interface FacilitationResponse {
  id: string;
  room_id: string;
  generated_by: string;
  disclosure: string;
  suggestion: string;
  uncertainty: string;
  considered: string[];
  missing_context: string[];
  generated_at: string;
  accepted?: boolean | null;
}

export interface UnderstandingArtifactDraft {
  id: string;
  room_id: string;
  title: string;
  originating_question: string;
  shared_summary: string;
  cultural_contexts: string[];
  linguistic_ambiguities: string[];
  areas_of_agreement: string[];
  unresolved_disagreements: string[];
  minority_perspectives: string[];
  evidence_references: string[];
  approved_translations: string[];
  required_approvers: string[];
  ai_disclosure: string;
  publication_scope: string;
  revision_hash: string;
  lifecycle: ArtifactLifecycle;
  created_at: string;
}

export interface PublicArtifactSummary {
  id: string;
  title: string;
  summary: string;
  revision_hash: string;
  consent_verified: boolean;
  transcript_exposed: boolean;
}

export interface ProjectSummary {
  id: string;
  title: string;
  status: string;
  source_artifact_id: string;
  contribution_needs: string[];
}

export interface RoomSnapshot {
  room: Room;
  messages: Message[];
  repairs: RepairThread[];
  facilitator_responses: FacilitationResponse[];
  artifact?: UnderstandingArtifactDraft | null;
  approvals: string[];
  consent_receipt_ids: string[];
  commons_publications: PublicArtifactSummary[];
  projects: ProjectSummary[];
  server_time: string;
}

export interface DemoIdentity {
  participant_id: string;
  display_name: string;
  device_id: string;
  public_key: string;
  private_key: string;
  label: "Development Demo Only";
}

export interface ProtocolEvent {
  protocol: "babel/1";
  id: string;
  schema: string;
  version: number;
  author_id: string;
  device_id: string;
  room_id?: string | null;
  created_at: string;
  client_sequence: number;
  parent_ids: string[];
  scope: "private" | "room" | "community" | "federated" | "public";
  expires_at?: string | null;
  content: Record<string, unknown>;
  attachments: unknown[];
  signature: string;
}

export type ClientEvent =
  | { type: "room.join"; payload: { participant_id: string } }
  | { type: "room.leave"; payload: { participant_id: string } }
  | {
      type: "typing.start" | "typing.stop";
      payload: { participant_id: string };
    }
  | {
      type: "message.send";
      payload: {
        event: ProtocolEvent;
        original_language: string;
        original_text: string;
        reply_to: string | null;
      };
    }
  | {
      type: "translation.review";
      payload: { message_id: string; reviewer_id: string; note: string };
    }
  | {
      type: "message.context_added";
      payload: {
        message_id: string;
        author_id: string;
        note_type: string;
        text: string;
      };
    }
  | {
      type: "repair.open";
      payload: {
        target_id: string;
        opened_by: string;
        reason: RepairThread["reason"];
        note: string;
      };
    }
  | {
      type: "repair.respond";
      payload: { repair_id: string; state: RepairThread["state"] };
    }
  | {
      type: "facilitator.request";
      payload: { requested_by: string; prompt: string };
    }
  | { type: "facilitator.reject"; payload: { response_id: string } }
  | { type: "artifact.propose"; payload: { requested_by: string } }
  | { type: "artifact.approve"; payload: { participant_id: string } }
  | { type: "artifact.publish"; payload: null }
  | { type: "project.create"; payload: null };

export type ServerEvent =
  | { type: "room.snapshot"; payload: RoomSnapshot }
  | { type: "room.participant_joined"; payload: { participant_id: string } }
  | { type: "room.participant_left"; payload: { participant_id: string } }
  | { type: "presence.updated"; payload: { participant_id: string; present: boolean } }
  | { type: "typing.updated"; payload: { participant_id: string; typing: boolean } }
  | { type: "message.accepted"; payload: Message }
  | { type: "message.delivered"; payload: { message_id: string } }
  | { type: "translation.started"; payload: { message_id: string } }
  | { type: "translation.completed"; payload: TranslationResult }
  | { type: "repair.updated"; payload: RepairThread }
  | { type: "message.context_added"; payload: CulturalContextNote }
  | { type: "facilitator.response"; payload: FacilitationResponse }
  | { type: "artifact.proposal_created"; payload: UnderstandingArtifactDraft }
  | { type: "artifact.updated"; payload: UnderstandingArtifactDraft }
  | { type: "commons.published"; payload: PublicArtifactSummary }
  | { type: "project.created"; payload: ProjectSummary }
  | { type: "error"; payload: { message: string } };
