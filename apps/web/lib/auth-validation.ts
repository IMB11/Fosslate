import { AuthClientError } from "@/lib/auth-client";

export function validEmail(email: string): boolean {
  const trimmed = email.trim();
  return trimmed.includes("@") && trimmed.length <= 320;
}

export function validateEmailPassword(
  email: string,
  password: string,
): string | null {
  if (!validEmail(email)) {
    return "Enter a valid email address.";
  }
  if (!password) {
    return "Enter your password.";
  }
  return null;
}

export function validatePasswordPolicy(password: string): string | null {
  if (password.length < 8) {
    return "Password must be at least 8 characters.";
  }
  if (!/[A-Z]/.test(password)) {
    return "Password must include an uppercase character.";
  }
  if (!/[^A-Za-z0-9\s]/.test(password)) {
    return "Password must include a special character.";
  }
  return null;
}

export function validatePasswordConfirmation(
  password: string,
  confirmation: string,
): string | null {
  return password === confirmation ? null : "Passwords do not match.";
}

export function authErrorMessage(error: unknown, fallback: string): string {
  if (!(error instanceof AuthClientError)) {
    return fallback;
  }

  if (error.status === 401) {
    return "Those credentials are invalid or expired.";
  }

  const messages: Record<string, string> = {
    account_exists: "An account already exists for that email.",
    bad_request: "Check the details and try again.",
    email_delivery_failed: "The email could not be sent. Try again.",
    email_delivery_not_configured:
      "Email delivery is not configured for this instance.",
    forbidden: "This request is no longer valid. Refresh and try again.",
    not_found: "That auth provider is not available.",
  };

  return messages[error.code] ?? fallback;
}

export function resetErrorMessage(error: unknown): string {
  if (error instanceof AuthClientError) {
    if (error.status === 401) {
      return "That reset link is invalid or expired.";
    }
    if (error.code === "bad_request") {
      return "Check the password details and try again.";
    }
  }

  return "Could not reset the password.";
}

export function safeRedirectPath(value: unknown, fallback = "/"): string {
  if (typeof value !== "string") {
    return fallback;
  }

  return value.startsWith("/") && !value.startsWith("//") ? value : fallback;
}
