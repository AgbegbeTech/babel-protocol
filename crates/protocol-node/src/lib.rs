use std::{
    collections::{HashMap, HashSet},
    env,
    net::SocketAddr,
    sync::Arc,
};

use axum::{
    extract::{
        ws::{Message as WsMessage, WebSocket},
        Path, Query, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use chrono::Utc;
use consent::{AttributionPreference, ConsentReceipt, PublicationScope};
use conversations::{CulturalContextNote, DeliveryState, Message, Room};
use facilitation::{
    FacilitationProvider, FacilitationRequest, FacilitationResponse, MockFacilitationProvider,
};
use futures::{SinkExt, StreamExt};
use identity::{generate_device_key_material, sign_event_with_private_key, DeviceKeyMaterial};
use protocol_core::{EventScope, ProtocolEvent, ReplayProtector, PROTOCOL_ID};
use realtime::{
    ArtifactApprovalInput, ClientEvent, CulturalContextInput, FacilitationInput, MessageSendInput,
    ProjectSummary, PublicArtifactSummary, RepairOpenInput, RepairTransitionInput, RoomSnapshot,
    ServerEvent, TranslationReviewInput,
};
use repair::RepairThread;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::sync::{broadcast, Mutex};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{info, warn};
use translation::{
    MockTranslationProvider, TranslationProvider, TranslationRequest, TranslationReviewStatus,
};
use understanding::{
    ArtifactLifecycle, InsightProposalInput, InsightProposalProvider, MockInsightProposalProvider,
    UnderstandingArtifactDraft,
};
use uuid::Uuid;

const DEMO_ROOM_ID: &str = "11111111-1111-4111-8111-111111111111";

#[derive(Clone)]
pub struct AppState {
    inner: Arc<Mutex<DemoState>>,
    broadcaster: broadcast::Sender<ServerEvent>,
    pool: Option<Arc<PgPool>>,
    translation_provider: Arc<dyn TranslationProvider>,
    facilitation_provider: Arc<dyn FacilitationProvider>,
    insight_provider: Arc<dyn InsightProposalProvider>,
    demo_devices: Arc<HashMap<String, DemoIdentity>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DemoIdentity {
    pub participant_id: String,
    pub display_name: String,
    pub device_id: String,
    pub public_key: String,
    pub private_key: String,
    pub label: String,
}

struct DemoState {
    room: Room,
    messages: Vec<Message>,
    repairs: Vec<RepairThread>,
    facilitator_responses: Vec<FacilitationResponse>,
    artifact: Option<UnderstandingArtifactDraft>,
    approvals: HashSet<String>,
    consent_receipts: Vec<ConsentReceipt>,
    commons_publications: Vec<PublicArtifactSummary>,
    projects: Vec<ProjectSummary>,
    replay: ReplayProtector,
}

impl Default for DemoState {
    fn default() -> Self {
        Self {
            room: Room::demo(),
            messages: Vec::new(),
            repairs: Vec::new(),
            facilitator_responses: Vec::new(),
            artifact: None,
            approvals: HashSet::new(),
            consent_receipts: Vec::new(),
            commons_publications: Vec::new(),
            projects: Vec::new(),
            replay: ReplayProtector::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AccessQuery {
    participant_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    status: &'static str,
    protocol: &'static str,
    local_mode: bool,
    database_url_configured: bool,
    cloudflare_required_for_local: bool,
}

pub async fn run() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "protocol_node=info,tower_http=info".into()),
        )
        .init();

    let bind = env::var("BABEL_NODE_BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let addr: SocketAddr = bind.parse()?;
    let state = AppState::connect_from_env().await?;
    let router = build_router(state);

    info!(%addr, "starting Babel Protocol local node");
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        warn!(%error, "failed to listen for shutdown signal");
    }
}

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/demo-identities", get(demo_identities))
        .route("/api/v1/identities", get(identities))
        .route("/api/v1/devices", get(devices))
        .route("/api/v1/rooms", get(rooms))
        .route("/api/v1/rooms/:room_id", get(room))
        .route("/api/v1/rooms/:room_id/messages", get(room_messages))
        .route("/api/v1/rooms/:room_id/history", get(room_history))
        .route("/api/v1/rooms/:room_id/repairs", get(room_repairs))
        .route(
            "/api/v1/rooms/:room_id/artifact-proposals",
            get(room_artifact),
        )
        .route("/api/v1/rooms/:room_id/ws", get(room_ws))
        .route("/api/v1/artifacts", get(artifacts))
        .route("/api/v1/artifacts/:id", get(artifact_by_id))
        .route("/api/v1/artifacts/:id/revise", get(not_implemented))
        .route("/api/v1/artifacts/:id/approve", get(not_implemented))
        .route("/api/v1/artifacts/:id/publish", get(not_implemented))
        .route("/api/v1/commons", get(commons))
        .route("/api/v1/commons/:id", get(commons_by_id))
        .route("/api/v1/projects", get(projects))
        .route("/api/v1/projects/:id", get(project_by_id))
        .route("/api/v1/verify/events/:id", get(verify_event))
        .route("/api/v1/verify/consent/:id", get(verify_consent))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

impl AppState {
    pub fn new() -> Self {
        Self::with_pool(None)
    }

    pub fn with_pool(pool: Option<PgPool>) -> Self {
        let (broadcaster, _) = broadcast::channel(256);
        let mut devices = HashMap::new();
        devices.insert(
            "did:babel:amara".to_string(),
            demo_identity("did:babel:amara", "Amara"),
        );
        devices.insert(
            "did:babel:diego".to_string(),
            demo_identity("did:babel:diego", "Diego"),
        );

        Self {
            inner: Arc::new(Mutex::new(DemoState::default())),
            broadcaster,
            pool: pool.map(Arc::new),
            translation_provider: Arc::new(MockTranslationProvider),
            facilitation_provider: Arc::new(MockFacilitationProvider),
            insight_provider: Arc::new(MockInsightProposalProvider),
            demo_devices: Arc::new(devices),
        }
    }

    pub async fn connect_from_env() -> anyhow::Result<Self> {
        if let Ok(database_url) = env::var("BABEL_DATABASE_URL") {
            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(&database_url)
                .await?;
            sqlx::migrate!("../../migrations").run(&pool).await?;
            let state = Self::with_pool(Some(pool));
            state.seed_demo_records().await?;
            Ok(state)
        } else {
            Ok(Self::new())
        }
    }

    async fn snapshot(&self) -> RoomSnapshot {
        let guard = self.inner.lock().await;
        RoomSnapshot {
            room: guard.room.clone(),
            messages: guard.messages.clone(),
            repairs: guard.repairs.clone(),
            facilitator_responses: guard.facilitator_responses.clone(),
            artifact: guard.artifact.clone(),
            approvals: guard.approvals.iter().cloned().collect(),
            consent_receipt_ids: guard
                .consent_receipts
                .iter()
                .map(|receipt| receipt.receipt_id.to_string())
                .collect(),
            commons_publications: guard.commons_publications.clone(),
            projects: guard.projects.clone(),
            server_time: Utc::now(),
        }
    }

    async fn is_participant(&self, participant_id: &str) -> bool {
        let guard = self.inner.lock().await;
        guard
            .room
            .participants
            .iter()
            .any(|participant| participant.id == participant_id)
    }

    fn demo_identity(&self, participant_id: &str) -> Option<DemoIdentity> {
        self.demo_devices.get(participant_id).cloned()
    }

    async fn seed_demo_records(&self) -> anyhow::Result<()> {
        let room = Room::demo();
        let Some(pool) = &self.pool else {
            return Ok(());
        };

        for identity in self.demo_devices.values() {
            sqlx::query(
                "INSERT INTO identities (id, display_name, preferred_languages, public_identity_key)
                 VALUES ($1, $2, $3, $4)
                 ON CONFLICT (id) DO NOTHING",
            )
            .bind(&identity.participant_id)
            .bind(&identity.display_name)
            .bind(vec!["en".to_string(), "es".to_string()])
            .bind(&identity.public_key)
            .execute(pool.as_ref())
            .await?;

            sqlx::query(
                "INSERT INTO devices (id, identity_id, display_name, public_key)
                 VALUES ($1, $2, $3, $4)
                 ON CONFLICT (id) DO NOTHING",
            )
            .bind(&identity.device_id)
            .bind(&identity.participant_id)
            .bind(format!("{} demo browser", identity.display_name))
            .bind(&identity.public_key)
            .execute(pool.as_ref())
            .await?;
        }

        sqlx::query(
            "INSERT INTO rooms (id, title, lifecycle, privacy, retention, artifact_proposal_allowed)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (id) DO NOTHING",
        )
        .bind(room.id)
        .bind(room.title)
        .bind("active")
        .bind(room.privacy)
        .bind(room.retention)
        .bind(room.artifact_proposal_allowed)
        .execute(pool.as_ref())
        .await?;

        for participant in room.participants {
            sqlx::query(
                "INSERT INTO room_participants (room_id, identity_id)
                 VALUES ($1, $2)
                 ON CONFLICT (room_id, identity_id) DO NOTHING",
            )
            .bind(room.id)
            .bind(&participant.id)
            .execute(pool.as_ref())
            .await?;

            sqlx::query(
                "INSERT INTO room_language_preferences
                 (room_id, identity_id, preferred_language, translation_target)
                 VALUES ($1, $2, $3, $4)
                 ON CONFLICT (room_id, identity_id)
                 DO UPDATE SET preferred_language = EXCLUDED.preferred_language,
                               translation_target = EXCLUDED.translation_target,
                               updated_at = now()",
            )
            .bind(room.id)
            .bind(participant.id)
            .bind(participant.preferred_language)
            .bind(participant.translation_target)
            .execute(pool.as_ref())
            .await?;
        }
        Ok(())
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

fn demo_identity(participant_id: &str, display_name: &str) -> DemoIdentity {
    let DeviceKeyMaterial {
        device_id,
        public_key,
        private_key,
    } = generate_device_key_material();
    DemoIdentity {
        participant_id: participant_id.to_string(),
        display_name: display_name.to_string(),
        device_id,
        public_key,
        private_key,
        label: "Development Demo Only".to_string(),
    }
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        protocol: PROTOCOL_ID,
        local_mode: true,
        database_url_configured: env::var("BABEL_DATABASE_URL").is_ok(),
        cloudflare_required_for_local: false,
    })
}

async fn demo_identities(State(state): State<AppState>) -> Json<Vec<DemoIdentity>> {
    Json(state.demo_devices.values().cloned().collect())
}

async fn identities(State(state): State<AppState>) -> Json<serde_json::Value> {
    let snapshot = state.snapshot().await;
    Json(json!({
        "identities": snapshot.room.participants.iter().map(|participant| {
            json!({
                "id": participant.id,
                "display_name": participant.display_name,
                "preferred_language": participant.preferred_language,
                "location_label": participant.location_label
            })
        }).collect::<Vec<_>>()
    }))
}

async fn devices(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({
        "devices": state.demo_devices.values().map(|identity| {
            json!({
                "participant_id": identity.participant_id,
                "device_id": identity.device_id,
                "public_key": identity.public_key,
                "demo_label": identity.label
            })
        }).collect::<Vec<_>>()
    }))
}

async fn rooms(State(state): State<AppState>) -> Json<serde_json::Value> {
    let snapshot = state.snapshot().await;
    Json(json!({ "rooms": [snapshot.room] }))
}

async fn room(
    Path(room_id): Path<Uuid>,
    Query(query): Query<AccessQuery>,
    State(state): State<AppState>,
) -> Response {
    if room_id.to_string() != DEMO_ROOM_ID {
        return not_found("room not found");
    }
    let Some(participant_id) = query.participant_id else {
        return forbidden("participant_id is required");
    };
    if !state.is_participant(&participant_id).await {
        return forbidden("room access is restricted to participants");
    }

    Json(state.snapshot().await).into_response()
}

async fn room_messages(
    Path(room_id): Path<Uuid>,
    Query(query): Query<AccessQuery>,
    State(state): State<AppState>,
) -> Response {
    protected_room_json(
        room_id,
        query,
        state,
        |snapshot| json!({ "messages": snapshot.messages }),
    )
    .await
}

async fn room_history(
    Path(room_id): Path<Uuid>,
    Query(query): Query<AccessQuery>,
    State(state): State<AppState>,
) -> Response {
    protected_room_json(room_id, query, state, |snapshot| {
        json!({
            "messages": snapshot.messages,
            "repairs": snapshot.repairs,
            "note": "Private history is returned only to room participants."
        })
    })
    .await
}

async fn room_repairs(
    Path(room_id): Path<Uuid>,
    Query(query): Query<AccessQuery>,
    State(state): State<AppState>,
) -> Response {
    protected_room_json(
        room_id,
        query,
        state,
        |snapshot| json!({ "repairs": snapshot.repairs }),
    )
    .await
}

async fn room_artifact(
    Path(room_id): Path<Uuid>,
    Query(query): Query<AccessQuery>,
    State(state): State<AppState>,
) -> Response {
    protected_room_json(
        room_id,
        query,
        state,
        |snapshot| json!({ "artifact": snapshot.artifact }),
    )
    .await
}

async fn artifacts(State(state): State<AppState>) -> Json<serde_json::Value> {
    let snapshot = state.snapshot().await;
    Json(json!({
        "artifacts": snapshot.artifact.map(|artifact| vec![artifact]).unwrap_or_default()
    }))
}

async fn artifact_by_id(Path(id): Path<String>, State(state): State<AppState>) -> Response {
    let snapshot = state.snapshot().await;
    match snapshot.artifact {
        Some(artifact) if artifact.id.to_string() == id => Json(artifact).into_response(),
        _ => not_found("artifact not found"),
    }
}

async fn commons(State(state): State<AppState>) -> Json<serde_json::Value> {
    let snapshot = state.snapshot().await;
    Json(json!({ "commons": snapshot.commons_publications }))
}

async fn commons_by_id(Path(id): Path<String>, State(state): State<AppState>) -> Response {
    let snapshot = state.snapshot().await;
    match snapshot
        .commons_publications
        .into_iter()
        .find(|publication| publication.id == id)
    {
        Some(publication) => Json(publication).into_response(),
        None => not_found("commons artifact not found"),
    }
}

async fn projects(State(state): State<AppState>) -> Json<serde_json::Value> {
    let snapshot = state.snapshot().await;
    Json(json!({ "projects": snapshot.projects }))
}

async fn project_by_id(Path(id): Path<String>, State(state): State<AppState>) -> Response {
    let snapshot = state.snapshot().await;
    match snapshot
        .projects
        .into_iter()
        .find(|project| project.id == id)
    {
        Some(project) => Json(project).into_response(),
        None => not_found("project not found"),
    }
}

async fn verify_event(Path(id): Path<String>, State(state): State<AppState>) -> Response {
    let snapshot = state.snapshot().await;
    match snapshot
        .messages
        .into_iter()
        .find(|message| message.id == id)
    {
        Some(message) => Json(json!({
            "event_id": message.id,
            "event_hash": message.event_hash,
            "signature_present": !message.signature.is_empty(),
            "original_message_preserved": true
        }))
        .into_response(),
        None => not_found("event not found"),
    }
}

async fn verify_consent(Path(id): Path<String>, State(state): State<AppState>) -> Response {
    let guard = state.inner.lock().await;
    match guard
        .consent_receipts
        .iter()
        .find(|receipt| receipt.receipt_id.to_string() == id)
    {
        Some(receipt) => Json(json!({
            "receipt_id": receipt.receipt_id,
            "artifact_id": receipt.artifact_id,
            "revision_hash": receipt.exact_revision_hash,
            "scope": receipt.approved_publication_scope,
            "signature_present": !receipt.signature.is_empty()
        }))
        .into_response(),
        None => not_found("consent receipt not found"),
    }
}

async fn not_implemented() -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "This endpoint is reserved for the protocol surface. Use the WebSocket demo flow in v0.1."
        })),
    )
        .into_response()
}

