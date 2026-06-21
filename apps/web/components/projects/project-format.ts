import type { Language } from "@/lib/projects";

export function formatNumber(value: number): string {
  return new Intl.NumberFormat("en").format(value);
}

export function formatLanguage(language: Language): string {
  return language.name || language.key;
}

export function languageInitial(language: Language): string {
  return (language.key || language.name).trim().slice(0, 2).toLowerCase();
}

export function formatRelativeTime(value: string, generatedAt: string): string {
  const then = new Date(value).getTime();
  const now = new Date(generatedAt).getTime();

  if (!Number.isFinite(then) || !Number.isFinite(now)) {
    return "Recently";
  }

  const seconds = Math.max(0, Math.round((now - then) / 1000));
  const units = [
    ["year", 60 * 60 * 24 * 365],
    ["month", 60 * 60 * 24 * 30],
    ["week", 60 * 60 * 24 * 7],
    ["day", 60 * 60 * 24],
    ["hour", 60 * 60],
    ["minute", 60],
  ] as const;

  for (const [unit, unitSeconds] of units) {
    const count = Math.floor(seconds / unitSeconds);
    if (count >= 1) {
      return `${count} ${unit}${count === 1 ? "" : "s"} ago`;
    }
  }

  return "Just now";
}
