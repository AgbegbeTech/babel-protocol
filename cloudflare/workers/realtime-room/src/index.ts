export interface Env {
  BABEL_NODE_ORIGIN: string;
}

export interface ConnectionState {
  participantId: string;
  connectedAt: number;
  lastCursor?: string;
}

export function participantFromUrl(url: string): string | null {
  return new URL(url).searchParams.get("participant_id");
}

export class ConversationRoom implements DurableObject {
  private sessions = new Map<WebSocket, ConnectionState>();

  constructor(
    private readonly state: DurableObjectState,
    private readonly env: Env,
  ) {}

  async fetch(request: Request): Promise<Response> {
    if (request.headers.get("upgrade") !== "websocket") {
      return Response.json({ error: "websocket required" }, { status: 426 });
    }

    const participantId = participantFromUrl(request.url);
    if (!participantId) {
      return Response.json({ error: "participant_id is required" }, { status: 403 });
    }

    const [client, server] = Object.values(new WebSocketPair());
    server.accept();
    this.sessions.set(server, { participantId, connectedAt: Date.now() });
    this.broadcast({
      type: "presence.updated",
      payload: { participant_id: participantId, present: true },
    });

    server.addEventListener("message", async (event) => {
      await this.forwardToNode(event.data, request.url, server);
    });
    server.addEventListener("close", () => this.close(server, participantId));
    server.addEventListener("error", () => this.close(server, participantId));

    return new Response(null, { status: 101, webSocket: client });
  }

  private async forwardToNode(data: unknown, originalUrl: string, socket: WebSocket) {
    const url = new URL(originalUrl);
    const upstream = new URL(this.env.BABEL_NODE_ORIGIN);
    upstream.pathname = url.pathname;
    upstream.search = url.search;
    const response = await fetch(upstream, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ websocket_event: data }),
    });
    if (!response.ok) {
      socket.send(JSON.stringify({ type: "error", payload: { message: "node rejected event" } }));
    }
  }

  private broadcast(event: unknown) {
    const text = JSON.stringify(event);
    for (const socket of this.sessions.keys()) {
      socket.send(text);
    }
  }

  private close(socket: WebSocket, participantId: string) {
    this.sessions.delete(socket);
    this.broadcast({
      type: "presence.updated",
      payload: { participant_id: participantId, present: false },
    });
  }
}

export default {
  fetch() {
    return Response.json({
      service: "babel-realtime-room",
      authority: "coordination-only",
    });
  },
};
