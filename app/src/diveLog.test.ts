import { beforeEach, describe, expect, it } from "vitest";
import { importErrorText } from "./backup";
import {
  changeErrorText,
  isLogChanged,
  latestOnly,
  logChangedNotice,
  logRef,
  logWatch,
  nextSearchNumber,
  sameLogSetting,
} from "./diveLog";
import { setLanguage, t } from "./i18n";
import type { Settings, SourceInfo, SourceKind } from "./types";

beforeEach(() => setLanguage("en"));

/** A promise settled from the test, to replay reads finishing in any order. */
function deferred<T>() {
  let resolve!: (v: T) => void;
  let reject!: (e: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

const flush = () => new Promise((r) => setTimeout(r, 0));

describe("reads of the dive log", () => {
  it("ignores a slow read of the previous log that finishes after the newer one", async () => {
    const run = latestOnly();
    const shown: string[] = [];
    const older = deferred<string>();
    const newer = deferred<string>();
    run(older.promise, (log) => shown.push(log), () => shown.push("error"));
    run(newer.promise, (log) => shown.push(log), () => shown.push("error"));
    newer.resolve("Subsurface log");
    await flush();
    older.resolve("MacDive log");
    await flush();
    expect(shown).toEqual(["Subsurface log"]);
  });

  it("ignores the failure of an older read too", async () => {
    const run = latestOnly();
    const shown: string[] = [];
    const older = deferred<string>();
    const newer = deferred<string>();
    run(older.promise, (log) => shown.push(log), () => shown.push("older failed"));
    run(newer.promise, (log) => shown.push(log), (e) => shown.push(`newer failed: ${String(e)}`));
    older.reject("gone");
    newer.reject("not found");
    await flush();
    expect(shown).toEqual(["newer failed: not found"]);
  });

  it("hears of every read when they finish in order", async () => {
    const run = latestOnly();
    const shown: string[] = [];
    run(Promise.resolve("first"), (log) => shown.push(log), () => {});
    await flush();
    run(Promise.resolve("second"), (log) => shown.push(log), () => {});
    await flush();
    expect(shown).toEqual(["first", "second"]);
  });
});

describe("one dive log at a time", () => {
  /** A log as the back end reports it after a read: application in use, resolved location. */
  const read = (kind: SourceKind, path: string, isDefaultPath = false): SourceInfo => ({ kind, path, isDefaultPath });
  const standardMacDive = "/Users/diver/Library/Application Support/MacDive/MacDive.sqlite";

  it("names the log shown by application and location only", () => {
    expect(logRef(read("subsurface", "/Logs/diver.xml"))).toEqual({
      kind: "subsurface",
      path: "/Logs/diver.xml",
    });
  });

  it("starts the flows again when a read shows another log", () => {
    const otherLog = logWatch();
    expect(otherLog(read("macDive", standardMacDive, true)), "first read").toBe(false);
    expect(otherLog(read("macDive", standardMacDive, true)), "Retry").toBe(false);
    expect(otherLog(read("subsurface", "/Users/diver/diver.xml", true)), "other application").toBe(true);
    expect(otherLog(read("subsurface", "/Logs/diver.xml")), "file chosen").toBe(true);
    expect(otherLog(read("subsurface", "/Logs/diver.xml")), "MacDive location changed").toBe(false);
    expect(otherLog(read("macDive", "/Logs/diver.xml")), "same location, other application").toBe(true);
  });

  it("does not start again for the standard location chosen on the standard file", () => {
    const otherLog = logWatch();
    otherLog(read("macDive", standardMacDive, false));
    expect(otherLog(read("macDive", standardMacDive, true))).toBe(false);
  });

  it("recognizes the back end's refusal of a log that changed", () => {
    expect(isLogChanged({ error: "logChanged" })).toBe(true);
    expect(isLogChanged({ error: "notConnected" })).toBe(false);
    expect(isLogChanged({ kind: "logChanged" })).toBe(false);
    expect(isLogChanged("logChanged")).toBe(false);
    expect(isLogChanged(null)).toBe(false);
  });

  it("says the dive log cannot change during a write", () => {
    const text = "Writing to Lightroom: the dive log can be changed once the write is done.";
    expect(changeErrorText({ kind: "writeInProgress" })).toBe(text);
    expect(importErrorText({ kind: "writeInProgress" })).toBe(text);
    expect(changeErrorText({ kind: "notSaved", detail: "disk full" })).toBe("Setting not saved: disk full");
    setLanguage("fr");
    expect(changeErrorText({ kind: "writeInProgress" })).toBe(
      "Écriture dans Lightroom en cours : le carnet de plongée pourra être changé une fois celle-ci terminée.",
    );
  });

  it("reads nothing again when only the other application's location changed", () => {
    const source = (kind: SourceKind, macDivePath: string | null, subsurfacePath: string | null): Settings["source"] => ({
      kind,
      locations: {
        ...(macDivePath === null ? {} : { macDive: macDivePath }),
        ...(subsurfacePath === null ? {} : { subsurface: subsurfacePath }),
      },
    });
    const before = source("subsurface", null, "/Logs/diver.xml");
    expect(sameLogSetting(before, source("subsurface", "/Old/MacDive.sqlite", "/Logs/diver.xml"), "subsurface")).toBe(true);
    expect(sameLogSetting(before, source("subsurface", null, "/Logs/other.xml"), "subsurface")).toBe(false);
    expect(sameLogSetting(before, source("subsurface", null, null), "subsurface"), "standard location").toBe(false);
    expect(sameLogSetting(before, source("macDive", null, "/Logs/diver.xml"), "subsurface"), "application").toBe(false);
    // Windows, settings copied from a Mac: Subsurface in use whatever the file says.
    const copied = source("macDive", "/Dives/MacDive.sqlite", "/Logs/diver.xml");
    expect(sameLogSetting(copied, source("macDive", null, "/Logs/diver.xml"), "subsurface")).toBe(true);
    expect(sameLogSetting(copied, source("macDive", "/Dives/MacDive.sqlite", null), "subsurface")).toBe(false);
  });

  it("says why the flow started again, an Apply first", () => {
    expect(t(logChangedNotice(null, false))).toBe("The dive log has changed: start again.");
    expect(t(logChangedNotice(null, true))).toBe("The dive log has changed. Nothing was written: start again.");
    expect(logChangedNotice("log.changedNothingWritten", false)).toBe("log.changedNothingWritten");
    setLanguage("fr");
    expect(t(logChangedNotice(null, true))).toBe("Le carnet de plongée a changé. Rien n'a été écrit : recommencez.");
    expect(t(logChangedNotice(null, false))).toBe("Le carnet de plongée a changé : recommencez.");
  });

  it("numbers photo searches in the order they are asked for, above any earlier window", () => {
    const first = nextSearchNumber();
    const second = nextSearchNumber();
    expect(second).toBeGreaterThan(first);
    expect(first).toBeGreaterThanOrEqual(Date.now() - 60_000);
  });
});
