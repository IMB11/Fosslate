export type BackendMeta = {
  app: string;
  version: string;
  database: {
    status: string;
  };
};

export type BackendMetaResult =
  | {
      ok: true;
      data: BackendMeta;
    }
  | {
      ok: false;
      error: string;
    };

export async function getBackendMeta(): Promise<BackendMetaResult> {
  const baseUrl = process.env.INTERNAL_API_URL ?? "http://127.0.0.1:4000";

  try {
    const response = await fetch(`${baseUrl}/api/v1/meta`, {
      cache: "no-store",
    });

    if (!response.ok) {
      return {
        ok: false,
        error: `Backend returned HTTP ${response.status}`,
      };
    }

    return {
      ok: true,
      data: (await response.json()) as BackendMeta,
    };
  } catch (error) {
    const message = error instanceof Error ? error.message : "Unknown error";

    return {
      ok: false,
      error: `Could not reach backend at ${baseUrl}: ${message}`,
    };
  }
}

