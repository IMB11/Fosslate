export type Language = {
  key: string;
  name: string;
};

export type Project = {
  id: number;
  public_id: string;
  name: string;
  icon_asset_id: number | null;
  source_language: Language;
  created_at: string;
  updated_at: string;
};

export type ProjectTargetLanguage = {
  id: number;
  project_id: number;
  language: Language;
  created_at: string;
  updated_at: string;
};

export type Namespace = {
  id: number;
  project_id: number;
  name: string;
  created_at: string;
  updated_at: string;
};

export type NamespaceLanguageStats = {
  project_id: number;
  namespace_id: number;
  target_language_id: number;
  string_count: number;
  translated_count: number;
  approved_count: number;
  candidate_count: number;
  missing_count: number;
  updated_at: string;
};

export type ProjectSummary = {
  project: Project;
  targetLanguages: ProjectTargetLanguage[];
  namespaceCount: number;
  sourceStringCount: number;
  translatedCount: number;
  approvedCount: number;
  candidateCount: number;
  missingCount: number;
  translationPercent: number;
  approvalPercent: number;
  updatedAt: string;
};

export type NamespaceSummary = {
  namespace: Namespace;
  stringCount: number;
  translatedCount: number;
  approvedCount: number;
  candidateCount: number;
  missingCount: number;
  translationPercent: number;
  approvalPercent: number;
  updatedAt: string;
};

export type ProjectDetail = ProjectSummary & {
  namespaces: NamespaceSummary[];
  stats: NamespaceLanguageStats[];
};

export type ProjectDataResult<T> =
  | {
      ok: true;
      data: T;
    }
  | {
      ok: false;
      status?: number;
      error: string;
    };

type ProjectParts = {
  project: Project;
  targetLanguages: ProjectTargetLanguage[];
  namespaces: Namespace[];
  stats: NamespaceLanguageStats[];
};

export async function getProjectSummaries(): Promise<
  ProjectDataResult<ProjectSummary[]>
> {
  try {
    const projects = await fetchBackend<Project[]>("/api/v1/projects");
    const summaries = await Promise.all(
      projects.map(async (project) =>
        summarizeProject({
          project,
          targetLanguages: await fetchTargetLanguages(project.public_id),
          namespaces: await fetchNamespaces(project.public_id),
          stats: await fetchNamespaceStats(project.public_id),
        }),
      ),
    );

    return { ok: true, data: summaries };
  } catch (error) {
    return normalizeProjectError(error);
  }
}

export async function getProjectDetail(
  projectPublicId: string,
): Promise<ProjectDataResult<ProjectDetail>> {
  try {
    const [project, targetLanguages, namespaces, stats] = await Promise.all([
      fetchBackend<Project>(`/api/v1/projects/${projectPublicId}`),
      fetchTargetLanguages(projectPublicId),
      fetchNamespaces(projectPublicId),
      fetchNamespaceStats(projectPublicId),
    ]);
    const summary = summarizeProject({
      project,
      targetLanguages,
      namespaces,
      stats,
    });

    return {
      ok: true,
      data: {
        ...summary,
        namespaces: namespaces.map((namespace) =>
          summarizeNamespace(namespace, statsForNamespace(stats, namespace.id)),
        ),
        stats,
      },
    };
  } catch (error) {
    return normalizeProjectError(error);
  }
}

async function fetchTargetLanguages(
  projectPublicId: string,
): Promise<ProjectTargetLanguage[]> {
  return fetchBackend<ProjectTargetLanguage[]>(
    `/api/v1/projects/${projectPublicId}/languages`,
  );
}

async function fetchNamespaces(projectPublicId: string): Promise<Namespace[]> {
  return fetchBackend<Namespace[]>(
    `/api/v1/projects/${projectPublicId}/namespaces`,
  );
}

async function fetchNamespaceStats(
  projectPublicId: string,
): Promise<NamespaceLanguageStats[]> {
  return fetchBackend<NamespaceLanguageStats[]>(
    `/api/v1/projects/${projectPublicId}/stats/namespaces`,
  );
}