async fn protected_room_json(
    room_id: Uuid,
    query: AccessQuery,
    state: AppState,
    build: impl FnOnce(RoomSnapshot) -> serde_json::Value,
) -> Response {
    if room_id.to_string() != DEMO_ROOM_ID {
        return not_found("room not found");
    }
    let Some(participant_id) = query.participant_id else {
        return forbidden("participant_id is required");
    };
    if !state.is_participant(&participant_id).await {
        return forbidden("room access is restricted to participants");
    }
    Json(build(state.snapshot().await)).into_response()
}

async fn room_ws(
    Path(room_id): Path<Uuid>,
    Query(query): Query<AccessQuery>,
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> Response {
    if room_id.to_string() != DEMO_ROOM_ID {
        return not_found("room not found");
    }
    let Some(participant_id) = query.participant_id else {
        return forbidden("participant_id is required");
    };
    if !state.is_participant(&participant_id).await {
        return forbidden("room access is restricted to participants");
    }

    ws.on_upgrade(move |socket| handle_socket(socket, state, room_id, participant_id))
}

async fn handle_socket(socket: WebSocket, state: AppState, room_id: Uuid, participant_id: String) {
    let (mut sender, mut receiver) = socket.split();
    let snapshot_event = ServerEvent::RoomSnapshot(Box::new(state.snapshot().await));
    if sender
        .send(WsMessage::Text(
            serde_json::to_string(&snapshot_event).expect("snapshot serializes"),
        ))
        .await
        .is_err()
    {
        return;
    }

    set_presence(&state, &participant_id, true).await;
    broadcast(
        &state,
        ServerEvent::ParticipantJoined {
            participant_id: participant_id.clone(),
        },
    );

    let mut rx = state.broadcaster.subscribe();
    let send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            let Ok(text) = serde_json::to_string(&event) else {
                continue;
            };
            if sender.send(WsMessage::Text(text)).await.is_err() {
                break;
            }
        }
    });

    while let Some(message) = receiver.next().await {
        match message {
            Ok(WsMessage::Text(text)) => {
                match serde_json::from_str::<ClientEvent>(&text) {
                    Ok(event) => process_client_event(&state, room_id, event).await,
                    Err(_) => broadcast(
                        &state,
                        ServerEvent::Error {
                            message: "Invalid WebSocket event".to_string(),
                        },
                    ),
                };
            }
            Ok(WsMessage::Close(_)) => break,
            Ok(_) => {}
            Err(_) => break,
        }
    }

    send_task.abort();
    set_presence(&state, &participant_id, false).await;
    broadcast(&state, ServerEvent::ParticipantLeft { participant_id });
}

