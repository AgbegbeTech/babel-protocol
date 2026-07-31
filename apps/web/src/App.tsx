import { useState } from "react";
import type { ReactNode } from "react";
import {
  Activity,
  AlertTriangle,
  AudioWaveform,
  Bell,
  BookOpen,
  BrainCircuit,
  CheckCircle2,
  CircleDot,
  FileCheck2,
  Globe2,
  History,
  Languages,
  Mic,
  PanelRight,
  PenLine,
  Radio,
  Send,
  Settings,
  ShieldCheck,
  Sparkles,
  UserRound,
  Wrench,
  XCircle,
} from "lucide-react";
import { useBabelRoom } from "./lib/useBabelRoom";
import { publicCommonsHasNoTranscript } from "./lib/reducer";
import type {
  DemoIdentity,
  FacilitationResponse,
  Message,
  Participant,
  RepairThread,
  RoomSnapshot,
} from "./types";

const navItems = ["Conversations", "Understanding", "The Commons", "Projects", "Profile"];
const toolbar = [
  { label: "Voice", icon: Mic },
  { label: "Transcribe", icon: Languages },
  { label: "Whiteboard", icon: PenLine },
  { label: "Insights", icon: Activity },
  { label: "History", icon: History },
  { label: "Settings", icon: Settings },
];

export default function App() {
  const room = useBabelRoom();
  const [activeNav, setActiveNav] = useState("Conversations");
  const [activeTool, setActiveTool] = useState("Insights");

  if (!room.snapshot) {
    return (
      <main className="boot-screen">
        <div className="brand-mark">BABEL</div>
        <p>Connecting to the local Babel node...</p>
      </main>
    );
  }

  return (
    <main className="hud-shell">
      <AmbientLayer />
      <TopNav
        activeNav={activeNav}
        setActiveNav={setActiveNav}
        connectionState={room.connectionState}
        activeIdentity={room.activeIdentity}
      />

      <section className="hud-grid" aria-label="Babel live conversation workspace">
        <LeftRail snapshot={room.snapshot} />

        <section className="conversation-stage" aria-label="Live conversation">
          {activeNav === "Conversations" ? (
            <ConversationRoom
              snapshot={room.snapshot}
              identities={room.identities}
              activeParticipantId={room.activeParticipantId}
              setActiveParticipantId={room.setActiveParticipantId}
              draft={room.draft}
              setDraft={room.setDraft}
              sendTyping={room.sendTyping}
              sendMessage={room.sendMessage}
              sendSeed={room.sendSeed}
              challengeTranslation={room.challengeTranslation}
              addCulturalContext={room.addCulturalContext}
              openRepair={room.openRepair}
            />
          ) : (
            <SectionView
              section={activeNav}
              snapshot={room.snapshot}
              proposeArtifact={room.proposeArtifact}
              approveArtifact={room.approveArtifact}
              publishArtifact={room.publishArtifact}
              createProject={room.createProject}
            />
          )}
        </section>

        <RightRail
          snapshot={room.snapshot}
          inviteFacilitator={room.inviteFacilitator}
          rejectFacilitator={room.rejectFacilitator}
          transitionRepair={room.transitionRepair}
          proposeArtifact={room.proposeArtifact}
          approveArtifact={room.approveArtifact}
          publishArtifact={room.publishArtifact}
          createProject={room.createProject}
        />

        <aside className="tool-strip" aria-label="Room tools">
          {toolbar.map(({ label, icon: Icon }) => (
            <button
              className={activeTool === label ? "tool-button active" : "tool-button"}
              key={label}
              type="button"
              onClick={() => setActiveTool(label)}
              title={label}
            >
              <Icon size={23} aria-hidden="true" />
              <span>{label}</span>
            </button>
          ))}
        </aside>
      </section>

      <WorldBand snapshot={room.snapshot} />
    </main>
  );
}

function TopNav({
  activeNav,
  setActiveNav,
  connectionState,
  activeIdentity,
}: {
  activeNav: string;
  setActiveNav: (value: string) => void;
  connectionState: string;
  activeIdentity: DemoIdentity | null;
}) {
  return (
    <header className="top-nav">
      <div className="logo-block" aria-label="Babel live conversation">
        <strong>BABEL</strong>
        <span>LIVE CONVERSATION</span>
      </div>
      <nav aria-label="Primary">
        {navItems.map((item) => (
          <button
            key={item}
            className={activeNav === item ? "nav-tab active" : "nav-tab"}
            type="button"
            onClick={() => setActiveNav(item)}
          >
            {item}
          </button>
        ))}
      </nav>
      <div className="top-status" aria-label="Network and profile status">
        <Globe2 size={21} />
        <AudioWaveform size={22} />
        <Bell size={21} />
        <span className="profile-chip">
          <UserRound size={16} />
          {activeIdentity?.display_name ?? "Amara"}
        </span>
        <span className={`connection-dot ${connectionState}`} />
      </div>
    </header>
  );
}

