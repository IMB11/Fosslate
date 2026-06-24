import type {
  GetServerSideProps,
  GetServerSidePropsContext,
  InferGetServerSidePropsType,
} from "next";
import Link from "next/link";
import type { ReactNode } from "react";
import {
  Activity,
  ArrowLeft,
  CheckCircle2,
  ExternalLink,
  FolderTree,
  Languages,
  ListTree,
  MessageSquareText,
  MoreHorizontal,
  Users,
  type LucideIcon,
} from "lucide-react";

import { ProgressMeter } from "@/components/projects/ProgressMeter";
import {
  formatLanguage,
  formatNumber,
  formatRelativeTime,
  languageInitial,
} from "@/components/projects/project-format";
import { Alert } from "@/components/retroui/Alert";
import { Badge } from "@/components/retroui/Badge";
import { buttonVariants } from "@/components/retroui/Button";
import { Text } from "@/components/retroui/Text";
import {
  getProjectDetail,
  type NamespaceLanguageStats,
  type NamespaceSummary,
  type ProjectDataResult,
  type ProjectDetail,
  type ProjectTargetLanguage,
} from "@/lib/projects";
import { cn } from "@/lib/utils";

type ProjectPageProps = {
  generatedAt: string;
  project: ProjectDataResult<ProjectDetail>;
  projectPublicId: string;
  selectedLanguageKey: string | null;
};

type CoverageSummary = {
  approvedCount: number;
  approvalPercent: number;
  candidateCount: number;
  missingCount: number;
  sourceStringCount: number;
  translatedCount: number;
  translationPercent: number;
};

type VisibleNamespace = NamespaceSummary & {
  awaitingApprovalCount: number;
};

export const getServerSideProps = (async (
  context: GetServerSidePropsContext,
) => {
  const projectPublicId =
    typeof context.params?.projectPublicId === "string"
      ? context.params.projectPublicId
      : null;

  if (!projectPublicId) {
    return { notFound: true };
  }

  const project = await getProjectDetail(projectPublicId);

  if (!project.ok && project.status === 404) {
    return { notFound: true };
  }

  return {
    props: {
      generatedAt: new Date().toISOString(),
      project,
      projectPublicId,
      selectedLanguageKey:
        typeof context.query.language === "string"
          ? context.query.language
          : null,
    },
  };
}) satisfies GetServerSideProps<ProjectPageProps>;

export default function ProjectPage({
  generatedAt,
  project,
  projectPublicId,
  selectedLanguageKey,
}: InferGetServerSidePropsType<typeof getServerSideProps>) {
  if (!project.ok) {
    return (
      <ProjectShell>
        <Alert status="error">
          <Alert.Description className="text-sm">
            {project.error}
          </Alert.Description>
        </Alert>
      </ProjectShell>
    );
  }

  const selectedLanguage =
    project.data.targetLanguages.find(
      (targetLanguage) => targetLanguage.language.key === selectedLanguageKey,
    ) ?? null;
  const coverage = selectedLanguage
    ? summarizeLanguageCoverage(project.data.stats, selectedLanguage)
    : summarizeAllLanguageCoverage(project.data);
  const namespaces = selectedLanguage
    ? project.data.namespaces.map((namespace) =>
        summarizeVisibleNamespace(
          namespace,
          project.data.stats.find(
            (row) =>
              row.namespace_id === namespace.namespace.id &&
              row.target_language_id === selectedLanguage.id,
          ) ?? null,
        ),
      )
    : project.data.namespaces.map((namespace) => ({
        ...namespace,
        awaitingApprovalCount: awaitingApproval(
          namespace.candidateCount,
          namespace.approvedCount,
        ),
      }));

  return (
    <ProjectShell>
      <div className="space-y-6">
        <Link
          className="inline-flex items-center gap-2 text-sm text-muted-foreground hover:underline"
          href="/"
        >
          <ArrowLeft aria-hidden="true" className="size-4" />
          Projects
        </Link>

        <ProjectHeader
          generatedAt={generatedAt}
          project={project.data}
          selectedLanguage={selectedLanguage}
        />

        <div className="flex gap-2 overflow-x-auto border-b-2 border-border pb-3">
          <ProjectTab active icon={ListTree} label="Namespaces" />
          <ProjectTab icon={Users} label="Contributors" />
          <ProjectTab icon={Activity} label="Activity log" />
        </div>

        <LanguageFilters
          project={project.data}
          projectPublicId={projectPublicId}
          selectedLanguage={selectedLanguage}
        />

        <StatsGrid coverage={coverage} />

        <NamespaceList
          generatedAt={generatedAt}
          namespaces={namespaces}
          selectedLanguage={selectedLanguage}
        />
      </div>
    </ProjectShell>
  );
}