async fn process_client_event(state: &AppState, room_id: Uuid, event: ClientEvent) {
    match event {
        ClientEvent::RoomJoin { participant_id } => {
            set_presence(state, &participant_id, true).await;
        }
        ClientEvent::RoomLeave { participant_id } => {
            set_presence(state, &participant_id, false).await;
        }
        ClientEvent::PresenceUpdate {
            participant_id,
            present,
        } => {
            set_presence(state, &participant_id, present).await;
        }
        ClientEvent::TypingStart { participant_id } => {
            set_typing(state, &participant_id, true).await;
        }
        ClientEvent::TypingStop { participant_id } => {
            set_typing(state, &participant_id, false).await;
        }
        ClientEvent::MessageSend(input) => {
            handle_message_send(state, room_id, *input).await;
        }
        ClientEvent::TranslationReview(input) => {
            handle_translation_review(state, input).await;
        }
        ClientEvent::CulturalContext(input) => {
            handle_context_added(state, input).await;
        }
        ClientEvent::RepairOpen(input) => {
            handle_repair_open(state, room_id, input).await;
        }
        ClientEvent::RepairTransition(input) => {
            handle_repair_transition(state, input).await;
        }
        ClientEvent::FacilitatorRequest(input) => {
            handle_facilitation(state, room_id, input).await;
        }
        ClientEvent::FacilitatorReject { response_id } => {
            handle_facilitation_reject(state, &response_id).await;
        }
        ClientEvent::ArtifactPropose { requested_by } => {
            handle_artifact_propose(state, room_id, requested_by).await;
        }
        ClientEvent::ArtifactApprove(input) => {
            handle_artifact_approve(state, input).await;
        }
        ClientEvent::ArtifactPublish => {
            handle_artifact_publish(state).await;
        }
        ClientEvent::ProjectCreate => {
            handle_project_create(state).await;
        }
    }
}

