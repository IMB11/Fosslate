import type { NextApiRequest, NextApiResponse } from "next";

import { allowMethods, proxySetupRequest } from "@/lib/setup-proxy";

export default async function handler(
  request: NextApiRequest,
  response: NextApiResponse,
) {
  if (!allowMethods(request, response, ["PUT"])) {
    return;
  }

  const provider = request.query.provider;
  if (provider !== "github" && provider !== "gitlab") {
    response.status(404).json({ error: "not_found" });
    return;
  }

  await proxySetupRequest(request, response, {
    method: "PUT",
    path: `/api/v1/setup/sso/${provider}`,
    body: request.body,
  });
}