function summarizeProject({
  project,
  targetLanguages,
  namespaces,
  stats,
}: ProjectParts): ProjectSummary {
  const namespaceSummaries = namespaces.map((namespace) =>
    summarizeNamespace(namespace, statsForNamespace(stats, namespace.id)),
  );
  const sourceStringCount = namespaceSummaries.reduce(
    (total, namespace) => total + namespace.stringCount,
    0,
  );
  const translatedCount = sum(stats, "translated_count");
  const approvedCount = sum(stats, "approved_count");
  const candidateCount = sum(stats, "candidate_count");
  const missingCount = sum(stats, "missing_count");
  const requiredTranslations = sourceStringCount * targetLanguages.length;

  return {
    project,
    targetLanguages,
    namespaceCount: namespaces.length,
    sourceStringCount,
    translatedCount,
    approvedCount,
    candidateCount,
    missingCount,
    translationPercent: percent(translatedCount, requiredTranslations),
    approvalPercent: percent(approvedCount, requiredTranslations),
    updatedAt: latestDate([
      project.updated_at,
      ...targetLanguages.map((language) => language.updated_at),
      ...namespaces.map((namespace) => namespace.updated_at),
      ...stats.map((row) => row.updated_at),
    ]),
  };
}

function summarizeNamespace(
  namespace: Namespace,
  rows: NamespaceLanguageStats[],
): NamespaceSummary {
  const stringCount = Math.max(0, ...rows.map((row) => row.string_count));
  const requiredTranslations = stringCount * uniqueTargetLanguageCount(rows);
  const translatedCount = sum(rows, "translated_count");
  const approvedCount = sum(rows, "approved_count");

  return {
    namespace,
    stringCount,
    translatedCount,
    approvedCount,
    candidateCount: sum(rows, "candidate_count"),
    missingCount: sum(rows, "missing_count"),
    translationPercent: percent(translatedCount, requiredTranslations),
    approvalPercent: percent(approvedCount, requiredTranslations),
    updatedAt: latestDate([namespace.updated_at, ...rows.map((row) => row.updated_at)]),
  };
}

function statsForNamespace(
  stats: NamespaceLanguageStats[],
  namespaceId: number,
): NamespaceLanguageStats[] {
  return stats.filter((row) => row.namespace_id === namespaceId);
}

function uniqueTargetLanguageCount(rows: NamespaceLanguageStats[]): number {
  return new Set(rows.map((row) => row.target_language_id)).size;
}

function percent(value: number, total: number): number {
  if (total <= 0) {
    return 0;
  }

  return Math.round((value / total) * 100);
}

function sum(
  rows: NamespaceLanguageStats[],
  key: keyof Pick<
    NamespaceLanguageStats,
    | "approved_count"
    | "candidate_count"
    | "missing_count"
    | "string_count"
    | "translated_count"
  >,
): number {
  return rows.reduce((total, row) => total + row[key], 0);
}

function latestDate(values: string[]): string {
  const latest = values
    .map((value) => new Date(value).getTime())
    .filter(Number.isFinite)
    .reduce((max, value) => Math.max(max, value), 0);

  return latest > 0 ? new Date(latest).toISOString() : new Date(0).toISOString();
}

async function fetchBackend<T>(path: string): Promise<T> {
  const baseUrl = process.env.INTERNAL_API_URL ?? "http://127.0.0.1:4000";

  let response: Response;
  try {
    response = await fetch(`${baseUrl}${path}`, { cache: "no-store" });
  } catch (error) {
    const message = error instanceof Error ? error.message : "Unknown error";
    throw new ProjectFetchError(
      `Could not reach backend at ${baseUrl}: ${message}`,
    );
  }

  if (!response.ok) {
    throw new ProjectFetchError(
      `Backend returned HTTP ${response.status} for ${path}`,
      response.status,
    );
  }

  return (await response.json()) as T;
}

function normalizeProjectError<T>(error: unknown): ProjectDataResult<T> {
  if (error instanceof ProjectFetchError) {
    return {
      ok: false,
      status: error.status,
      error: error.message,
    };
  }

  const message = error instanceof Error ? error.message : "Unknown error";
  return {
    ok: false,
    error: message,
  };
}

class ProjectFetchError extends Error {
  constructor(
    message: string,
    readonly status?: number,
  ) {
    super(message);
  }
}
