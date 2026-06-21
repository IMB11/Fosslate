import { unsafeRequestHeaders } from "@/lib/auth-client";
import type { Language, Project } from "@/lib/projects";

export type CreateProjectRequest = {
  name: string;
  icon_asset_id: number | null;
  source_language: Language;
};

export class ProjectsClientError extends Error {
  status: number;
  code: string;

  constructor(status: number, code: string) {
    super(code);
    this.name = "ProjectsClientError";
    this.status = status;
    this.code = code;
  }
}

export async function createProject(
  body: CreateProjectRequest,
): Promise<Project> {
  const response = await fetch("/api/v1/projects", {
    method: "POST",
    credentials: "include",
    cache: "no-store",
    headers: {
      Accept: "application/json",
      "Content-Type": "application/json",
      ...unsafeRequestHeaders(),
    },
    body: JSON.stringify(body),
  });

  if (!response.ok) {
    throw new ProjectsClientError(response.status, await readErrorCode(response));
  }

  return (await response.json()) as Project;
}

export function projectErrorMessage(error: unknown): string {
  if (error instanceof ProjectsClientError) {
    if (error.status === 401) {
      return "Sign in before creating a project.";
    }

    if (error.status === 403) {
      return "Admin permission is required.";
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

async function readErrorCode(response: Response): Promise<string> {
  try {
    const body = (await response.json()) as { error?: string };
    return body.error ?? `http_${response.status}`;
  } catch {
    return `http_${response.status}`;
  }
}
