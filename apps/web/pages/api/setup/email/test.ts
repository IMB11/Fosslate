import type { NextApiRequest, NextApiResponse } from "next";

import { allowMethods, proxySetupRequest } from "@/lib/setup-proxy";

export default async function handler(
  request: NextApiRequest,
  response: NextApiResponse,
) {
  if (!allowMethods(request, response, ["POST"])) {
    return;
  }

  await proxySetupRequest(request, response, {
    method: "POST",
    path: "/api/v1/setup/email/test",
    body: request.body,
  });
}