function ProjectShell({ children }: { children: ReactNode }) {
  return (
    <main className="min-h-[calc(100vh-5rem)] bg-background text-foreground">
      <div className="mx-auto w-full max-w-7xl px-4 py-8 sm:px-6 lg:px-10">
        {children}
      </div>
    </main>
  );
}

function ProjectHeader({
  generatedAt,
  project,
  selectedLanguage,
}: {
  generatedAt: string;
  project: ProjectDetail;
  selectedLanguage: ProjectTargetLanguage | null;
}) {
  const visibleLanguages = project.targetLanguages.slice(0, 5);

  return (
    <header className="flex flex-col gap-5 border-2 border-border bg-card p-5 shadow-md lg:flex-row lg:items-center lg:justify-between">
      <div className="flex min-w-0 gap-4">
        <div className="flex size-14 shrink-0 items-center justify-center border-2 border-border bg-primary font-head text-2xl font-bold text-primary-foreground">
          {project.project.name.trim().charAt(0).toUpperCase() || "F"}
        </div>
        <div className="min-w-0 space-y-2">
          <div className="flex flex-wrap items-center gap-3">
            <Text as="h1" className="break-words text-3xl">
              {project.project.name}
            </Text>
            {selectedLanguage ? (
              <Badge className="border-2 border-border bg-background" size="sm">
                {formatLanguage(selectedLanguage.language)}
              </Badge>
            ) : null}
          </div>
          <p className="text-sm text-muted-foreground">
            Source language: {formatLanguage(project.project.source_language)}
          </p>
          <p className="text-sm text-muted-foreground">
            Updated {formatRelativeTime(project.updatedAt, generatedAt)}
          </p>
        </div>
      </div>

      <div className="flex flex-col gap-3 sm:flex-row sm:items-center">
        {visibleLanguages.length > 0 ? (
          <div className="flex flex-wrap gap-2 lg:justify-end">
            {visibleLanguages.map((targetLanguage) => (
              <Badge
                className="border-2 border-border bg-background text-foreground"
                key={targetLanguage.id}
                size="sm"
              >
                {languageInitial(targetLanguage.language)}
              </Badge>
            ))}
          </div>
        ) : null}
        <Link
          className={cn(buttonVariants({ size: "icon", variant: "outline" }), "bg-background")}
          href="#"
          aria-label="Project actions"
        >
          <MoreHorizontal aria-hidden="true" className="size-5" />
        </Link>
      </div>
    </header>
  );
}

function ProjectTab({
  active = false,
  icon: Icon,
  label,
}: {
  active?: boolean;
  icon: LucideIcon;
  label: string;
}) {
  return (
    <button
      aria-current={active ? "page" : undefined}
      className={cn(
        "flex min-h-10 shrink-0 items-center gap-2 border-2 border-transparent px-3 py-2 font-head text-sm transition",
        "hover:border-border hover:bg-accent",
        active && "border-border bg-primary text-primary-foreground shadow-sm",
      )}
      type="button"
    >
      <Icon aria-hidden="true" className="size-4" />
      {label}
    </button>
  );
}

function LanguageFilters({
  project,
  projectPublicId,
  selectedLanguage,
}: {
  project: ProjectDetail;
  projectPublicId: string;
  selectedLanguage: ProjectTargetLanguage | null;
}) {
  return (
    <section className="flex flex-col gap-3 sm:flex-row sm:items-center">
      <span className="font-head text-sm font-bold">Language:</span>
      <div className="flex flex-wrap gap-2">
        <LanguageChip
          active={!selectedLanguage}
          href={`/projects/${projectPublicId}`}
          label="All languages"
        />
        {project.targetLanguages.map((targetLanguage) => (
          <LanguageChip
            active={selectedLanguage?.id === targetLanguage.id}
            href={`/projects/${projectPublicId}?language=${encodeURIComponent(
              targetLanguage.language.key,
            )}`}
            key={targetLanguage.id}
            label={formatLanguage(targetLanguage.language)}
          />
        ))}
      </div>
    </section>
  );
}