function LeftRail({ snapshot }: { snapshot: RoomSnapshot }) {
  const participantCount = snapshot.room.participants.length;
  const messageCount = snapshot.messages.length;

  return (
    <aside className="left-rail">
      <HudPanel title="Room Status" status="LIVE">
        <div className="room-status">
          <div className="pulse-orbit" aria-hidden="true">
            <span />
          </div>
          <div>
            <strong>Global Dialogue Room</strong>
            <span>Room ID: GDR-77F3</span>
            <small>{participantCount} participants</small>
          </div>
        </div>
        <AvatarStack participants={snapshot.room.participants} />
      </HudPanel>

      <HudPanel title="Translation Confidence">
        <div className="confidence">
          <div className="confidence-ring">
            <span>94%</span>
          </div>
          <div className="confidence-copy">
            <span>Overall Confidence</span>
            <strong>High</strong>
            <WaveBars />
          </div>
        </div>
        {[
          ["Spanish -> English", "96%"],
          ["English -> Spanish", "93%"],
          ["Spanish -> French", "91%"],
          ["French -> English", "90%"],
        ].map(([pair, score]) => (
          <div className="score-row" key={pair}>
            <span>{pair}</span>
            <strong>{score}</strong>
          </div>
        ))}
      </HudPanel>

      <HudPanel title="Active Languages">
        <div className="language-chips">
          {["Spanish", "English", "French", "Portuguese"].map((language) => (
            <span key={language}>{language}</span>
          ))}
        </div>
      </HudPanel>

      <HudPanel title="Room Intelligence">
        <div className="globe-widget">
          <div className="wire-globe" aria-hidden="true" />
          <div className="intelligence-readout">
            <span>Conversation Flow</span>
            <strong>Balanced</strong>
            <span>Understanding</span>
            <strong>Strengthening</strong>
            <span>Cultural Sensitivity</span>
            <strong>Optimal</strong>
          </div>
        </div>
        <small>{messageCount} accepted messages in this private room</small>
      </HudPanel>
    </aside>
  );
}

function ConversationRoom({
  snapshot,
  identities,
  activeParticipantId,
  setActiveParticipantId,
  draft,
  setDraft,
  sendTyping,
  sendMessage,
  sendSeed,
  challengeTranslation,
  addCulturalContext,
  openRepair,
}: {
  snapshot: RoomSnapshot;
  identities: DemoIdentity[];
  activeParticipantId: string;
  setActiveParticipantId: (id: string) => void;
  draft: string;
  setDraft: (value: string) => void;
  sendTyping: (typing: boolean) => void;
  sendMessage: () => Promise<void>;
  sendSeed: () => void;
  challengeTranslation: (messageId: string) => void;
  addCulturalContext: (messageId: string) => void;
  openRepair: (messageId: string) => void;
}) {
  const amara = snapshot.room.participants[0];
  const diego = snapshot.room.participants[1];
  const typingParticipant = snapshot.room.participants.find(
    (participant) => participant.typing && participant.id !== activeParticipantId,
  );

  return (
    <>
      <div className="participant-bridge">
        <ParticipantHeader participant={amara} side="left" />
        <div className="bridge-core" aria-hidden="true">
          <WaveBars />
          <div className="babel-emblem">A</div>
          <WaveBars />
        </div>
        <ParticipantHeader participant={diego} side="right" />
      </div>

      <div className="demo-switcher">
        <span>Development Demo Only</span>
        {identities.map((identity) => (
          <button
            key={identity.participant_id}
            className={identity.participant_id === activeParticipantId ? "active" : ""}
            type="button"
            onClick={() => setActiveParticipantId(identity.participant_id)}
          >
            {identity.display_name}
          </button>
        ))}
        <button type="button" onClick={sendSeed}>
          Send demo line
        </button>
      </div>

      <div className="message-stream" aria-live="polite">
        {snapshot.messages.length === 0 ? (
          <div className="empty-stream">
            <Languages size={38} />
            <strong>Private live room ready</strong>
            <span>Start with the demo line or write your own message.</span>
          </div>
        ) : (
          snapshot.messages.map((message) => (
            <MessageBubble
              key={message.id}
              message={message}
              participant={participantById(snapshot, message.sender_id)}
              mine={message.sender_id === activeParticipantId}
              challengeTranslation={challengeTranslation}
              addCulturalContext={addCulturalContext}
              openRepair={openRepair}
            />
          ))
        )}
        {typingParticipant ? (
          <div className="typing-indicator">
            <Avatar participant={typingParticipant} />
            <span>{typingParticipant.display_name} is typing...</span>
            <i />
            <i />
            <i />
          </div>
        ) : null}
      </div>

      <form
        className="composer"
        onSubmit={(event) => {
          event.preventDefault();
          void sendMessage();
        }}
      >
        <button type="button" className="icon-button" title="Microphone">
          <Mic size={20} />
        </button>
        <input
          value={draft}
          onBlur={() => sendTyping(false)}
          onChange={(event) => {
            setDraft(event.target.value);
            sendTyping(true);
          }}
          placeholder="Type your message..."
          aria-label="Message text"
        />
        <span className="language-toggle">EN</span>
        <span className="language-toggle">ES</span>
        <button type="submit" className="send-button" title="Send message">
          <Send size={24} />
        </button>
      </form>

      <div className="translation-status">
        <span />
        REALTIME TRANSLATION ON
      </div>
    </>
  );
}

