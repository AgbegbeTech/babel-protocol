export const API_BASE =
  import.meta.env.VITE_BABEL_API_BASE?.replace(/\/$/, "") ?? "http://localhost:8080";

export const WS_BASE =
  import.meta.env.VITE_BABEL_WS_BASE?.replace(/\/$/, "") ?? "ws://localhost:8080";

export const DEMO_ROOM_ID = "11111111-1111-4111-8111-111111111111";
