// Texts and file choosers that depend on the dive application.

import { open } from "@tauri-apps/plugin-dialog";
import { t, type Key } from "./i18n";
import type { ReaderError, SourceKind } from "./types";

export const sourceName = (kind: SourceKind) => t(`source.${kind}`);

type ErrorKind = "notFound" | "notADiveLog" | "database" | "oldFormat" | "severalLogs";

/** What went wrong, as the screens tell it (other reader errors read as "database"). */
export function errorKind(kind: SourceKind, error: ReaderError): ErrorKind {
  switch (error.kind) {
    case "notFound":
    case "notADiveLog":
      return error.kind;
    case "oldFormat":
    case "severalLogs":
      return kind === "subsurface" ? error.kind : "database";
    default:
      return "database";
  }
}

/** Title or explanation of a reader error, worded for the application. */
export function errorText(kind: SourceKind, error: ReaderError, field: "title" | "text"): string {
  const prefix = kind === "subsurface" ? "errSs" : "err";
  return t(`${prefix}.${errorKind(kind, error)}.${field}` as Key);
}

/**
 * Asks for a dive log next to `current`: MacDive's database, a Subsurface
 * file, or (folder) a Subsurface git storage folder. Null when cancelled.
 */
export async function chooseLog(kind: SourceKind, current: string | undefined, folder = false): Promise<string | null> {
  const parent = current?.replace(/[/\\][^/\\]*$/, "");
  const picked = await open({
    multiple: false,
    directory: folder,
    defaultPath: (folder ? current : parent) || undefined,
    filters: folder
      ? undefined
      : kind === "subsurface"
        ? [{ name: t("errSs.dialogFilter"), extensions: ["xml", "ssrf"] }]
        : [{ name: t("err.dialogFilter"), extensions: ["sqlite"] }],
  });
  return typeof picked === "string" ? picked : null;
}