function ParticipantHeader({ participant, side }: { participant: Participant; side: "left" | "right" }) {
  return (
    <div className={`participant-header ${side}`}>
      <Avatar participant={participant} large />
      <div>
        <strong>{participant.display_name}</strong>
        <span>{participant.preferred_language}</span>
      </div>
    </div>
  );
}

function MessageBubble({
  message,
  participant,
  mine,
  challengeTranslation,
  addCulturalContext,
  openRepair,
}: {
  message: Message;
  participant?: Participant;
  mine: boolean;
  challengeTranslation: (messageId: string) => void;
  addCulturalContext: (messageId: string) => void;
  openRepair: (messageId: string) => void;
}) {
  const translation = message.translations[0];
  const confidence = translation ? Math.round(translation.confidence * 100) : null;

  return (
    <article className={mine ? "message-row mine" : "message-row"}>
      <Avatar participant={participant} />
      <div className="message-cluster">
        <div className="message-meta">
          <strong>{participant?.display_name ?? "Participant"}</strong>
          <time>{new Date(message.sent_at).toLocaleTimeString([], { timeStyle: "short" })}</time>
        </div>
        <div className="message-card">
          <div className="original-line">
            <span>Original</span>
            <p>{message.original_text}</p>
          </div>
          <div className="translation-line">
            <span>Translation</span>
            <p>{translation?.translated_text ?? "Translation pending..."}</p>
          </div>
          <div className="message-state">
            <CheckCircle2 size={14} />
            {message.delivery_state.replace(/_/g, " ")}
            {confidence ? (
              <b>
                {translation?.source_language.toUpperCase()} {"->"}{" "}
                {translation?.target_language.toUpperCase()} {confidence}%
              </b>
            ) : null}
          </div>
        </div>
        <div className="message-actions">
          <button type="button" onClick={() => challengeTranslation(message.id)}>
            Translation may be incomplete
          </button>
          <button type="button" onClick={() => addCulturalContext(message.id)}>
            Context
          </button>
          <button type="button" onClick={() => openRepair(message.id)}>
            I need clarification
          </button>
        </div>
        {message.context_notes.map((note) => (
          <div className="context-note" key={note.id}>
            <Sparkles size={14} />
            {note.text}
          </div>
        ))}
      </div>
    </article>
  );
}