async fn handle_message_send(state: &AppState, room_id: Uuid, input: MessageSendInput) {
    let Some(identity) = state.demo_identity(&input.event.author_id) else {
        broadcast_error(state, "Unknown author");
        return;
    };
    if input.event.room_id != Some(room_id) {
        broadcast_error(state, "Event room does not match WebSocket room");
        return;
    }
    if input.event.schema != "babel.message.created/1" {
        broadcast_error(state, "Unsupported message schema");
        return;
    }
    if input
        .event
        .verify_with_public_key_b64(&identity.public_key)
        .is_err()
    {
        broadcast_error(state, "Message signature could not be verified");
        return;
    }

    let mut event = input.event;
    if event
        .content
        .get("original_text")
        .and_then(|value| value.as_str())
        != Some(input.original_text.as_str())
    {
        broadcast_error(state, "Signed content does not match message body");
        return;
    }

    let event_hash = match event.content_hash() {
        Ok(hash) => hash,
        Err(_) => {
            broadcast_error(state, "Event hash failed");
            return;
        }
    };

    {
        let mut guard = state.inner.lock().await;
        if let Err(error) = guard.replay.accept(&event) {
            broadcast_error(state, &error.to_string());
            return;
        }
    }

    let message = Message {
        id: event.id.clone(),
        room_id,
        sender_id: event.author_id.clone(),
        sender_device_id: event.device_id.clone(),
        original_language: input.original_language.clone(),
        original_text: input.original_text.clone(),
        sent_at: event.created_at,
        client_sequence: event.client_sequence,
        reply_to: input.reply_to.clone(),
        signature: event.signature.clone(),
        event_hash,
        delivery_state: DeliveryState::Persisted,
        translations: Vec::new(),
        context_notes: Vec::new(),
    };

    if let Err(error) = persist_message(state, &event, &message).await {
        warn!(%error, "message persistence failed before broadcast");
        broadcast_error(state, "Message could not be durably persisted");
        return;
    }

    {
        let mut guard = state.inner.lock().await;
        guard.messages.push(message.clone());
    }
    broadcast(
        state,
        ServerEvent::MessageAccepted(Box::new(message.clone())),
    );
    broadcast(
        state,
        ServerEvent::MessageDelivered {
            message_id: message.id.clone(),
        },
    );
    broadcast(
        state,
        ServerEvent::TranslationStarted {
            message_id: message.id.clone(),
        },
    );

    let target_language = target_language_for_message(state, &message.sender_id).await;
    match state
        .translation_provider
        .translate(TranslationRequest {
            message_id: message.id.clone(),
            source_language: input.original_language,
            target_language,
            original_text: input.original_text,
        })
        .await
    {
        Ok(translation) => {
            if let Err(error) = persist_translation(state, &translation).await {
                warn!(%error, "translation persistence failed");
            }
            let mut guard = state.inner.lock().await;
            if let Some(stored) = guard
                .messages
                .iter_mut()
                .find(|stored| stored.id == translation.message_id)
            {
                stored.delivery_state = DeliveryState::Translated;
                stored.translations.push(translation.clone());
            }
            drop(guard);
            broadcast(
                state,
                ServerEvent::TranslationCompleted(Box::new(translation)),
            );
        }
        Err(_) => broadcast_error(state, "Translation failed"),
    }

    event.signature.clear();
}

