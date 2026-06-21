import type {
  SaveSsoProviderRequest,
  SetupCompleteResponse,
  SetupStatus,
  TestEmailDeliveryRequest,
} from "@/lib/setup-types";

export class SetupClientError extends Error {
  status: number;
  code: string;

  constructor(status: number, code: string) {
    super(code);
    this.name = "SetupClientError";
    this.status = status;
    this.code = code;
  }
}

async function setupRequest<T>(
  path: string,
  secret: string,
  init: RequestInit = {},
): Promise<T> {
  const response = await fetch(path, {
    ...init,
    headers: {
      Authorization: `Bearer ${secret}`,
      "Content-Type": "application/json",
      ...(init.headers ?? {}),
    },
  });

  if (!response.ok) {
    throw new SetupClientError(response.status, await readErrorCode(response));
  }

  return (await response.json()) as T;
}

async function readErrorCode(response: Response): Promise<string> {
  try {
    const body = (await response.json()) as { error?: string };
    return body.error ?? `http_${response.status}`;
  } catch {
    return `http_${response.status}`;
  }
}

export async function verifySetupSecret(secret: string): Promise<SetupStatus> {
  return setupRequest<SetupStatus>("/api/setup/verify", secret, {
    method: "POST",
  });
}

export async function getSetupStatus(secret: string): Promise<SetupStatus> {
  return setupRequest<SetupStatus>("/api/setup/status", secret);
}

export async function saveGithubSsoSetup(
  secret: string,
  body: SaveSsoProviderRequest,
): Promise<SetupStatus> {
  return setupRequest<SetupStatus>("/api/setup/sso/github", secret, {
    method: "PUT",
    body: JSON.stringify(body),
  });
}

export async function saveGitlabSsoSetup(
  secret: string,
  body: SaveSsoProviderRequest,
): Promise<SetupStatus> {
  return setupRequest<SetupStatus>("/api/setup/sso/gitlab", secret, {
    method: "PUT",
    body: JSON.stringify(body),
  });
}

export async function testEmailDeliverySetup(
  secret: string,
  body: TestEmailDeliveryRequest,
): Promise<SetupStatus> {
  return setupRequest<SetupStatus>("/api/setup/email/test", secret, {
    method: "POST",
    body: JSON.stringify(body),
  });
}

export async function completeSetup(
  secret: string,
): Promise<SetupCompleteResponse> {
  return setupRequest<SetupCompleteResponse>("/api/setup/complete", secret, {
    method: "POST",
  });
}

