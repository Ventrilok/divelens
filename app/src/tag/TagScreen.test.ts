import { beforeEach, describe, expect, it } from "vitest";
import { setLanguage } from "../i18n";
import { describeFailure, failure } from "./TagScreen";
import type { CollectionsError, TagError } from "../types";

const DETAIL = "disk full";

const plain: TagError["error"][] = [
  "nothingChosen",
  "notConnected",
  "linkLost",
  "badGps",
  "unknownDive",
  "unknownSite",
  "logChanged",
  "noPhotos",
];
const reasons = ["noSelection", "busy", "gpsNotWritten", "selectionChanged", "staleSelection", "badRequest", "error", "somethingNew"];

/** Every write failure, the plug-in's reasons with and without a detail. */
const writeErrors: TagError[] = [
  ...plain.map((error) => ({ error }) as TagError),
  { error: "source", detail: { kind: "database", detail: DETAIL } },
  ...reasons.flatMap((reason): TagError[] => [
    { error: "lightroom", reason, detail: DETAIL },
    { error: "lightroom", reason, detail: null },
  ]),
];

/** A code is a word of the program (unknownDive, badRequest...); plain words like "busy" are also English. */
function codesOf(e: TagError | CollectionsError): string[] {
  return (e.error === "lightroom" ? [e.error, e.reason] : [e.error]).filter((c) => /[A-Z]/.test(c));
}

describe.each(["en", "fr"] as const)("Tag Selection failure texts (%s)", (lang) => {
  beforeEach(() => setLanguage(lang));

  it.each(writeErrors.map((e) => [JSON.stringify(e), e] as const))("write: %s", (_, e) => {
    const text = failure(e);
    expect(text.trim()).not.toBe("");
    expect(text).not.toContain("{");
    expect(text).not.toContain("[object");
    for (const code of codesOf(e)) expect(text).not.toContain(code);
    if (e.error === "lightroom" && e.detail === null && e.reason !== "busy") expect(text).not.toContain(e.reason);
  });

  const readErrors: CollectionsError[] = [
    { error: "nothingChosen" },
    { error: "nothingToWrite" },
    { error: "notConnected" },
    { error: "linkLost" },
    { error: "source", detail: { kind: "database", detail: DETAIL } },
    ...reasons.flatMap((reason): CollectionsError[] => [
      { error: "lightroom", reason, detail: DETAIL },
      { error: "lightroom", reason, detail: null },
    ]),
  ];
  it.each(readErrors.map((e) => [JSON.stringify(e), e] as const))("read: %s", (_, e) => {
    const text = describeFailure(e);
    expect(text).not.toBeNull();
    expect(text!.trim()).not.toBe("");
    expect(text).not.toContain("{");
    for (const code of codesOf(e)) expect(text).not.toContain(code);
    if (e.error === "lightroom" && e.detail === null && e.reason !== "busy") expect(text).not.toContain(e.reason);
  });

  it("read: a newer description replaced this one", () => {
    expect(describeFailure({ error: "noPhotos" })).toBeNull();
    expect(describeFailure({ error: "logChanged" })).toBeNull();
  });
});
