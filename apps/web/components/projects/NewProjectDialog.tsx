import { useMutation } from "@tanstack/react-query";
import { Field as BaseField } from "@base-ui/react/field";
import { FolderPlus, Plus } from "lucide-react";
import { useRouter } from "next/router";
import type { FormEvent, ReactNode } from "react";
import { useState } from "react";

import { Alert } from "@/components/retroui/Alert";
import { Button } from "@/components/retroui/Button";
import { Dialog } from "@/components/retroui/Dialog";
import { Input } from "@/components/retroui/Input";
import { Label } from "@/components/retroui/Label";
import {
  createProject,
  projectErrorMessage,
} from "@/lib/projects-client";

export function NewProjectDialog() {
  const router = useRouter();
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const [sourceLanguageKey, setSourceLanguageKey] = useState("en");
  const [sourceLanguageName, setSourceLanguageName] = useState("English");
  const [error, setError] = useState<string | null>(null);

  const mutation = useMutation({
    mutationFn: createProject,
    onSuccess: async (project) => {
      setError(null);
      setOpen(false);
      await router.push(`/projects/${project.public_id}`);
    },
    onError: (error) => setError(projectErrorMessage(error)),
  });

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    const trimmedName = name.trim();
    const trimmedLanguageKey = sourceLanguageKey.trim();
    const trimmedLanguageName = sourceLanguageName.trim();

    if (!trimmedName || !trimmedLanguageKey || !trimmedLanguageName) {
      setError("Project name and source language are required.");
      return;
    }

    mutation.mutate({
      name: trimmedName,
      icon_asset_id: null,
      source_language: {
        key: trimmedLanguageKey,
        name: trimmedLanguageName,
      },
    });
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <Dialog.Trigger className="inline-flex w-fit items-center justify-center gap-2 rounded border-2 border-black bg-primary px-4 py-1.5 font-head font-medium text-primary-foreground shadow-md transition hover:translate-y-1 hover:bg-primary-hover hover:shadow active:translate-x-1 active:translate-y-2 active:shadow-none focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary">
        <Plus aria-hidden="true" className="size-4" />
        New project
      </Dialog.Trigger>
      <Dialog.Content className="max-h-[calc(100vh-2rem)] max-w-[calc(100vw-2rem)] sm:max-w-lg">
        <Dialog.Header>
          <div className="flex items-center gap-2 font-head text-lg font-bold">
            <FolderPlus aria-hidden="true" className="size-5" />
            New project
          </div>
        </Dialog.Header>

        <form onSubmit={handleSubmit}>
          <div className="space-y-5 p-5">
            <Dialog.Description className="text-sm leading-6 text-muted-foreground">
              Create a translation workspace with a source language for original
              strings.
            </Dialog.Description>

            {error ? (
              <Alert status="error">
                <Alert.Description className="text-sm">{error}</Alert.Description>
              </Alert>
            ) : null}

            <ProjectField label="Project name">
              <Input
                autoComplete="off"
                autoFocus
                onChange={(event) => {
                  setName(event.target.value);
                  setError(null);
                }}
                placeholder="Fosslate"
                value={name}
              />
            </ProjectField>

            <div className="grid gap-4 sm:grid-cols-[8rem_minmax(0,1fr)]">
              <ProjectField label="Language key">
                <Input
                  autoComplete="off"
                  onChange={(event) => {
                    setSourceLanguageKey(event.target.value);
                    setError(null);
                  }}
                  placeholder="en"
                  value={sourceLanguageKey}
                />
              </ProjectField>

              <ProjectField label="Language name">
                <Input
                  autoComplete="off"
                  onChange={(event) => {
                    setSourceLanguageName(event.target.value);
                    setError(null);
                  }}
                  placeholder="English"
                  value={sourceLanguageName}
                />
              </ProjectField>
            </div>
          </div>

          <Dialog.Footer>
            <Dialog.Close
              className="font-head text-sm hover:underline"
              disabled={mutation.isPending}
              type="button"
            >
              Cancel
            </Dialog.Close>
            <Button disabled={mutation.isPending} type="submit">
              {mutation.isPending ? "Creating..." : "Create project"}
            </Button>
          </Dialog.Footer>
        </form>
      </Dialog.Content>
    </Dialog>
  );
}

function ProjectField({
  children,
  label,
}: {
  children: ReactNode;
  label: string;
}) {
  return (
    <BaseField.Root className="space-y-2">
      <Label className="font-head text-sm font-bold">{label}</Label>
      {children}
    </BaseField.Root>
  );
}
