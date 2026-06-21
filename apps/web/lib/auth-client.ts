export type AuthUser = {
  id: number;
  email: string;
  username: string;
  avatar_url: string | null;
  email_verified_at: string | null;
  created_at: string;
  updated_at: string;
};

export type AuthProviderAvailability = {
  enabled: boolean;
  start_url: string | null;
  base_url: string | null;
};

export type AuthProviders = {
  password: boolean;
  sso: {
    github: AuthProviderAvailability;
    gitlab: AuthProviderAvailability;
  };
};

type AuthUserResponse = {
  user: AuthUser;
};

type RequestBody = Record<string, unknown>;

export class AuthClientError extends Error {
  status: number;
  code: string;

  constructor(status: number, code: string) {
    super(code);
    this.name = "AuthClientError";
    this.status = status;
    this.code = code;
  }
}

async function authRequest<T>(
  path: string,
  init: RequestInit & { body?: string } = {},
): Promise<T> {
  const response = await fetch(`/api/v1/auth${path}`, {
    credentials: "include",
    cache: "no-store",
    ...init,
    headers: {
      Accept: "application/json",
      ...(init.body ? { "Content-Type": "application/json" } : {}),
      ...(init.headers ?? {}),
    },
  });

  if (!response.ok) {
    throw new AuthClientError(response.status, await readErrorCode(response));
  }

  if (response.status === 202 || response.status === 204) {
    return undefined as T;
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

function jsonBody(body: RequestBody): string {
  return JSON.stringify(body);
}

export async function getAuthProviders(): Promise<AuthProviders> {
  return authRequest<AuthProviders>("/providers");
}

export async function startSignup(body: {
  email: string;
  password: string;
}): Promise<void> {
  await authRequest<void>("/signup/start", {
    method: "POST",
    body: jsonBody(body),
  });
}

export async function completeSignup(body: {
  email: string;
  password: string;
  code: string;
}): Promise<AuthUser> {
  const response = await authRequest<AuthUserResponse>("/signup/complete", {
    method: "POST",
    body: jsonBody(body),
  });
  return response.user;
}

export async function login(body: {
  email: string;
  password: string;
}): Promise<AuthUser> {
  const response = await authRequest<AuthUserResponse>("/login", {
    method: "POST",
    body: jsonBody(body),
  });
  return response.user;
}

export async function getAuthSession(): Promise<AuthUser> {
  const response = await authRequest<AuthUserResponse>("/session");
  return response.user;
}

export async function refreshAuthSession(): Promise<AuthUser> {
  const response = await authRequest<AuthUserResponse>("/session/refresh", {
    method: "POST",
  });
  return response.user;
}

export async function getAuthSessionWithRefresh(): Promise<AuthUser> {
  try {
    return await getAuthSession();
  } catch (error) {
    if (error instanceof AuthClientError && error.status === 401) {
      return refreshAuthSession();
    }
    throw error;
  }
}

export async function getOptionalAuthSession(): Promise<AuthUser | null> {
  try {
    return await getAuthSessionWithRefresh();
  } catch (error) {
    if (error instanceof AuthClientError && error.status === 401) {
      return null;
    }
    throw error;
  }
}

export async function logout(): Promise<void> {
  await authRequest<void>("/logout", {
    method: "POST",
  });
}

export async function forgotPassword(body: { email: string }): Promise<void> {
  await authRequest<void>("/password/forgot", {
    method: "POST",
    body: jsonBody(body),
  });
}

export async function resetPassword(body: {
  token: string;
  password: string;
  password_confirmation: string;
}): Promise<void> {
  await authRequest<void>("/password/reset", {
    method: "POST",
    body: jsonBody(body),
  });
}

export function csrfToken(): string | null {
  if (typeof document === "undefined") {
    return null;
  }

  return (
    document.cookie
      .split(";")
      .map((cookie) => cookie.trim())
      .find((cookie) => cookie.startsWith("fs_csrf="))
      ?.slice("fs_csrf=".length) ?? null
  );
}

export function unsafeRequestHeaders(): HeadersInit {
  const token = csrfToken();
  return token ? { "X-CSRF-Token": token } : {};
}
