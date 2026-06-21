import { SetupClientError } from "@/lib/setup-client";
import type { SetupStatus, SetupStep } from "@/lib/setup-types";

export const SECRET_STORAGE_KEY = "fosslate.setupSecret";

export type Provider = "github" | "gitlab";

export function activeSetupStep(status?: SetupStatus): SetupStep {
  if (!status) {
    return "github";
  }

  if (status.next_step === "complete") {
    return "email";
  }

  return status.next_step ?? "github";
}

export function isStepDone(step: SetupStep, status?: SetupStatus): boolean {
  if (!status) {
    return false;
  }

  if (step === "github") {
    return status.github.configured || status.github.skipped;
  }

  if (step === "gitlab") {
    return status.gitlab.configured || status.gitlab.skipped;
  }

  if (step === "email") {
    return status.email.configured;
  }

  return status.completed;
}

export function titleForStep(step: SetupStep) {
  if (step === "github") {
    return "Set up GitHub SSO";
  }

  if (step === "gitlab") {
    return "Set up GitLab SSO";
  }

  if (step === "email") {
    return "Set up email delivery";
  }

  return "Complete setup";
}

export function descriptionForStep(step: SetupStep) {
  if (step === "github") {
    return "Add GitHub OAuth credentials, or skip this provider if your instance will not use GitHub sign-in.";
  }

  if (step === "gitlab") {
    return "Add GitLab OAuth credentials for gitlab.com or your own GitLab instance, or skip this provider.";
  }

  if (step === "email") {
    return "Send a test email with Resend before completing first-run setup.";
  }

  return "Finish first-run setup and open Fosslate.";
}

export function errorMessage(error: unknown) {
  if (error instanceof SetupClientError) {
    if (error.status === 401) {
      return "The setup code was not accepted. Check the current code in the API logs.";
    }

    if (error.status === 409 || error.code === "setup_complete") {
      return "Setup has already been completed.";
    }

    if (error.status === 502 || error.code === "email_delivery_failed") {
      return "Resend rejected the test email. Check the API key, verified sender, and recipient address.";
    }

    if (error.status === 400) {
      return "The backend rejected this step. Check the required fields and complete earlier steps first.";
    }

    return `Backend returned ${error.status}: ${error.code}`;
  }

  if (error instanceof Error) {
    return error.message;
  }

  return "Something went wrong.";
}

