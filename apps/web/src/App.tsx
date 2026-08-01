import { useMemo, useState } from "react";
import type { ReactNode } from "react";
import {
  AlertTriangle,
  BookOpen,
  CheckCircle2,
  ChevronRight,
  FileCheck2,
  Languages,
  Mic,
  Send,
  ShieldCheck,
  Sparkles,
  UsersRound,
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

const navItems = ["Live Room", "Understanding", "The Commons", "Projects"];

export default function App() {
  const room = useBabelRoom();
  const [activeNav, setActiveNav] = useState("Live Room");

  if (!room.snapshot) {
    return (
      <main className="boot-screen">
        <div className="brand-mark">BABEL</div>
        <p>Connecting to the local Babel node...</p>
      </main>
    );
  }

  return (
    <main className="babel-shell">
      <TopBar
        activeNav={activeNav}
        setActiveNav={setActiveNav}
        connectionState={room.connectionState}
        activeIdentity={room.activeIdentity}
      />

      <section className="room-layout" aria-label="Babel live conversation workspace">
        <ParticipantRail
          snapshot={room.snapshot}
          identities={room.identities}
          activeParticipantId={room.activeParticipantId}
          setActiveParticipantId={room.setActiveParticipantId}
        />

        <section className="conversation-surface" aria-label="Live conversation">
          {activeNav === "Live Room" ? (
            <ConversationRoom
              snapshot={room.snapshot}
              activeParticipantId={room.activeParticipantId}
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

        <UnderstandingRail
          snapshot={room.snapshot}
          inviteFacilitator={room.inviteFacilitator}
          rejectFacilitator={room.rejectFacilitator}
          transitionRepair={room.transitionRepair}
          proposeArtifact={room.proposeArtifact}
          approveArtifact={room.approveArtifact}
          publishArtifact={room.publishArtifact}
          createProject={room.createProject}
        />
      </section>
    </main>
  );
}

function TopBar({
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
    <header className="top-bar">
      <div className="logo-block" aria-label="Babel">
        <strong>BABEL</strong>
        <span>Live Room</span>
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
      <div className="room-status-line">
        <span className={`connection-dot ${connectionState}`} />
        <span>{connectionState === "open" ? "Connected" : "Reconnecting"}</span>
        <b>{activeIdentity?.display_name ?? "Amara"}</b>
      </div>
    </header>
  );
}

function ParticipantRail({
  snapshot,
  identities,
  activeParticipantId,
  setActiveParticipantId,
}: {
  snapshot: RoomSnapshot;
  identities: DemoIdentity[];
  activeParticipantId: string;
  setActiveParticipantId: (id: string) => void;
}) {
  return (
    <aside className="participant-rail">
      <Panel title="People">
        <div className="participant-list">
          {snapshot.room.participants.map((participant) => (
            <button
              className={participant.id === activeParticipantId ? "participant-row active" : "participant-row"}
              key={participant.id}
              type="button"
              onClick={() => setActiveParticipantId(participant.id)}
            >
              <Avatar participant={participant} />
              <span>
                <strong>{participant.display_name}</strong>
                <small>{participant.preferred_language}</small>
              </span>
            </button>
          ))}
        </div>
      </Panel>

      <Panel title="Room">
        <div className="quiet-room-card">
          <UsersRound size={20} />
          <strong>{snapshot.room.title}</strong>
          <span>{snapshot.room.privacy.replace(/_/g, " ")}</span>
        </div>
      </Panel>

      <Panel title="Identity">
        <div className="identity-switcher">
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
        </div>
      </Panel>
    </aside>
  );
}

function ConversationRoom({
  snapshot,
  activeParticipantId,
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
  activeParticipantId: string;
  draft: string;
  setDraft: (value: string) => void;
  sendTyping: (typing: boolean) => void;
  sendMessage: () => Promise<void>;
  sendSeed: () => void;
  challengeTranslation: (messageId: string) => void;
  addCulturalContext: (messageId: string) => void;
  openRepair: (messageId: string) => void;
}) {
  const typingParticipant = snapshot.room.participants.find(
    (participant) => participant.typing && participant.id !== activeParticipantId,
  );

  return (
    <>
      <div className="room-heading">
        <div>
          <span className="section-label">Live Room</span>
          <h1>{snapshot.room.title}</h1>
        </div>
        <div className="language-pair">
          <Languages size={18} />
          <span>English</span>
          <ChevronRight size={15} />
          <span>Spanish</span>
        </div>
      </div>

      <div className="message-stream" aria-live="polite">
        {snapshot.messages.length === 0 ? (
          <div className="empty-stream">
            <Languages size={36} />
            <strong>The room is quiet.</strong>
            <button type="button" onClick={sendSeed}>
              Send demo line
            </button>
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
          placeholder="Type a message..."
          aria-label="Message text"
        />
        <span className="language-toggle">EN</span>
        <span className="language-toggle">ES</span>
        <button type="submit" className="send-button" title="Send message">
          <Send size={21} />
        </button>
      </form>
    </>
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
          <div className="paired-lines">
            <div>
              <span>Original</span>
              <p>{message.original_text}</p>
            </div>
            <div>
              <span>Translation</span>
              <p>{translation?.translated_text ?? "Translation pending..."}</p>
            </div>
          </div>
          <div className="message-state">
            <CheckCircle2 size={14} />
            <span>{message.delivery_state.replace(/_/g, " ")}</span>
            {confidence ? <b>{confidence}%</b> : null}
          </div>
        </div>
        <div className="message-actions">
          <button type="button" onClick={() => challengeTranslation(message.id)}>
            Challenge translation
          </button>
          <button type="button" onClick={() => openRepair(message.id)}>
            Ask for clarification
          </button>
          <button type="button" onClick={() => addCulturalContext(message.id)}>
            Add context
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

function UnderstandingRail({
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
  const latestTranslation = useMemo(() => {
    const translations = snapshot.messages.flatMap((message) => message.translations);
    return translations[translations.length - 1];
  }, [snapshot.messages]);
  const confidence = latestTranslation ? Math.round(latestTranslation.confidence * 100) : 94;

  return (
    <aside className="understanding-rail">
      <Panel title="Understanding Thread">
        <div className="thread-item">
          <Languages size={18} />
          <div>
            <strong>Translation</strong>
            <span>{confidence}% confidence</span>
          </div>
        </div>

        {latestRepair ? (
          <div className="thread-card emphasis">
            <AlertTriangle size={20} />
            <span>Clarification Requested</span>
            <p>{latestRepair.note}</p>
            <div className="button-row">
              <button type="button" onClick={() => transitionRepair(latestRepair, "acknowledged")}>
                Acknowledge
              </button>
              <button type="button" onClick={() => transitionRepair(latestRepair, "resolved")}>
                Resolve
              </button>
            </div>
          </div>
        ) : (
          <div className="thread-card quiet">
            <CheckCircle2 size={20} />
            <span>No clarification open</span>
          </div>
        )}

        {latestFacilitation ? (
          <FacilitationCard response={latestFacilitation} rejectFacilitator={rejectFacilitator} />
        ) : (
          <div className="thread-card">
            <Sparkles size={19} />
            <span>Facilitation</span>
            <p>One clarification question can be invited, then accepted or rejected.</p>
            <button type="button" onClick={inviteFacilitator}>
              Invite facilitator
            </button>
          </div>
        )}
      </Panel>

      <Panel title="Consent">
        <ConsentWidget
          snapshot={snapshot}
          proposeArtifact={proposeArtifact}
          approveArtifact={approveArtifact}
          publishArtifact={publishArtifact}
          createProject={createProject}
        />
      </Panel>
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
        <FileCheck2 size={22} />
        <strong>Consent Not Yet Given</strong>
        <p>Shared insight requires an exact proposal and participant approval.</p>
        <button type="button" onClick={proposeArtifact}>
          Propose shared insight
        </button>
      </div>
    );
  }

  return (
    <div className="consent-widget">
      <FileCheck2 size={22} />
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
      <div className="button-row stack">
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
          Commons export verified
        </span>
      ) : null}
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
    <div className="thread-card">
      <Sparkles size={19} />
      <span>Facilitation</span>
      <p>{response.suggestion}</p>
      <small>{response.disclosure}</small>
      <button type="button" onClick={() => rejectFacilitator(response.id)}>
        Reject suggestion
      </button>
      {response.accepted === false ? <b>Suggestion rejected</b> : null}
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
      <span className="section-label">{section}</span>
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
    </div>
  );
}

function Panel({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="panel">
      <h2>{title}</h2>
      {children}
    </section>
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

function participantById(snapshot: RoomSnapshot, id: string) {
  return snapshot.room.participants.find((participant) => participant.id === id);
}

function shortName(id: string) {
  const parts = id.split(":");
  return parts[parts.length - 1] ?? id;
}
