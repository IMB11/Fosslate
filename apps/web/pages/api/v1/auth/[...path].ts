import type { NextApiRequest, NextApiResponse } from "next";

import { proxyAuthRequest } from "@/lib/auth-proxy";

export default async function handler(
  request: NextApiRequest,
  response: NextApiResponse,
) {
  await proxyAuthRequest(request, response);
}