async fn persist_message(
    state: &AppState,
    event: &ProtocolEvent,
    message: &Message,
) -> anyhow::Result<()> {
    let Some(pool) = &state.pool else {
        return Ok(());
    };
    sqlx::query(
        "INSERT INTO protocol_events
         (id, protocol, schema, version, author_id, device_id, room_id, created_at,
          client_sequence, parent_ids, scope, expires_at, content, attachments, signature, event_hash)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)",
    )
    .bind(&event.id)
    .bind(&event.protocol)
    .bind(&event.schema)
    .bind(event.version as i32)
    .bind(&event.author_id)
    .bind(&event.device_id)
    .bind(event.room_id)
    .bind(event.created_at)
    .bind(event.client_sequence as i64)
    .bind(&event.parent_ids)
    .bind(format!("{:?}", event.scope).to_lowercase())
    .bind(event.expires_at)
    .bind(&event.content)
    .bind(serde_json::to_value(&event.attachments)?)
    .bind(&event.signature)
    .bind(&message.event_hash)
    .execute(pool.as_ref())
    .await?;

    sqlx::query(
        "INSERT INTO messages
         (id, room_id, sender_id, sender_device_id, original_language, original_text,
          sent_at, client_sequence, reply_to, signature, event_hash, delivery_state)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
    )
    .bind(&message.id)
    .bind(message.room_id)
    .bind(&message.sender_id)
    .bind(&message.sender_device_id)
    .bind(&message.original_language)
    .bind(&message.original_text)
    .bind(message.sent_at)
    .bind(message.client_sequence as i64)
    .bind(&message.reply_to)
    .bind(&message.signature)
    .bind(&message.event_hash)
    .bind("persisted")
    .execute(pool.as_ref())
    .await?;

    sqlx::query(
        "INSERT INTO outbox_events (aggregate_type, aggregate_id, event_type, payload)
         VALUES ($1, $2, $3, $4)",
    )
    .bind("message")
    .bind(&message.id)
    .bind("babel.message.created/1")
    .bind(serde_json::to_value(message)?)
    .execute(pool.as_ref())
    .await?;

    Ok(())
}

