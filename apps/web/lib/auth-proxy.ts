import type { NextApiRequest, NextApiResponse } from "next";

import { backendBaseUrl } from "@/lib/setup-proxy";

const AUTH_PREFIX = "/api/v1/auth";
const AUTH_METHODS = ["GET", "POST"];

type HeaderValue = string | string[] | undefined;

export async function proxyAuthRequest(
  request: NextApiRequest,
  response: NextApiResponse,
) {
  if (!request.method || !AUTH_METHODS.includes(request.method)) {
    response.setHeader("Allow", AUTH_METHODS);
    response.status(405).json({ error: "method_not_allowed" });
    return;
  }

  const targetUrl = `${backendBaseUrl()}${authTargetPath(request)}`;
  const backendResponse = await fetch(targetUrl, {
    method: request.method,
    headers: requestHeaders(request),
    body: request.method === "GET" ? undefined : requestBody(request),
    cache: "no-store",
    redirect: "manual",
  });

  forwardHeaders(backendResponse, response);

  const text = await backendResponse.text();
  response.status(backendResponse.status);

  if (!text || backendResponse.status === 204) {
    response.end();
    return;
  }

  response.send(text);
}

function authTargetPath(request: NextApiRequest): string {
  const path = request.query.path;
  const segments = Array.isArray(path) ? path : path ? [path] : [];
  const encodedPath = segments.map(encodeURIComponent).join("/");
  const query = queryString(request);

  return `${AUTH_PREFIX}/${encodedPath}${query ? `?${query}` : ""}`;
}

function queryString(request: NextApiRequest): string {
  const params = new URLSearchParams();

  for (const [key, value] of Object.entries(request.query)) {
    if (key === "path") {
      continue;
    }

    if (Array.isArray(value)) {
      value.forEach((item) => params.append(key, item));
    } else if (value !== undefined) {
      params.append(key, value);
    }
  }

  return params.toString();
}

function requestHeaders(request: NextApiRequest): HeadersInit {
  const headers: Record<string, string> = {
    Accept: "application/json",
  };

  copyHeader(headers, "Cookie", request.headers.cookie);
  copyHeader(headers, "User-Agent", request.headers["user-agent"]);
  copyHeader(headers, "X-Forwarded-For", request.headers["x-forwarded-for"]);
  copyHeader(headers, "X-CSRF-Token", request.headers["x-csrf-token"]);

  if (request.method !== "GET" && request.body !== undefined) {
    headers["Content-Type"] = "application/json";
  }

  return headers;
}

function requestBody(request: NextApiRequest): string | undefined {
  if (request.body === undefined) {
    return undefined;
  }

  return typeof request.body === "string"
    ? request.body
    : JSON.stringify(request.body);
}

function copyHeader(
  headers: Record<string, string>,
  name: string,
  value: HeaderValue,
) {
  if (Array.isArray(value)) {
    headers[name] = value.join(", ");
  } else if (value) {
    headers[name] = value;
  }
}

function forwardHeaders(
  backendResponse: Response,
  response: NextApiResponse,
) {
  const contentType = backendResponse.headers.get("content-type");
  const location = backendResponse.headers.get("location");
  const cookies = setCookieHeaders(backendResponse.headers);

  if (contentType) {
    response.setHeader("Content-Type", contentType);
  }

  if (location) {
    response.setHeader("Location", location);
  }

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
