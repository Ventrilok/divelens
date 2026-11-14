import { beforeEach, describe, expect, it } from "vitest";
import { importErrorText } from "./backup";
import { changeErrorText, isLogChanged, latestOnly, logRef, sourceWatch } from "./diveLog";
import { setLanguage } from "./i18n";
import type { Settings } from "./types";

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
  const source = (kind: "macDive" | "subsurface", macDivePath: string | null, subsurfacePath: string | null) =>
    ({ kind, macDivePath, subsurfacePath }) as Settings["source"];

  it("names the log shown by application and location only", () => {
    expect(logRef({ kind: "subsurface", path: "/Logs/diver.xml", isDefaultPath: false })).toEqual({
      kind: "subsurface",
      path: "/Logs/diver.xml",
    });
  });

  it("starts the flows again when the application or a location changes", () => {
    const changed = sourceWatch(source("macDive", null, null));
    expect(changed(source("macDive", null, null)), "same log read again").toBe(false);
    expect(changed(source("subsurface", null, null)), "other application").toBe(true);
    expect(changed(source("subsurface", null, "/Logs/diver.xml")), "file chosen").toBe(true);
    expect(changed(source("subsurface", null, "/Logs/diver.xml")), "Retry").toBe(false);
    expect(changed(source("subsurface", null, null)), "standard location").toBe(true);
    expect(changed(source("subsurface", "/Dives/MacDive.sqlite", null)), "import").toBe(true);
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
});
