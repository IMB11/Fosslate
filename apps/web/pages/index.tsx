import type { GetServerSideProps, InferGetServerSidePropsType } from "next";
import Link from "next/link";
import {
  Clock3,
  FolderTree,
  Languages,
  MessageSquareText,
  type LucideIcon,
} from "lucide-react";

import { NewProjectDialog } from "@/components/projects/NewProjectDialog";
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
import { Card } from "@/components/retroui/Card";
import { Text } from "@/components/retroui/Text";
import type { AuthUser } from "@/lib/auth-client";
import {
  getProjectSummaries,
  type ProjectDataResult,
  type ProjectSummary,
} from "@/lib/projects";
import { cn } from "@/lib/utils";

type HomeProps = {
  generatedAt: string;
  initialAuthUser?: AuthUser | null;
  projects: ProjectDataResult<ProjectSummary[]>;
};

export const getServerSideProps = (async () => {
  return {
    props: {
      generatedAt: new Date().toISOString(),
      projects: await getProjectSummaries(),
    },
  };
}) satisfies GetServerSideProps<HomeProps>;

export default function ProjectsPage({
  generatedAt,
  initialAuthUser,
  projects,
}: InferGetServerSidePropsType<typeof getServerSideProps>) {
  const projectCount = projects.ok ? projects.data.length : 0;
  const canCreateProjects = Boolean(initialAuthUser?.is_admin);

  return (
    <main className="min-h-[calc(100vh-5rem)] bg-background text-foreground">
      <div className="mx-auto w-full max-w-7xl space-y-7 px-4 py-8 sm:px-6 lg:px-10">
        <header className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
          <div className="space-y-2">
            <Text as="h1" className="text-4xl">
              Projects
            </Text>
          </div>
          {canCreateProjects ? <NewProjectDialog /> : null}
        </header>

        {!projects.ok ? (
          <Alert status="error">
            <Alert.Description className="text-sm">
              {projects.error}
            </Alert.Description>
          </Alert>
        ) : null}

        {projects.ok && projectCount === 0 ? (
          <section className="border-2 border-border bg-card p-8 shadow-md">
            <div className="max-w-xl space-y-2">
              <h2 className="font-head text-2xl font-bold">No projects yet</h2>
              <p className="text-sm leading-6 text-muted-foreground">
                Create a project from the API to start organizing namespaces,
                source strings, and target languages.
              </p>
            </div>
          </section>
        ) : null}

        {projects.ok && projectCount > 0 ? (
          <section
            aria-label="Projects"
            className="grid grid-cols-[repeat(auto-fit,minmax(min(100%,22rem),1fr))] gap-5"
          >
            {projects.data.map((project) => (
              <ProjectCard
                generatedAt={generatedAt}
                key={project.project.public_id}
                project={project}
              />
            ))}
          </section>
        ) : null}
      </div>
    </main>
  );
}

function ProjectCard({
  generatedAt,
  project,
}: {
  generatedAt: string;
  project: ProjectSummary;
}) {
  const visibleLanguages = project.targetLanguages.slice(0, 8);
  const hiddenLanguageCount = Math.max(
    0,
    project.targetLanguages.length - visibleLanguages.length,
  );

  return (
    <Card className="block h-full w-full">
      <Card.Content className="flex h-full flex-col gap-5 p-5">
        <div className="flex items-start justify-between gap-4">
          <div className="min-w-0 space-y-2">
            <h2 className="break-words font-head text-2xl font-bold leading-tight">
              {project.project.name}
            </h2>
            <p className="line-clamp-2 text-sm leading-6 text-muted-foreground">
              Source language: {formatLanguage(project.project.source_language)}
            </p>
          </div>
          <Badge
            className="shrink-0 border-2 border-border bg-background text-foreground"
            size="sm"
            variant="default"
          >
            {languageInitial(project.project.source_language)}
          </Badge>
        </div>

        <div className="flex flex-wrap gap-x-4 gap-y-2 text-sm text-muted-foreground">
          <ProjectStat
            icon={FolderTree}
            label={`${formatNumber(project.namespaceCount)} namespaces`}
          />
          <ProjectStat
            icon={MessageSquareText}
            label={`${formatNumber(project.sourceStringCount)} strings`}
          />
          <ProjectStat
            icon={Languages}
            label={`${formatNumber(project.targetLanguages.length)} languages`}
          />
          <ProjectStat
            icon={Clock3}
            label={formatRelativeTime(project.updatedAt, generatedAt)}
          />
        </div>

        <div className="space-y-2">
          <p className="font-head text-xs font-bold uppercase tracking-normal text-muted-foreground">
            Target languages
          </p>
          {project.targetLanguages.length > 0 ? (
            <div className="flex flex-wrap gap-2">
              {visibleLanguages.map((targetLanguage) => (
                <Badge
                  className="border-2 border-border bg-background text-foreground"
                  key={targetLanguage.id}
                  size="sm"
                >
                  {languageInitial(targetLanguage.language)}
                </Badge>
              ))}
              {hiddenLanguageCount > 0 ? (
                <Badge
                  className="border-2 border-border bg-primary text-primary-foreground"
                  size="sm"
                >
                  +{hiddenLanguageCount}
                </Badge>
              ) : null}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">
              No target languages configured.
            </p>
          )}
        </div>

        <div className="mt-auto space-y-3">
          <ProgressMeter
            approved={project.approvalPercent}
            translated={project.translationPercent}
          />
          <p className="text-sm text-muted-foreground">
            {project.translationPercent}% translated across{" "}
            {formatNumber(project.targetLanguages.length)} languages
          </p>
        </div>

        <div className="flex justify-end">
          <Link
            className={cn(buttonVariants({ size: "sm" }), "min-w-24")}
            href={`/projects/${project.project.public_id}`}
          >
            Open
          </Link>
        </div>
      </Card.Content>
    </Card>
  );
}

function ProjectStat({
  icon: Icon,
  label,
}: {
  icon: LucideIcon;
  label: string;
}) {
  return (
    <span className="inline-flex items-center gap-1.5">
      <Icon aria-hidden="true" className="size-4" />
      <span>{label}</span>
    </span>
  );
}