function RightRail({
  snapshot,
  inviteFacilitator,
  rejectFacilitator,
  transitionRepair,
  proposeArtifact,
  approveArtifact,
  publishArtifact,
  createProject,
}: {
  snapshot: RoomSnapshot;
  inviteFacilitator: () => void;
  rejectFacilitator: (responseId: string) => void;
  transitionRepair: (repair: RepairThread, state: RepairThread["state"]) => void;
  proposeArtifact: () => void;
  approveArtifact: () => void;
  publishArtifact: () => void;
  createProject: () => void;
}) {
  const latestFacilitation = snapshot.facilitator_responses[snapshot.facilitator_responses.length - 1];
  const latestRepair = snapshot.repairs[snapshot.repairs.length - 1];

  return (
    <aside className="right-rail">
      <HudPanel title="Cultural Context" status="ACTIVE">
        <div className="context-widget">
          <BrainCircuit size={42} />
          <div>
            <strong>Humility in Communication</strong>
            <p>Humility may build trust and open dialogue when communities are protecting local knowledge.</p>
            <span>Context Match 93%</span>
          </div>
        </div>
      </HudPanel>

      <HudPanel title="Fact-Check & Verification" status="ACTIVE">
        <VerificationRow
          state="Needs Context"
          claim="Local collaboration begins with listening."
          source="Participant statement"
          confidence="84%"
        />
        <VerificationRow
          state="Partially Supported"
          claim="Listening is not always the same as understanding."
          source="Dialogue context"
          confidence="91%"
        />
      </HudPanel>

      <HudPanel title="Misunderstanding Repair" status={latestRepair ? latestRepair.state : "STANDBY"}>
        {latestRepair ? (
          <div className="repair-widget">
            <AlertTriangle size={28} />
            <strong>{latestRepair.note}</strong>
            <span>{latestRepair.reason.replace(/_/g, " ")}</span>
            <div className="repair-actions">
              <button type="button" onClick={() => transitionRepair(latestRepair, "acknowledged")}>
                Acknowledge
              </button>
              <button type="button" onClick={() => transitionRepair(latestRepair, "resolved")}>
                Resolve
              </button>
              <button type="button" onClick={() => transitionRepair(latestRepair, "unresolved")}>
                Preserve
              </button>
            </div>
          </div>
        ) : (
          <div className="repair-quiet">
            <CircleDot size={34} />
            <span>No potential misunderstandings detected</span>
          </div>
        )}
      </HudPanel>

      <HudPanel title="AI Facilitation Suggestions" status="ACTIVE">
        {latestFacilitation ? (
          <FacilitationCard response={latestFacilitation} rejectFacilitator={rejectFacilitator} />
        ) : (
          <div className="ai-widget">
            <span>Suggested Action</span>
            <p>Invite the facilitator to suggest one clarification question.</p>
            <button type="button" onClick={inviteFacilitator}>
              Invite AI facilitator
            </button>
          </div>
        )}
      </HudPanel>

      <HudPanel title="Consent & Commons">
        <ConsentWidget
          snapshot={snapshot}
          proposeArtifact={proposeArtifact}
          approveArtifact={approveArtifact}
          publishArtifact={publishArtifact}
          createProject={createProject}
        />
      </HudPanel>
    </aside>
  );
}

function ConsentWidget({
  snapshot,
  proposeArtifact,
  approveArtifact,
  publishArtifact,
  createProject,
}: {
  snapshot: RoomSnapshot;
  proposeArtifact: () => void;
  approveArtifact: () => void;
  publishArtifact: () => void;
  createProject: () => void;
}) {
  const artifact = snapshot.artifact;
  const publication = snapshot.commons_publications[snapshot.commons_publications.length - 1];

  if (!artifact) {
    return (
      <div className="consent-widget">
        <FileCheck2 size={26} />
        <p>The conversation can remain private. Shared knowledge begins only by explicit proposal.</p>
        <button type="button" onClick={proposeArtifact}>
          Propose something from this conversation
        </button>
      </div>
    );
  }

  return (
    <div className="consent-widget">
      <strong>{artifact.title}</strong>
      <p>{artifact.shared_summary}</p>
      <code>{artifact.revision_hash.slice(0, 18)}...</code>
      <div className="approval-list">
        {artifact.required_approvers.map((approver) => (
          <span key={approver} className={snapshot.approvals.includes(approver) ? "approved" : ""}>
            {shortName(approver)} {snapshot.approvals.includes(approver) ? "approved" : "pending"}
          </span>
        ))}
      </div>
      <div className="consent-actions">
        <button type="button" onClick={approveArtifact}>
          Approve exact revision
        </button>
        <button type="button" onClick={publishArtifact}>
          Publish to The Commons
        </button>
        <button type="button" onClick={createProject} disabled={!publication}>
          Create project
        </button>
      </div>
      {publication ? (
        <span className="privacy-proof">
          <ShieldCheck size={15} />
          Commons export verified: no transcript exposed
        </span>
      ) : null}
    </div>
  );
}