function LanguageChip({
  active,
  href,
  label,
}: {
  active: boolean;
  href: string;
  label: string;
}) {
  return (
    <Link
      className={cn(
        "border-2 border-border px-3 py-1 font-head text-sm shadow-xs transition hover:translate-y-0.5 hover:bg-accent",
        active ? "bg-primary text-primary-foreground" : "bg-background",
      )}
      href={href}
    >
      {label}
    </Link>
  );
}

function StatsGrid({ coverage }: { coverage: CoverageSummary }) {
  const stats = [
    {
      label: "Source strings",
      value: coverage.sourceStringCount,
      icon: MessageSquareText,
    },
    {
      label: "Translated",
      value: `${coverage.translationPercent}%`,
      icon: Languages,
    },
    {
      label: "Approved",
      value: `${coverage.approvalPercent}%`,
      icon: CheckCircle2,
    },
    {
      label: "Missing",
      value: coverage.missingCount,
      icon: FolderTree,
    },
    {
      label: "Awaiting approval",
      value: awaitingApproval(coverage.candidateCount, coverage.approvedCount),
      icon: Users,
    },
    {
      label: "Candidates",
      value: coverage.candidateCount,
      icon: Activity,
    },
  ];

  return (
    <section
      aria-label="Project stats"
      className="grid grid-cols-2 border-2 border-border bg-card shadow-md md:grid-cols-3 xl:grid-cols-6"
    >
      {stats.map((stat, index) => {
        const Icon = stat.icon;

        return (
          <div
            className={cn(
              "space-y-1 border-border p-4",
              index % 2 === 1 && "border-l-2 md:border-l-0",
              index >= 2 && "border-t-2 md:border-t-0",
              index >= 3 && "md:border-t-2 xl:border-t-0",
              index % 3 !== 0 && "md:border-l-2",
              index % 3 !== 0 && "xl:border-l-0",
              index !== 0 && "xl:border-l-2",
            )}
            key={stat.label}
          >
            <div className="flex items-center gap-2 text-muted-foreground">
              <Icon aria-hidden="true" className="size-4" />
              <span className="text-xs font-bold uppercase tracking-normal">
                {stat.label}
              </span>
            </div>
            <p className="font-head text-2xl font-bold">
              {typeof stat.value === "number"
                ? formatNumber(stat.value)
                : stat.value}
            </p>
          </div>
        );
      })}
    </section>
  );
}

function NamespaceList({
  generatedAt,
  namespaces,
  selectedLanguage,
}: {
  generatedAt: string;
  namespaces: VisibleNamespace[];
  selectedLanguage: ProjectTargetLanguage | null;
}) {
  if (namespaces.length === 0) {
    return (
      <section className="border-2 border-border bg-card p-6 shadow-md">
        <h2 className="font-head text-2xl font-bold">No namespaces yet</h2>
        <p className="mt-2 text-sm text-muted-foreground">
          Add namespaces to organize source strings for this project.
        </p>
      </section>
    );
  }

  return (
    <section className="overflow-hidden border-2 border-border bg-card shadow-md">
      <div className="hidden grid-cols-[minmax(12rem,1fr)_minmax(16rem,2fr)_12rem_8rem_8rem] gap-4 border-b-2 border-border px-5 py-3 font-head text-xs font-bold uppercase tracking-normal text-muted-foreground lg:grid">
        <span>Namespace</span>
        <span>Coverage</span>
        <span>Open work</span>
        <span>Updated</span>
        <span className="text-right">Actions</span>
      </div>
      <div className="divide-y-2 divide-border">
        {namespaces.map((namespace) => (
          <NamespaceRow
            generatedAt={generatedAt}
            key={namespace.namespace.id}
            namespace={namespace}
            selectedLanguage={selectedLanguage}
          />
        ))}
      </div>
    </section>
  );
}