async fn persist_translation(
    state: &AppState,
    translation: &translation::TranslationResult,
) -> anyhow::Result<()> {
    let Some(pool) = &state.pool else {
        return Ok(());
    };

    sqlx::query(
        "INSERT INTO translations
         (message_id, source_language, target_language, translated_text, provider, confidence,
          uncertain_phrases, literal_alternative, cultural_notes, review_status, stream_state, generated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
    )
    .bind(&translation.message_id)
    .bind(&translation.source_language)
    .bind(&translation.target_language)
    .bind(&translation.translated_text)
    .bind(&translation.provider)
    .bind(translation.confidence)
    .bind(&translation.uncertain_phrases)
    .bind(&translation.literal_alternative)
    .bind(&translation.cultural_notes)
    .bind(format!("{:?}", translation.review_status).to_lowercase())
    .bind(format!("{:?}", translation.stream_state).to_lowercase())
    .bind(translation.generated_at)
    .execute(pool.as_ref())
    .await?;
    Ok(())
}

async fn handle_translation_review(state: &AppState, input: TranslationReviewInput) {
    let mut updated = None;
    {
        let mut guard = state.inner.lock().await;
        for message in &mut guard.messages {
            if message.id == input.message_id {
                for translation in &mut message.translations {
                    translation.review_status = TranslationReviewStatus::Challenged;
                    translation
                        .cultural_notes
                        .push(format!("{}: {}", input.reviewer_id, input.note));
                    updated = Some(translation.clone());
                }
                message.delivery_state = DeliveryState::TranslationReviewed;
            }
        }
    }
    if let Some(translation) = updated {
        broadcast(
            state,
            ServerEvent::TranslationCompleted(Box::new(translation)),
        );
    }
}

async fn handle_context_added(state: &AppState, input: CulturalContextInput) {
    let note = CulturalContextNote {
        id: format!("context-{}", Utc::now().timestamp_micros()),
        message_id: input.message_id.clone(),
        author_id: input.author_id,
        note_type: input.note_type,
        text: input.text,
        created_at: Utc::now(),
    };
    {
        let mut guard = state.inner.lock().await;
        if let Some(message) = guard
            .messages
            .iter_mut()
            .find(|message| message.id == input.message_id)
        {
            message.context_notes.push(note.clone());
        }
    }
    broadcast(state, ServerEvent::ContextAdded(note));
}

async fn handle_repair_open(state: &AppState, room_id: Uuid, input: RepairOpenInput) {
    let repair = RepairThread::open(
        room_id,
        input.target_id,
        input.opened_by,
        input.reason,
        input.note,
    );
    {
        let mut guard = state.inner.lock().await;
        guard.repairs.push(repair.clone());
        if let Some(artifact) = &mut guard.artifact {
            artifact.lifecycle = ArtifactLifecycle::UnderReview;
        }
    }
    broadcast(state, ServerEvent::RepairUpdated(repair));
}

async fn handle_repair_transition(state: &AppState, input: RepairTransitionInput) {
    let Ok(repair_id) = Uuid::parse_str(&input.repair_id) else {
        broadcast_error(state, "Invalid repair id");
        return;
    };
    let mut updated = None;
    {
        let mut guard = state.inner.lock().await;
        if let Some(repair) = guard
            .repairs
            .iter_mut()
            .find(|repair| repair.id == repair_id)
        {
            repair.transition(input.state);
            updated = Some(repair.clone());
        }
    }
    if let Some(repair) = updated {
        broadcast(state, ServerEvent::RepairUpdated(repair));
    }
}

async fn handle_facilitation(state: &AppState, room_id: Uuid, input: FacilitationInput) {
    let visible_context = {
        let guard = state.inner.lock().await;
        guard
            .messages
            .iter()
            .rev()
            .take(4)
            .map(|message| {
                format!(
                    "{} said in {}. Translation count: {}",
                    message.sender_id,
                    message.original_language,
                    message.translations.len()
                )
            })
            .collect::<Vec<_>>()
    };

    match state
        .facilitation_provider
        .assist(FacilitationRequest {
            room_id: room_id.to_string(),
            requested_by: input.requested_by,
            prompt: input.prompt,
            visible_context,
        })
        .await
    {
        Ok(response) => {
            {
                let mut guard = state.inner.lock().await;
                guard.facilitator_responses.push(response.clone());
            }
            broadcast(state, ServerEvent::FacilitatorResponse(response));
        }
        Err(_) => broadcast_error(state, "Facilitation failed"),
    }
}

async fn handle_facilitation_reject(state: &AppState, response_id: &str) {
    let mut updated = None;
    {
        let mut guard = state.inner.lock().await;
        if let Some(response) = guard
            .facilitator_responses
            .iter_mut()
            .find(|response| response.id == response_id)
        {
            response.accepted = Some(false);
            updated = Some(response.clone());
        }
    }
    if let Some(response) = updated {
        broadcast(state, ServerEvent::FacilitatorResponse(response));
    }
}

async fn handle_artifact_propose(state: &AppState, room_id: Uuid, requested_by: String) {
    let participant_ids = {
        let guard = state.inner.lock().await;
        guard
            .room
            .participants
            .iter()
            .map(|participant| participant.id.clone())
            .collect::<Vec<_>>()
    };
    match state
        .insight_provider
        .propose(InsightProposalInput {
            room_id,
            requested_by,
            participant_ids,
            visible_context: vec!["Private transcript remains private.".to_string()],
        })
        .await
    {
        Ok(mut artifact) => {
            artifact.lifecycle = ArtifactLifecycle::Proposed;
            {
                let mut guard = state.inner.lock().await;
                guard.approvals.clear();
                guard.artifact = Some(artifact.clone());
            }
            broadcast(
                state,
                ServerEvent::ArtifactProposalCreated(Box::new(artifact)),
            );
            broadcast(
                state,
                ServerEvent::RoomSnapshot(Box::new(state.snapshot().await)),
            );
        }
        Err(_) => broadcast_error(state, "Artifact proposal failed"),
    }
}

async fn handle_artifact_approve(state: &AppState, input: ArtifactApprovalInput) {
    let mut updated = None;
    {
        let mut guard = state.inner.lock().await;
        guard.approvals.insert(input.participant_id.clone());
        let approvals = guard.approvals.clone();
        let mut receipt = None;
        if let Some(artifact) = &mut guard.artifact {
            if artifact
                .required_approvers
                .iter()
                .all(|approver| approvals.contains(approver))
            {
                artifact.lifecycle = ArtifactLifecycle::Approved;
            } else {
                artifact.lifecycle = ArtifactLifecycle::UnderReview;
            }

            receipt = Some(ConsentReceipt {
                receipt_id: Uuid::new_v4(),
                artifact_id: artifact.id,
                exact_revision_hash: artifact.revision_hash.clone(),
                approving_participant: input.participant_id.clone(),
                approving_device: state
                    .demo_identity(&input.participant_id)
                    .map(|identity| identity.device_id)
                    .unwrap_or_else(|| "unknown-device".to_string()),
                approved_publication_scope: PublicationScope::Commons,
                attribution_preference: AttributionPreference::Named,
                approved_translations: vec!["en".to_string(), "es".to_string()],
                ai_processing_permissions: vec!["artifact_proposal".to_string()],
                timestamp: Utc::now(),
                optional_review_date: None,
                consent_statement_version: "babel-consent/1".to_string(),
                signature: format!("demo-consent-signature-{}", input.participant_id),
            });
            updated = Some(artifact.clone());
        }
        if let Some(receipt) = receipt {
            guard.consent_receipts.push(receipt);
        }
    }
    if let Some(artifact) = updated {
        broadcast(state, ServerEvent::ArtifactUpdated(Box::new(artifact)));
        broadcast(
            state,
            ServerEvent::RoomSnapshot(Box::new(state.snapshot().await)),
        );
    }
}

async fn handle_artifact_publish(state: &AppState) {
    let publication = {
        let mut guard = state.inner.lock().await;
        let Some(artifact) = &mut guard.artifact else {
            broadcast_error(state, "No artifact exists");
            return;
        };
        if artifact.lifecycle != ArtifactLifecycle::Approved {
            broadcast_error(
                state,
                "All required participants must approve the exact revision first",
            );
            return;
        }
        artifact.lifecycle = ArtifactLifecycle::Published;
        let summary = PublicArtifactSummary {
            id: format!("commons-{}", artifact.id),
            title: artifact.title.clone(),
            summary: artifact.shared_summary.clone(),
            revision_hash: artifact.revision_hash.clone(),
            consent_verified: true,
            transcript_exposed: false,
        };
        guard.commons_publications.push(summary.clone());
        summary
    };
    broadcast(state, ServerEvent::CommonsPublished(publication));
    broadcast(
        state,
        ServerEvent::RoomSnapshot(Box::new(state.snapshot().await)),
    );
}

async fn handle_project_create(state: &AppState) {
    let project = {
        let mut guard = state.inner.lock().await;
        let Some(publication) = guard.commons_publications.last() else {
            broadcast_error(
                state,
                "Publish an approved artifact before creating a project",
            );
            return;
        };
        let summary = ProjectSummary {
            id: format!("project-{}", Uuid::new_v4()),
            title: "Community-led clean-water coordination".to_string(),
            status: "forming".to_string(),
            source_artifact_id: publication.id.clone(),
            contribution_needs: vec![
                "translation".to_string(),
                "engineering".to_string(),
                "local_context".to_string(),
                "care_work".to_string(),
            ],
        };
        guard.projects.push(summary.clone());
        summary
    };
    broadcast(state, ServerEvent::ProjectCreated(project));
    broadcast(
        state,
        ServerEvent::RoomSnapshot(Box::new(state.snapshot().await)),
    );
}

async fn set_presence(state: &AppState, participant_id: &str, present: bool) {
    {
        let mut guard = state.inner.lock().await;
        if let Some(participant) = guard
            .room
            .participants
            .iter_mut()
            .find(|participant| participant.id == participant_id)
        {
            participant.present = present;
        }
    }
    broadcast(
        state,
        ServerEvent::PresenceUpdated {
            participant_id: participant_id.to_string(),
            present,
        },
    );
}

async fn set_typing(state: &AppState, participant_id: &str, typing: bool) {
    {
        let mut guard = state.inner.lock().await;
        if let Some(participant) = guard
            .room
            .participants
            .iter_mut()
            .find(|participant| participant.id == participant_id)
        {
            participant.typing = typing;
        }
    }
    broadcast(
        state,
        ServerEvent::TypingUpdated {
            participant_id: participant_id.to_string(),
            typing,
        },
    );

    if typing {
        let cloned = state.clone();
        let participant = participant_id.to_string();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(4)).await;
            expire_typing(cloned, participant).await;
        });
    }
}

