import type { NextApiRequest, NextApiResponse } from "next";

import { proxyProjectsRequest } from "@/lib/projects-proxy";

export default async function handler(
  request: NextApiRequest,
  response: NextApiResponse,
) {
  await proxyProjectsRequest(request, response);
}