function NamespaceRow({
  generatedAt,
  namespace,
  selectedLanguage,
}: {
  generatedAt: string;
  namespace: VisibleNamespace;
  selectedLanguage: ProjectTargetLanguage | null;
}) {
  return (
    <article className="grid gap-4 px-5 py-5 lg:grid-cols-[minmax(12rem,1fr)_minmax(16rem,2fr)_12rem_8rem_8rem] lg:items-center">
      <div className="min-w-0">
        <h2 className="break-words font-head text-xl font-bold">
          {namespace.namespace.name}
        </h2>
        <p className="mt-1 text-sm text-muted-foreground">
          {formatNumber(namespace.stringCount)} strings
        </p>
      </div>

      <div className="space-y-2">
        <div className="flex justify-between gap-3 text-sm text-muted-foreground">
          <span>{namespace.translationPercent}% translated</span>
          <span>{namespace.approvalPercent}% approved</span>
        </div>
        <ProgressMeter
          approved={namespace.approvalPercent}
          translated={namespace.translationPercent}
        />
      </div>

      <div className="grid grid-cols-3 gap-3 text-center lg:grid-cols-3">
        <MiniMetric label="Missing" value={namespace.missingCount} />
        <MiniMetric label="Awaiting" value={namespace.awaitingApprovalCount} />
        <MiniMetric label="Candidates" value={namespace.candidateCount} />
      </div>

      <p className="text-sm text-muted-foreground lg:text-right">
        {formatRelativeTime(namespace.updatedAt, generatedAt)}
      </p>

      <div className="flex justify-end gap-2">
        <Link
          className={cn(buttonVariants({ size: "sm" }), "gap-2")}
          href="#"
        >
          Open
          <ExternalLink aria-hidden="true" className="size-4" />
        </Link>
        <Link
          aria-label={`More actions for ${namespace.namespace.name}`}
          className={cn(
            buttonVariants({ size: "icon", variant: "outline" }),
            "bg-background",
          )}
          href="#"
        >
          <MoreHorizontal aria-hidden="true" className="size-4" />
        </Link>
      </div>

      {selectedLanguage ? (
        <span className="sr-only">
          Showing {formatLanguage(selectedLanguage.language)} only.
        </span>
      ) : null}
    </article>
  );
}

function MiniMetric({ label, value }: { label: string; value: number }) {
  return (
    <div>
      <p className="font-head text-base font-bold">{formatNumber(value)}</p>
      <p className="text-xs text-muted-foreground">{label}</p>
    </div>
  );
}

function summarizeAllLanguageCoverage(project: ProjectDetail): CoverageSummary {
  return {
    approvedCount: project.approvedCount,
    approvalPercent: project.approvalPercent,
    candidateCount: project.candidateCount,
    missingCount: project.missingCount,
    sourceStringCount: project.sourceStringCount,
    translatedCount: project.translatedCount,
    translationPercent: project.translationPercent,
  };
}

function summarizeLanguageCoverage(
  stats: NamespaceLanguageStats[],
  selectedLanguage: ProjectTargetLanguage,
): CoverageSummary {
  const rows = stats.filter(
    (row) => row.target_language_id === selectedLanguage.id,
  );
  const sourceStringCount = rows.reduce(
    (total, row) => total + row.string_count,
    0,
  );
  const translatedCount = rows.reduce(
    (total, row) => total + row.translated_count,
    0,
  );
  const approvedCount = rows.reduce(
    (total, row) => total + row.approved_count,
    0,
  );

  return {
    approvedCount,
    approvalPercent: percent(approvedCount, sourceStringCount),
    candidateCount: rows.reduce((total, row) => total + row.candidate_count, 0),
    missingCount: rows.reduce((total, row) => total + row.missing_count, 0),
    sourceStringCount,
    translatedCount,
    translationPercent: percent(translatedCount, sourceStringCount),
  };
}

function summarizeVisibleNamespace(
  namespace: NamespaceSummary,
  row: NamespaceLanguageStats | null,
): VisibleNamespace {
  if (!row) {
    return {
      ...namespace,
      approvedCount: 0,
      approvalPercent: 0,
      awaitingApprovalCount: 0,
      candidateCount: 0,
      missingCount: 0,
      stringCount: 0,
      translatedCount: 0,
      translationPercent: 0,
    };
  }

  return {
    ...namespace,
    approvedCount: row.approved_count,
    approvalPercent: percent(row.approved_count, row.string_count),
    awaitingApprovalCount: awaitingApproval(
      row.candidate_count,
      row.approved_count,
    ),
    candidateCount: row.candidate_count,
    missingCount: row.missing_count,
    stringCount: row.string_count,
    translatedCount: row.translated_count,
    translationPercent: percent(row.translated_count, row.string_count),
    updatedAt: row.updated_at,
  };
}

function awaitingApproval(candidateCount: number, approvedCount: number): number {
  return Math.max(0, candidateCount - approvedCount);
}

function percent(value: number, total: number): number {
  if (total <= 0) {
    return 0;
  }

  return Math.round((value / total) * 100);
}