async fn expire_typing(state: AppState, participant_id: String) {
    {
        let mut guard = state.inner.lock().await;
        if let Some(participant) = guard
            .room
            .participants
            .iter_mut()
            .find(|participant| participant.id == participant_id)
        {
            participant.typing = false;
        }
    }
    broadcast(
        &state,
        ServerEvent::TypingUpdated {
            participant_id,
            typing: false,
        },
    );
}

async fn target_language_for_message(state: &AppState, sender_id: &str) -> String {
    let guard = state.inner.lock().await;
    guard
        .room
        .participants
        .iter()
        .find(|participant| participant.id != sender_id)
        .map(|participant| participant.translation_target.clone())
        .unwrap_or_else(|| "en".to_string())
}

fn broadcast(state: &AppState, event: ServerEvent) {
    let _ = state.broadcaster.send(event);
}

fn broadcast_error(state: &AppState, message: &str) {
    broadcast(
        state,
        ServerEvent::Error {
            message: message.to_string(),
        },
    );
}

fn forbidden(message: &str) -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({
            "error": message,
            "privacy_boundary": "room content is restricted to participants"
        })),
    )
        .into_response()
}

fn not_found(message: &str) -> Response {
    (StatusCode::NOT_FOUND, Json(json!({ "error": message }))).into_response()
}

