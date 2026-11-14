import { beforeEach, describe, expect, it } from "vitest";
import { setLanguage } from "./i18n";
import { appOf, chooseButtons, errorKind, errorText, readText, setApps, withLocation } from "./sourceText";
import type { AppTexts, DiveApp } from "./types";

// Fictional applications, described as the back end describes them
// (core/src/apps.rs): one whose log is a file, one whose log is a file or a
// folder and that explains two more reader errors.
const texts = (lang: string, extra: AppTexts["problems"] = {}): AppTexts => ({
  log: `${lang} log`,
  logHint: `${lang} hint`,
  reading: `${lang} reading`,
  read: { db: `${lang} valid · {dives} · {sites}`, folder: `${lang} folder · {dives} · {sites}` },
  fileType: `${lang} file`,
  locate: `${lang} locate`,
  problems: {
    notFound: { title: `${lang} not found`, text: `${lang} not found text` },
    notADiveLog: { title: `${lang} not a log`, text: `${lang} not a log text` },
    database: { title: `${lang} unreadable`, text: `${lang} unreadable text` },
    ...extra,
  },
});
const fileApp: DiveApp = {
  id: "fileLog",
  name: "FileLog",
  choose: { file: ["flog"], folder: false },
  texts: { en: texts("en"), fr: texts("fr") },
};
const own = { oldFormat: { title: "old", text: "old text" }, severalLogs: { title: "several", text: "several text" } };
const eitherApp: DiveApp = {
  id: "eitherLog",
  name: "EitherLog",
  choose: { file: ["elog", "xml"], folder: true },
  texts: { en: { ...texts("en", own), locate: null }, fr: { ...texts("fr", own), locate: null } },
};
const folderApp: DiveApp = { ...fileApp, id: "folderLog", choose: { file: null, folder: true } };

beforeEach(() => {
  setLanguage("en");
  setApps([fileApp, eitherApp]);
});

describe("errorKind", () => {
  it("keeps an application's own errors for that application only", () => {
    expect(errorKind(eitherApp, { kind: "severalLogs", detail: "" })).toBe("severalLogs");
    expect(errorKind(eitherApp, { kind: "oldFormat", detail: "" })).toBe("oldFormat");
    expect(errorKind(fileApp, { kind: "oldFormat", detail: "" })).toBe("database");
  });

  it("reads other reader errors as a database error", () => {
    expect(errorKind(fileApp, { kind: "notFound", detail: "" })).toBe("notFound");
    expect(errorKind(eitherApp, { kind: "notADiveLog", detail: "" })).toBe("notADiveLog");
    expect(errorKind(eitherApp, { kind: "badProfile", detail: "" })).toBe("database");
  });

  it("words the error for the application, in the interface language", () => {
    expect(errorText(fileApp, { kind: "notFound", detail: "" }, "title")).toBe("en not found");
    expect(errorText(eitherApp, { kind: "severalLogs", detail: "" }, "text")).toBe("several text");
    setLanguage("fr");
    expect(errorText(fileApp, { kind: "unknownDive", detail: "" }, "title")).toBe("fr unreadable");
  });
});

describe("the dive applications", () => {
  it("uses the application chosen when this system has it, else the first one", () => {
    expect(appOf("eitherLog")).toBe(eitherApp);
    expect(appOf("fileLog")).toBe(fileApp);
    // Settings from another system or a newer version.
    expect(appOf("blueDive")).toBe(fileApp);
    setApps([eitherApp]);
    expect(appOf("fileLog")).toBe(eitherApp);
  });

  it("says what was read, with the counts", () => {
    const counts = { dives: "3 dives", sites: "1 site" };
    expect(readText(fileApp, "db", counts)).toBe("en valid · 3 dives · 1 site");
    expect(readText(fileApp, "folder", counts)).toBe("en folder · 3 dives · 1 site");
    expect(readText(fileApp, "newer", counts), "a format this version does not know").toBe("en valid · 3 dives · 1 site");
    setLanguage("fr");
    expect(readText(fileApp, "db", counts)).toBe("fr valid · 3 dives · 1 site");
  });

  it("offers a file and a folder, or the one kind of log", () => {
    expect(chooseButtons(eitherApp)).toEqual([
      { folder: false, label: "Choose a file…" },
      { folder: true, label: "Choose a folder…" },
    ]);
    expect(chooseButtons(fileApp)).toEqual([{ folder: false, label: "Choose…" }]);
    expect(chooseButtons(fileApp, "Locate…")).toEqual([{ folder: false, label: "Locate…" }]);
    expect(chooseButtons(folderApp)).toEqual([{ folder: true, label: "Choose…" }]);
    setLanguage("fr");
    expect(chooseButtons(eitherApp).map((b) => b.label)).toEqual(["Choisir un fichier…", "Choisir un dossier…"]);
  });

  it("changes one application's location only", () => {
    const source = { kind: "eitherLog", locations: { fileLog: "/Logs/a.flog", eitherLog: "/Logs/b.xml" } };
    expect(withLocation(source, eitherApp, "/Logs/c")).toEqual({
      kind: "eitherLog",
      locations: { fileLog: "/Logs/a.flog", eitherLog: "/Logs/c" },
    });
    expect(withLocation(source, fileApp, null), "standard location").toEqual({
      kind: "eitherLog",
      locations: { eitherLog: "/Logs/b.xml" },
    });
    expect(source.locations.fileLog, "unchanged").toBe("/Logs/a.flog");
  });
});