function SectionView({
  section,
  snapshot,
  proposeArtifact,
  approveArtifact,
  publishArtifact,
  createProject,
}: {
  section: string;
  snapshot: RoomSnapshot;
  proposeArtifact: () => void;
  approveArtifact: () => void;
  publishArtifact: () => void;
  createProject: () => void;
}) {
  const commonsClean = publicCommonsHasNoTranscript(snapshot.commons_publications);
  return (
    <div className="section-view">
      <PanelRight size={34} />
      <h1>{section}</h1>
      {section === "Understanding" ? (
        <ConsentWidget
          snapshot={snapshot}
          proposeArtifact={proposeArtifact}
          approveArtifact={approveArtifact}
          publishArtifact={publishArtifact}
          createProject={createProject}
        />
      ) : null}
      {section === "The Commons" ? (
        <div className="publication-list">
          {snapshot.commons_publications.map((publication) => (
            <article key={publication.id}>
              <BookOpen size={20} />
              <strong>{publication.title}</strong>
              <p>{publication.summary}</p>
              <span>{publication.consent_verified ? "Consent verified" : "Consent pending"}</span>
            </article>
          ))}
          <span className={commonsClean ? "privacy-proof" : "privacy-proof warning"}>
            {commonsClean ? <ShieldCheck size={15} /> : <XCircle size={15} />}
            Public API contains no room transcript
          </span>
        </div>
      ) : null}
      {section === "Projects" ? (
        <div className="publication-list">
          {snapshot.projects.map((project) => (
            <article key={project.id}>
              <Wrench size={20} />
              <strong>{project.title}</strong>
              <p>{project.contribution_needs.join(", ")}</p>
              <span>{project.status}</span>
            </article>
          ))}
        </div>
      ) : null}
      {section === "Profile" ? (
        <p className="profile-note">Demo identities are local development identities with separate device keys.</p>
      ) : null}
    </div>
  );
}

function HudPanel({
  title,
  status,
  children,
}: {
  title: string;
  status?: string;
  children: ReactNode;
}) {
  return (
    <section className="hud-panel">
      <header>
        <h2>{title}</h2>
        {status ? <span>{status}</span> : null}
      </header>
      {children}
    </section>
  );
}

function VerificationRow({
  state,
  claim,
  source,
  confidence,
}: {
  state: string;
  claim: string;
  source: string;
  confidence: string;
}) {
  return (
    <div className="verification-row">
      <ShieldCheck size={18} />
      <div>
        <strong>{state}</strong>
        <p>{claim}</p>
        <span>{source}</span>
      </div>
      <b>{confidence}</b>
    </div>
  );
}

function FacilitationCard({
  response,
  rejectFacilitator,
}: {
  response: FacilitationResponse;
  rejectFacilitator: (id: string) => void;
}) {
  return (
    <div className="ai-widget">
      <span>Suggested Action</span>
      <p>{response.suggestion}</p>
      <small>{response.disclosure}</small>
      <button type="button" onClick={() => rejectFacilitator(response.id)}>
        Reject suggestion
      </button>
      {response.accepted === false ? <b>Suggestion rejected</b> : null}
    </div>
  );
}

function WorldBand({ snapshot }: { snapshot: RoomSnapshot }) {
  return (
    <footer className="world-band" aria-label="Global conversation metrics">
      <div className="world-map" aria-hidden="true">
        <span />
        <span />
        <span />
        <span />
      </div>
      <div className="metric-card">
        <span>Time Together</span>
        <strong>01:24:17</strong>
      </div>
      <div className="metric-card">
        <span>Words Exchanged</span>
        <strong>{snapshot.messages.reduce((sum, message) => sum + message.original_text.split(/\s+/).length, 0)}</strong>
      </div>
      <div className="metric-card trend">
        <span>Understanding Trend</span>
        <strong>+32%</strong>
      </div>
    </footer>
  );
}

function AvatarStack({ participants }: { participants: Participant[] }) {
  return (
    <div className="avatar-stack" aria-label="Participants">
      {participants.map((participant) => (
        <Avatar key={participant.id} participant={participant} />
      ))}
      <span>+8</span>
    </div>
  );
}

function Avatar({ participant, large = false }: { participant?: Participant; large?: boolean }) {
  return (
    <span className={large ? "avatar large" : "avatar"} aria-label={participant?.display_name}>
      {participant?.display_name.slice(0, 1) ?? "B"}
      <i className={participant?.present ? "online" : ""} />
    </span>
  );
}

function WaveBars() {
  return (
    <span className="wave-bars" aria-hidden="true">
      {Array.from({ length: 18 }).map((_, index) => (
        <i key={index} style={{ height: `${8 + ((index * 7) % 24)}px` }} />
      ))}
    </span>
  );
}

function AmbientLayer() {
  return (
    <div className="ambient-layer" aria-hidden="true">
      <span />
      <span />
      <span />
    </div>
  );
}

function participantById(snapshot: RoomSnapshot, id: string) {
  return snapshot.room.participants.find((participant) => participant.id === id);
}

function shortName(id: string) {
  const parts = id.split(":");
  return parts[parts.length - 1] ?? id;
}
