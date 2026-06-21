import type { NextApiRequest, NextApiResponse } from "next";

import { proxySettingsRequest } from "@/lib/settings-proxy";

export default async function handler(
  request: NextApiRequest,
  response: NextApiResponse,
) {
  await proxySettingsRequest(request, response);
}
