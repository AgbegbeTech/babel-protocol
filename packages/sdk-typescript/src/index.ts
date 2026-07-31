export const BABEL_PROTOCOL = "babel/1" as const;

export interface BabelClientOptions {
  apiBase: string;
  wsBase: string;
  participantId: string;
}

export class BabelClient {
  constructor(private readonly options: BabelClientOptions) {}

  health() {
    return fetch(`${this.options.apiBase}/api/v1/health`).then((response) => response.json());
  }

  room(roomId: string) {
    const params = new URLSearchParams({ participant_id: this.options.participantId });
    return fetch(`${this.options.apiBase}/api/v1/rooms/${roomId}?${params}`).then((response) =>
      response.json(),
    );
  }

  connectRoom(roomId: string) {
    const params = new URLSearchParams({ participant_id: this.options.participantId });
    return new WebSocket(`${this.options.wsBase}/api/v1/rooms/${roomId}/ws?${params}`);
  }
}
