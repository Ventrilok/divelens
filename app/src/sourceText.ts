// Texts and file choosers of the dive applications, as the back end
// describes each one (core/src/apps.rs): no screen knows any application.

import { open } from "@tauri-apps/plugin-dialog";
import { fill, lang, t } from "./i18n";
import type { AppTexts, DiveApp, ReaderError, Settings, SourceKind } from "./types";

let apps: DiveApp[] = [];

/** The dive applications available on this system, in the order Settings lists them (set once, at start). */
export function setApps(list: DiveApp[]): void {
  apps = list;
}

export const availableApps = (): readonly DiveApp[] => apps;

/**
 * The application used for `kind`, as the back end resolves it: that one when
 * this system has it, else the first one available.
 */
export function appOf(kind: SourceKind): DiveApp {
  return apps.find((a) => a.id === kind) ?? apps[0];
}

/** What the screens say of `app`, in the interface language. */
export const appTexts = (app: DiveApp): AppTexts => app.texts[lang];

export const sourceName = (kind: SourceKind) => appOf(kind).name;

/** What went wrong, as the screens tell it: the reader's error when the application explains it, else "database". */
export function errorKind(app: DiveApp, error: ReaderError): string {
  return error.kind in appTexts(app).problems ? error.kind : "database";
}

/** Title or explanation of a reader error, worded for the application. */
export function errorText(app: DiveApp, error: ReaderError, field: "title" | "text"): string {
  return appTexts(app).problems[errorKind(app, error)][field];
}

/** What Settings says once the log is read, with its counts. */
export function readText(app: DiveApp, format: string, counts: { dives: string; sites: string }): string {
  const read = appTexts(app).read;
  return fill(read[format] ?? Object.values(read)[0], counts);
}

/** The buttons choosing a log of `app`: a file and a folder, or the one kind it is. */
export function chooseButtons(app: DiveApp, single: string = t("settings.choose")): { folder: boolean; label: string }[] {
  const file = app.choose.file !== null;
  if (file && app.choose.folder) {
    return [
      { folder: false, label: t("settings.chooseFile") },
      { folder: true, label: t("settings.chooseFolder") },
    ];
  }
  return [{ folder: !file, label: single }];
}

/** The settings' locations with `app`'s set to `path` (null: its standard location). */
export function withLocation(source: Settings["source"], app: DiveApp, path: string | null): Settings["source"] {
  const locations = { ...source.locations };
  if (path === null) delete locations[app.id];
  else locations[app.id] = path;
  return { ...source, locations };
}

/** Asks for a log of `app` next to `current`: a file, or (folder) a folder. Null when cancelled. */
export async function chooseLog(app: DiveApp, current: string | undefined, folder = false): Promise<string | null> {
  const parent = current?.replace(/[/\\][^/\\]*$/, "");
  const picked = await open({
    multiple: false,
    directory: folder,
    defaultPath: (folder ? current : parent) || undefined,
    filters: folder ? undefined : [{ name: appTexts(app).fileType, extensions: app.choose.file ?? [] }],
  });
  return typeof picked === "string" ? picked : null;
}
