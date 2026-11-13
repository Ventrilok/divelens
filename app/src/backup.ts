// Settings export and import (spec F11): what the Settings screen says about
// the files, apart from the screen itself.

import { t } from "./i18n";

/** Last part of a path, macOS or Windows ("Dive Lens settings.json"). */
export function fileName(path: string): string {
  return path.split(/[/\\]/).filter(Boolean).pop() ?? path;
}

/** Why a settings file was not imported (an ImportError from the back end). */
export function importErrorText(error: unknown): string {
  const e = (error ?? {}) as { kind?: string; detail?: string };
  switch (e.kind) {
    case "unreadable":
      return t("settings.importUnreadable", { detail: e.detail ?? "" });
    case "notSettings":
      return t("settings.importNotSettings");
    case "notSaved":
      return t("settings.importNotSaved", { detail: e.detail ?? "" });
    default:
      return t("settings.importNotSaved", { detail: String(error) });
  }
}
