import type { IncomingMessage, ServerResponse } from "http";

import { backendBaseUrl } from "@/lib/setup-proxy";
import type { AuthUser } from "@/lib/auth-client";

type AuthUserResponse = {
  user: AuthUser;
};

type ServerAuthSessionResult = {
  user: AuthUser | null;
  resolved: boolean;
};

export async function getServerAuthSession(
  request: IncomingMessage | undefined,
  response: ServerResponse | undefined,
): Promise<ServerAuthSessionResult> {
  if (!request) {
    return { user: null, resolved: false };
  }

  try {
    const session = await authRequest("/api/v1/auth/session", "GET", request);
    if (session.ok) {
      return { user: session.user, resolved: true };
    }

    if (session.status !== 401) {
      return { user: null, resolved: true };
    }

    const refreshed = await authRequest(
      "/api/v1/auth/session/refresh",
      "POST",
      request,
    );

    if (!refreshed.ok) {
      return { user: null, resolved: true };
    }

    forwardSetCookie(refreshed.headers, response);
    return { user: refreshed.user, resolved: true };
  } catch {
    return { user: null, resolved: false };
  }
}

async function authRequest(
  path: string,
  method: "GET" | "POST",
  request: IncomingMessage,
): Promise<
  | {
      ok: true;
      headers: Headers;
      user: AuthUser;
    }
  | {
      ok: false;
      headers: Headers;
      status: number;
    }
> {
  const backendResponse = await fetch(`${backendBaseUrl()}${path}`, {
    method,
    headers: requestHeaders(request),
    cache: "no-store",
  });

  if (!backendResponse.ok) {
    return {
      ok: false,
      headers: backendResponse.headers,
      status: backendResponse.status,
    };
  }

  const body = (await backendResponse.json()) as AuthUserResponse;

  return {
    ok: true,
    headers: backendResponse.headers,
    user: body.user,
  };
}

function requestHeaders(request: IncomingMessage): HeadersInit {
  const headers: Record<string, string> = {
    Accept: "application/json",
  };

  copyHeader(headers, "Cookie", request.headers.cookie);
  copyHeader(headers, "User-Agent", request.headers["user-agent"]);
  copyHeader(headers, "X-Forwarded-For", request.headers["x-forwarded-for"]);

  return headers;
}

function copyHeader(
  headers: Record<string, string>,
  name: string,
  value: string | string[] | undefined,
) {
  if (Array.isArray(value)) {
    headers[name] = value.join(", ");
  } else if (value) {
    headers[name] = value;
  }
}

function forwardSetCookie(
  headers: Headers,
  response: ServerResponse | undefined,
) {
  if (!response) {
    return;
  }

  const cookies = setCookieHeaders(headers);
  if (cookies.length > 0) {
    response.setHeader("Set-Cookie", cookies);
  }
}

function setCookieHeaders(headers: Headers): string[] {
  const getSetCookie = (
    headers as Headers & { getSetCookie?: () => string[] }
  ).getSetCookie?.();

  if (getSetCookie && getSetCookie.length > 0) {
    return getSetCookie;
  }

  const header = headers.get("set-cookie");
  if (!header) {
    return [];
  }

  return header.split(/,(?=\s*[^;,]+=)/).map((cookie) => cookie.trim());
}