pub fn demo_signed_message_event(
    state: &AppState,
    participant_id: &str,
    room_id: Uuid,
    sequence: u64,
    original_text: &str,
) -> ProtocolEvent {
    let identity = state
        .demo_identity(participant_id)
        .expect("demo participant exists");
    let mut event = ProtocolEvent::new(
        "babel.message.created/1",
        participant_id,
        identity.device_id.clone(),
        Some(room_id),
        sequence,
        EventScope::Room,
        json!({ "original_text": original_text }),
    );
    sign_event_with_private_key(&mut event, &identity.private_key).expect("demo event signs");
    event
}

#[cfg(test)]
mod tests {
    use super::{demo_signed_message_event, process_client_event, AppState};
    use realtime::{ClientEvent, MessageSendInput};
    use uuid::Uuid;

    #[tokio::test]
    async fn accepted_message_is_persisted_before_translation() {
        let state = AppState::new();
        let room_id =
            Uuid::parse_str("11111111-1111-4111-8111-111111111111").expect("valid room id");
        let text = "Local knowledge should stay with the community.";
        let event = demo_signed_message_event(&state, "did:babel:amara", room_id, 1, text);

        process_client_event(
            &state,
            room_id,
            ClientEvent::MessageSend(Box::new(MessageSendInput {
                event,
                original_language: "en".to_string(),
                original_text: text.to_string(),
                reply_to: None,
            })),
        )
        .await;

        let snapshot = state.snapshot().await;
        assert_eq!(snapshot.messages.len(), 1);
        assert_eq!(snapshot.messages[0].original_text, text);
        assert_eq!(snapshot.messages[0].translations.len(), 1);
    }

    #[tokio::test]
    async fn duplicate_sequence_is_rejected() {
        let state = AppState::new();
        let room_id =
            Uuid::parse_str("11111111-1111-4111-8111-111111111111").expect("valid room id");
        let text = "One message.";
        let first = demo_signed_message_event(&state, "did:babel:amara", room_id, 1, text);
        let second = demo_signed_message_event(&state, "did:babel:amara", room_id, 1, text);

        for event in [first, second] {
            process_client_event(
                &state,
                room_id,
                ClientEvent::MessageSend(Box::new(MessageSendInput {
                    event,
                    original_language: "en".to_string(),
                    original_text: text.to_string(),
                    reply_to: None,
                })),
            )
            .await;
        }

        assert_eq!(state.snapshot().await.messages.len(), 1);
    }
}
