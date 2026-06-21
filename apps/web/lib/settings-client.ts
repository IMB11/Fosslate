import { unsafeRequestHeaders, type AuthUser } from "@/lib/auth-client";
import type {
  AuthProviderSetupStatus,
  EmailDeliverySetupStatus,
} from "@/lib/setup-types";

export type InstanceSettings = {
  github: AuthProviderSetupStatus;
  gitlab: AuthProviderSetupStatus;
  email: EmailDeliverySetupStatus;
};

export type SaveInstanceSsoProviderRequest = {
  enabled: boolean;
  client_id?: string;
  client_secret?: string;
  base_url?: string;
};

export type TestInstanceEmailDeliveryRequest = {
  resend_api_key?: string;
  from_name: string;
  from_email: string;
  test_recipient: string;
};

type AuthUserResponse = {
  user: AuthUser;
};

export class SettingsClientError extends Error {
  status: number;
  code: string;

  constructor(status: number, code: string) {
    super(code);
    this.name = "SettingsClientError";
    this.status = status;
    this.code = code;
  }
}

async function settingsRequest<T>(
  path: string,
  init: RequestInit & { body?: string } = {},
): Promise<T> {
  const response = await fetch(`/api/v1/settings${path}`, {
    credentials: "include",
    cache: "no-store",
    ...init,
    headers: {
      Accept: "application/json",
      ...(init.body ? { "Content-Type": "application/json" } : {}),
      ...(init.method && init.method !== "GET" ? unsafeRequestHeaders() : {}),
      ...(init.headers ?? {}),
    },
  });

  if (!response.ok) {
    throw new SettingsClientError(response.status, await readErrorCode(response));
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

function jsonBody(body: unknown): string {
  return JSON.stringify(body);
}

export async function claimInstanceAdmin(setupSecret: string): Promise<AuthUser> {
  const response = await settingsRequest<AuthUserResponse>(
    "/instance/admin/claim",
    {
      method: "POST",
      body: jsonBody({ setup_secret: setupSecret }),
    },
  );
  return response.user;
}

export async function getInstanceSettings(): Promise<InstanceSettings> {
  return settingsRequest<InstanceSettings>("/instance");
}

export async function saveInstanceSsoProvider(
  provider: "github" | "gitlab",
  body: SaveInstanceSsoProviderRequest,
): Promise<InstanceSettings> {
  return settingsRequest<InstanceSettings>(`/instance/sso/${provider}`, {
    method: "PUT",
    body: jsonBody(body),
  });
}

export async function testInstanceEmailDelivery(
  body: TestInstanceEmailDeliveryRequest,
): Promise<InstanceSettings> {
  return settingsRequest<InstanceSettings>("/instance/email/test", {
    method: "POST",
    body: jsonBody(body),
  });
}

export function settingsErrorMessage(error: unknown): string {
  if (error instanceof SettingsClientError) {
    if (error.status === 401) {
      return "The admin code was not accepted. Check the current code in the backend logs.";
    }

    if (error.status === 403) {
      return "Admin permission is required.";
    }

    if (error.status === 502) {
      return "Resend rejected the test email. Check the API key, sender, and recipient.";
    }

    if (error.code === "bad_request") {
      return "Check the fields and try again.";
    }
  }

  if (error instanceof Error) {
    return error.message;
  }

  return "Something went wrong.";
}
