import type { NextApiRequest, NextApiResponse } from "next";

import { backendBaseUrl } from "@/lib/setup-proxy";

const PROJECTS_METHODS = ["GET", "POST"];

type HeaderValue = string | string[] | undefined;

export async function proxyProjectsRequest(
  request: NextApiRequest,
  response: NextApiResponse,
) {
  if (!request.method || !PROJECTS_METHODS.includes(request.method)) {
    response.setHeader("Allow", PROJECTS_METHODS);
    response.status(405).json({ error: "method_not_allowed" });
    return;
  }

  const backendResponse = await fetch(`${backendBaseUrl()}/api/v1/projects`, {
    method: request.method,
    headers: requestHeaders(request),
    body: request.method === "GET" ? undefined : requestBody(request),
    cache: "no-store",
  });

  const text = await backendResponse.text();
  const contentType = backendResponse.headers.get("content-type");

  response.status(backendResponse.status);

  if (contentType) {
    response.setHeader("Content-Type", contentType);
  }

  if (!text) {
    response.end();
    return;
  }

  response.send(text);
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
