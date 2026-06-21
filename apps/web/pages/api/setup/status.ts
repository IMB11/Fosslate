import type { NextApiRequest, NextApiResponse } from "next";

import { allowMethods, proxySetupRequest } from "@/lib/setup-proxy";

export default async function handler(
  request: NextApiRequest,
  response: NextApiResponse,
) {
  if (!allowMethods(request, response, ["GET"])) {
    return;
  }

  await proxySetupRequest(request, response, {
    method: "GET",
    path: "/api/v1/setup/status",
  });
}

