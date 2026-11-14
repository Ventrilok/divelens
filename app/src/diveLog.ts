// One dive log at a time (spec F2): the log a screen shows, as every action
// names it, and the reads of the log, of which only the latest counts.

import { t, type Key } from "./i18n";
import type { LogRef, Settings, SourceInfo, SourceKind } from "./types";

/** The dive log a screen shows, as the back end checks it: application and location. */
export const logRef = (source: SourceInfo): LogRef => ({ kind: source.kind, path: source.path });

/** The back end refused an action: the dive log is no longer the one the screen shows. */
export function isLogChanged(error: unknown): boolean {
  return typeof error === "object" && error !== null && (error as { error?: unknown }).error === "logChanged";
}

/**
 * What the screen the diver lands on says after a refusal because the dive
 * log changed: an Apply refused says nothing was written, and wins.
 */
export function logChangedNotice(current: Key | null, applied: boolean): Key {
  return applied ? "log.changedNothingWritten" : (current ?? "log.changed");
}

/**
 * Whether a change of the dive-log settings leaves the log in use alone:
 * only the location of the other application changed (an import, the
 * previous plug-in's settings). `kindInUse` is the application of the log
 * shown, as the back end resolved it. Nothing needs reading again then.
 */
export function sameLogSetting(before: Settings["source"], after: Settings["source"], kindInUse: SourceKind): boolean {
  const path = (s: Settings["source"]) => (kindInUse === "subsurface" ? s.subsurfacePath : s.macDivePath);
  return before.kind === after.kind && path(before) === path(after);
}

let lastSearch = Date.now();
/**
 * Number of a photo search the screen asks for now, higher than any before
 * (also across a reload of the window): the back end keeps the latest only.
 */
export const nextSearchNumber = () => ++lastSearch;

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
 * Follows the dive log shown, read after read: the application in use and
 * its location as the back end resolved them, the same log every action is
 * checked against. Tells whether a read shows another log than the previous
 * one, which starts every flow again. The first read is no change, nor is a
 * read of the same log again (Retry, the standard location chosen for the
 * file that already was the standard one, a setting of the other
 * application changed).
 */
export function logWatch() {
  let shown: LogRef | null = null;
  return (source: SourceInfo): boolean => {
    const changed = shown !== null && (shown.kind !== source.kind || shown.path !== source.path);
    shown = logRef(source);
    return changed;
  };
}
