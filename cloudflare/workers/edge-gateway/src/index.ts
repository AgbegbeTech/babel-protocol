export interface Env {
  BABEL_NODE_ORIGIN: string;
  ALLOWED_WEB_ORIGIN: string;
  ROOMS: DurableObjectNamespace;
}

const privatePath = /^\/api\/v1\/rooms\/([^/]+)\/ws$/;

export function securityHeaders(requestId: string): Headers {
  const headers = new Headers();
  headers.set("x-babel-request-id", requestId);
  headers.set("x-content-type-options", "nosniff");
  headers.set("referrer-policy", "strict-origin-when-cross-origin");
  headers.set("permissions-policy", "camera=(), microphone=(self), geolocation=()");
  headers.set("cache-control", "no-store");
  return headers;
}

export function hasBasicEnvelopeShape(value: unknown): boolean {
  if (!value || typeof value !== "object") return false;
  const event = value as Record<string, unknown>;
  return (
    event.protocol === "babel/1" &&
    typeof event.id === "string" &&
    typeof event.schema === "string" &&
    typeof event.author_id === "string" &&
    typeof event.device_id === "string" &&
    typeof event.client_sequence === "number" &&
    typeof event.signature === "string"
  );
}

async function handleRequest(request: Request, env: Env): Promise<Response> {
  const requestId = crypto.randomUUID();
  const headers = securityHeaders(requestId);
  headers.set("access-control-allow-origin", env.ALLOWED_WEB_ORIGIN);
  headers.set("vary", "origin");

  if (request.method === "OPTIONS") {
    headers.set("access-control-allow-methods", "GET,POST,OPTIONS");
    headers.set("access-control-allow-headers", "content-type,authorization");
    return new Response(null, { headers });
  }

  const url = new URL(request.url);
  const roomMatch = url.pathname.match(privatePath);
  if (roomMatch) {
    const id = env.ROOMS.idFromName(`conversation:${roomMatch[1]}`);
    return env.ROOMS.get(id).fetch(request);
  }

  if (request.method === "POST") {
    const clone = request.clone();
    const contentLength = Number(request.headers.get("content-length") ?? "0");
    if (contentLength > 128_000) {
      return Response.json({ error: "request too large" }, { status: 413, headers });
    }
    try {
      const body = (await clone.json()) as unknown;
      const event =
        body && typeof body === "object" && "event" in body
          ? (body as { event?: unknown }).event
          : undefined;
      if (event && !hasBasicEnvelopeShape(event)) {
        return Response.json({ error: "invalid event envelope" }, { status: 400, headers });
      }
    } catch {
      return Response.json({ error: "invalid JSON" }, { status: 400, headers });
    }
  }

  const upstream = new URL(request.url);
  upstream.protocol = "https:";
  const nodeOrigin = new URL(env.BABEL_NODE_ORIGIN);
  upstream.hostname = nodeOrigin.hostname;
  upstream.port = nodeOrigin.port;
  upstream.pathname = url.pathname;
  upstream.search = url.search;
  const response = await fetch(upstream, request);
  const outbound = new Response(response.body, response);
  securityHeaders(requestId).forEach((value, key) => outbound.headers.set(key, value));
  return outbound;
}

export default {
  fetch: handleRequest,
};
