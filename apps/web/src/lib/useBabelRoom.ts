import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { DEMO_ROOM_ID, WS_BASE } from "./config";
import { fetchDemoIdentities, fetchSnapshot } from "./api";
import { applyServerEvent } from "./reducer";
import { createUnsignedMessageEvent, signProtocolEvent } from "./protocol";
import type {
  ClientEvent,
  DemoIdentity,
  Message,
  RepairThread,
  RoomSnapshot,
  ServerEvent,
} from "../types";

const seedTextByParticipant: Record<string, string> = {
  "did:babel:amara":
    "I believe global collaboration begins with listening, but communities must keep control of their own knowledge.",
  "did:babel:diego":
    "Totalmente de acuerdo. Pero escuchar no siempre es entender.",
};

export function useBabelRoom() {
  const [activeParticipantId, setActiveParticipantId] = useState("did:babel:amara");
  const [snapshot, setSnapshot] = useState<RoomSnapshot | null>(null);
  const [connectionState, setConnectionState] = useState<"connecting" | "open" | "closed">(
    "connecting",
  );
  const [sequenceByParticipant, setSequenceByParticipant] = useState<Record<string, number>>({});
  const [draft, setDraft] = useState("");
  const socketRef = useRef<WebSocket | null>(null);

  const identitiesQuery = useQuery({
    queryKey: ["demo-identities"],
    queryFn: fetchDemoIdentities,
  });

  const activeIdentity = useMemo(
    () =>
      identitiesQuery.data?.find((identity) => identity.participant_id === activeParticipantId) ??
      null,
    [activeParticipantId, identitiesQuery.data],
  );

  const sendRaw = useCallback((event: ClientEvent) => {
    const socket = socketRef.current;
    if (socket?.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify(event));
    }
  }, []);

  useEffect(() => {
    if (!activeParticipantId) return;
    fetchSnapshot(activeParticipantId)
      .then(setSnapshot)
      .catch(() => undefined);
  }, [activeParticipantId]);

  useEffect(() => {
    if (!activeParticipantId) return;

    const socket = new WebSocket(
      `${WS_BASE}/api/v1/rooms/${DEMO_ROOM_ID}/ws?participant_id=${encodeURIComponent(
        activeParticipantId,
      )}`,
    );
    socketRef.current = socket;
    setConnectionState("connecting");

    socket.addEventListener("open", () => {
      setConnectionState("open");
      sendRaw({
        type: "room.join",
        payload: { participant_id: activeParticipantId },
      });
    });
    socket.addEventListener("close", () => setConnectionState("closed"));
    socket.addEventListener("message", (event) => {
      const parsed = JSON.parse(event.data as string) as ServerEvent;
      setSnapshot((current) =>
        current ? applyServerEvent(current, parsed) : parsed.type === "room.snapshot" ? parsed.payload : current,
      );
    });

    return () => {
      socket.close();
    };
  }, [activeParticipantId, sendRaw]);

  const sendTyping = useCallback(
    (typing: boolean) => {
      sendRaw({
        type: typing ? "typing.start" : "typing.stop",
        payload: { participant_id: activeParticipantId },
      });
    },
    [activeParticipantId, sendRaw],
  );

  const sendMessage = useCallback(
    async (text?: string) => {
      if (!activeIdentity || !snapshot) return;
      const originalText = text ?? draft.trim();
      if (!originalText) return;
      const nextSequence = (sequenceByParticipant[activeParticipantId] ?? 0) + 1;
      const unsigned = createUnsignedMessageEvent(
        activeIdentity,
        snapshot.room.id,
        nextSequence,
        originalText,
      );
      const signed = await signProtocolEvent(unsigned, activeIdentity);
      const language = activeParticipantId === "did:babel:diego" ? "es" : "en";
      const optimisticMessage: Message = {
        id: signed.id,
        room_id: snapshot.room.id,
        sender_id: activeIdentity.participant_id,
        sender_device_id: activeIdentity.device_id,
        original_language: language,
        original_text: originalText,
        sent_at: signed.created_at,
        client_sequence: nextSequence,
        reply_to: null,
        signature: signed.signature,
        event_hash: "pending",
        delivery_state: "local_pending",
        translations: [],
        context_notes: [],
      };
      setSnapshot((current) =>
        current ? { ...current, messages: [...current.messages, optimisticMessage] } : current,
      );
      setSequenceByParticipant((current) => ({
        ...current,
        [activeParticipantId]: nextSequence,
      }));
      setDraft("");
      sendRaw({
        type: "message.send",
        payload: {
          event: signed,
          original_language: language,
          original_text: originalText,
          reply_to: null,
        },
      });
    },
    [activeIdentity, activeParticipantId, draft, sendRaw, sequenceByParticipant, snapshot],
  );

  const sendSeed = useCallback(() => {
    void sendMessage(seedTextByParticipant[activeParticipantId]);
  }, [activeParticipantId, sendMessage]);

  const challengeTranslation = useCallback(
    (messageId: string) => {
      sendRaw({
        type: "translation.review",
        payload: {
          message_id: messageId,
          reviewer_id: activeParticipantId,
          note: "Translation may be incomplete; please preserve the speaker's intended context.",
        },
      });
    },
    [activeParticipantId, sendRaw],
  );

  const addCulturalContext = useCallback(
    (messageId: string) => {
      sendRaw({
        type: "message.context_added",
        payload: {
          message_id: messageId,
          author_id: activeParticipantId,
          note_type: "humility_in_communication",
          text:
            "Humility here means listening without taking control of the community's knowledge.",
        },
      });
    },
    [activeParticipantId, sendRaw],
  );

  const openRepair = useCallback(
    (messageId: string) => {
      sendRaw({
        type: "repair.open",
        payload: {
          target_id: messageId,
          opened_by: activeParticipantId,
          reason: "needs_clarification",
          note: "I need clarification before this becomes part of any shared artifact.",
        },
      });
    },
    [activeParticipantId, sendRaw],
  );

  const transitionRepair = useCallback(
    (repair: RepairThread, state: RepairThread["state"]) => {
      sendRaw({
        type: "repair.respond",
        payload: { repair_id: repair.id, state },
      });
    },
    [sendRaw],
  );

  const inviteFacilitator = useCallback(() => {
    sendRaw({
      type: "facilitator.request",
      payload: {
        requested_by: activeParticipantId,
        prompt: "Suggest a clarification question without declaring consensus.",
      },
    });
  }, [activeParticipantId, sendRaw]);

  const rejectFacilitator = useCallback(
    (responseId: string) => {
      sendRaw({ type: "facilitator.reject", payload: { response_id: responseId } });
    },
    [sendRaw],
  );

  const proposeArtifact = useCallback(() => {
    sendRaw({
      type: "artifact.propose",
      payload: { requested_by: activeParticipantId },
    });
  }, [activeParticipantId, sendRaw]);

  const approveArtifact = useCallback(() => {
    sendRaw({ type: "artifact.approve", payload: { participant_id: activeParticipantId } });
  }, [activeParticipantId, sendRaw]);

  const publishArtifact = useCallback(() => {
    sendRaw({ type: "artifact.publish", payload: null });
  }, [sendRaw]);

  const createProject = useCallback(() => {
    sendRaw({ type: "project.create", payload: null });
  }, [sendRaw]);

  return {
    identities: identitiesQuery.data ?? [],
    activeIdentity,
    activeParticipantId,
    setActiveParticipantId,
    snapshot,
    connectionState,
    draft,
    setDraft,
    sendTyping,
    sendMessage,
    sendSeed,
    challengeTranslation,
    addCulturalContext,
    openRepair,
    transitionRepair,
    inviteFacilitator,
    rejectFacilitator,
    proposeArtifact,
    approveArtifact,
    publishArtifact,
    createProject,
  };
}
