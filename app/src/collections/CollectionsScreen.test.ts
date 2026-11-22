import { beforeEach, describe, expect, it } from "vitest";
import { setLanguage } from "../i18n";
import { searchFailure } from "./CollectionsScreen";
import type { CollectionsError } from "../types";

const DETAIL = "disk full";
const reasons = ["busy", "error", "noSelection", "staleSearch", "badRequest", "somethingNew"];

const errors: CollectionsError[] = [
  { error: "nothingChosen" },
  { error: "notConnected" },
  { error: "linkLost" },
  { error: "noPhotos" },
  { error: "nothingToWrite" },
  { error: "logChanged" },
  { error: "source", detail: { kind: "database", detail: DETAIL } },
  ...reasons.flatMap((reason): CollectionsError[] => [
    { error: "lightroom", reason, detail: DETAIL },
    { error: "lightroom", reason, detail: null },
  ]),
];

/** The search writes nothing: these words belong to the write failures only. */
const writeWords = [/written/i, /écrit/i, /undo/i, /annuler/i, /apply again/i, /réappliquez/i, /appliquez/i];

describe.each(["en", "fr"] as const)("Create Collections search failure texts (%s)", (lang) => {
  beforeEach(() => setLanguage(lang));

  it.each(errors.map((e) => [JSON.stringify(e), e] as const))("%s", (_, e) => {
    const text = searchFailure(e);
    expect(text.trim()).not.toBe("");
    expect(text).not.toContain("{");
    for (const word of writeWords) expect(text).not.toMatch(word);
    const codes = (e.error === "lightroom" ? [e.error, e.reason] : [e.error]).filter((c) => /[A-Z]/.test(c));
    for (const code of codes) expect(text).not.toContain(code);
    if (e.error === "lightroom" && e.detail === null && e.reason !== "busy") expect(text).not.toContain(e.reason);
  });
});
