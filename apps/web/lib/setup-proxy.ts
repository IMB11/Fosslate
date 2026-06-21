import type { NextApiRequest, NextApiResponse } from "next";

type ProxyOptions = {
  method: "GET" | "POST" | "PUT";
  path: string;
  body?: unknown;
};

export function allowMethods(
  request: NextApiRequest,
  response: NextApiResponse,
  methods: string[],
): boolean {
  if (request.method && methods.includes(request.method)) {
    return true;
  }

  response.setHeader("Allow", methods);
  response.status(405).json({ error: "method_not_allowed" });
  return false;
}

export async function proxySetupRequest(
  request: NextApiRequest,
  response: NextApiResponse,
  options: ProxyOptions,
) {
  const authorization = request.headers.authorization;

  if (!authorization) {
    response.status(401).json({ error: "unauthorized" });
    return;
  }

  const backendResponse = await fetch(`${backendBaseUrl()}${options.path}`, {
    method: options.method,
    headers: {
      Accept: "application/json",
      Authorization: authorization,
      ...(options.body === undefined ? {} : { "Content-Type": "application/json" }),
    },
    body: options.body === undefined ? undefined : JSON.stringify(options.body),
    cache: "no-store",
  });

  const text = await backendResponse.text();

  response.status(backendResponse.status);

  if (!text) {
    response.end();
    return;
  }

  const contentType = backendResponse.headers.get("content-type");
  if (contentType?.includes("application/json")) {
    response.json(JSON.parse(text));
    return;
  }

  response.send(text);
}

export function backendBaseUrl() {
  return (process.env.INTERNAL_API_URL ?? "http://127.0.0.1:4000").replace(
    /\/$/,
    "",
  );
}

