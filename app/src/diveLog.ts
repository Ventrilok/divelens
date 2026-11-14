// One dive log at a time (spec F2): the log a screen shows, as every action
// names it, and the reads of the log, of which only the latest counts.

import { t } from "./i18n";
import type { LogRef, Settings, SourceInfo } from "./types";

/** The dive log a screen shows, as the back end checks it: application and location. */
export const logRef = (source: SourceInfo): LogRef => ({ kind: source.kind, path: source.path });

/** The back end refused an action: the dive log is no longer the one the screen shows. */
export function isLogChanged(error: unknown): boolean {
  return typeof error === "object" && error !== null && (error as { error?: unknown }).error === "logChanged";
}

/** Why a setting was not changed (a ChangeError from the back end). */
export function changeErrorText(error: unknown): string {
  const e = (error ?? {}) as { kind?: string; detail?: string };
  if (e.kind === "writeInProgress") return t("settings.logLocked");
  return t("settings.saveError", { detail: e.kind === "notSaved" ? (e.detail ?? "") : String(error) });
}

/**
 * Runs tasks so that only the last one started is heard of: an older one
 * finishing later is ignored (a slow read of the previous dive log never
 * replaces the newer one).
 */
export function latestOnly() {
  let last = 0;
  return function run<T>(task: Promise<T>, onDone: (value: T) => void, onFail: (error: unknown) => void): void {
    const mine = ++last;
    task.then(
      (value) => {
        if (mine === last) onDone(value);
      },
      (error: unknown) => {
        if (mine === last) onFail(error);
      },
    );
  };
}

/**
 * Follows the dive-log settings (application and locations) the flows were
 * started with: tells, change after change, whether they now point at
 * another log, which starts every flow again. Reading the same log again
 * (Retry) is not a change.
 */
export function sourceWatch(source: Settings["source"]) {
  let current = JSON.stringify(source);
  return (next: Settings["source"]): boolean => {
    const key = JSON.stringify(next);
    const changed = key !== current;
    current = key;
    return changed;
  };
}
