import { beforeEach, describe, expect, it } from "vitest";
import { setLanguage } from "../i18n";
import { applyFailure, searchFailure } from "./failure";
import type { ApplyError } from "../types";

const DETAIL = "disk full";

const plain: ApplyError["error"][] = [
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
const writeErrors: ApplyError[] = [
  ...plain.map((error) => ({ error }) as ApplyError),
  { error: "source", detail: { kind: "database", detail: DETAIL } },
  ...reasons.flatMap((reason): ApplyError[] => [
    { error: "lightroom", reason, detail: DETAIL },
    { error: "lightroom", reason, detail: null },
  ]),
];

/** A code is a word of the program (unknownDive, badRequest...); plain words like "busy" are also English. */
function codesOf(e: ApplyError): string[] {
  return (e.error === "lightroom" ? [e.error, e.reason] : [e.error]).filter((c) => /[A-Z]/.test(c));
}

describe.each(["en", "fr"] as const)("Tag Selection failure texts (%s)", (lang) => {
  beforeEach(() => setLanguage(lang));

  it.each(writeErrors.map((e) => [JSON.stringify(e), e] as const))("write: %s", (_, e) => {
    const text = applyFailure(e, "tag");
    expect(text.trim()).not.toBe("");
    expect(text).not.toContain("{");
    expect(text).not.toContain("[object");
    for (const code of codesOf(e)) expect(text).not.toContain(code);
    if (e.error === "lightroom" && e.detail === null && e.reason !== "busy") expect(text).not.toContain(e.reason);
  });

  const readErrors: ApplyError[] = [
    { error: "nothingChosen" },
    { error: "nothingToWrite" },
    { error: "notConnected" },
    { error: "linkLost" },
    { error: "source", detail: { kind: "database", detail: DETAIL } },
    ...reasons.flatMap((reason): ApplyError[] => [
      { error: "lightroom", reason, detail: DETAIL },
      { error: "lightroom", reason, detail: null },
    ]),
  ];
  it.each(readErrors.map((e) => [JSON.stringify(e), e] as const))("read: %s", (_, e) => {
    const text = searchFailure(e, "tag");
    expect(text).not.toBeNull();
    expect(text!.trim()).not.toBe("");
    expect(text).not.toContain("{");
    for (const code of codesOf(e)) expect(text).not.toContain(code);
    if (e.error === "lightroom" && e.detail === null && e.reason !== "busy") expect(text).not.toContain(e.reason);
  });

  const nothingWritten = lang === "en" ? "Nothing was written" : "Rien n'a été écrit";
  const undo = lang === "en" ? "Edit › Undo Dive Lens in Lightroom" : "Édition › Annuler Dive Lens dans Lightroom";

  it("gpsNotWritten and linkLost say what was written, never that nothing was", () => {
    for (const detail of [DETAIL, null]) {
      const text = applyFailure({ error: "lightroom", reason: "gpsNotWritten", detail }, "tag");
      expect(text).not.toContain(nothingWritten);
      expect(text).toContain(undo);
    }
    expect(applyFailure({ error: "linkLost" }, "tag")).not.toContain(nothingWritten);
  });

  it("every other write failure says that nothing was written", () => {
    const others = writeErrors.filter(
      (e) => e.error !== "linkLost" && !(e.error === "lightroom" && e.reason === "gpsNotWritten"),
    );
    for (const e of others) expect(applyFailure(e, "tag"), JSON.stringify(e)).toContain(nothingWritten);
  });

  it("a vanished dive or site says to reopen Dive Lens", () => {
    const sentence = lang === "en" ? "To see the dive log as it is now, reopen Dive Lens." : "Pour voir le carnet de plongée à jour, rouvrez Dive Lens.";
    expect(applyFailure({ error: "unknownDive" }, "tag")).toContain(sentence);
    expect(applyFailure({ error: "unknownSite" }, "tag")).toContain(sentence);
  });

  it("read: a newer description replaced this one", () => {
    expect(searchFailure({ error: "noPhotos" }, "tag")).toBeNull();
    expect(searchFailure({ error: "logChanged" }, "tag")).toBeNull();
  });
});

const collReasons = ["busy", "error", "noSelection", "staleSearch", "badRequest", "somethingNew"];

const collSearchErrors: ApplyError[] = [
  { error: "nothingChosen" },
  { error: "notConnected" },
  { error: "linkLost" },
  { error: "noPhotos" },
  { error: "nothingToWrite" },
  { error: "logChanged" },
  { error: "source", detail: { kind: "database", detail: DETAIL } },
  ...collReasons.flatMap((reason): ApplyError[] => [
    { error: "lightroom", reason, detail: DETAIL },
    { error: "lightroom", reason, detail: null },
  ]),
];

/** The search writes nothing: these words belong to the write failures only. */
const writeWords = [/written/i, /écrit/i, /undo/i, /annuler/i, /apply again/i, /réappliquez/i, /appliquez/i];

describe.each(["en", "fr"] as const)("Create Collections search failure texts (%s)", (lang) => {
  beforeEach(() => setLanguage(lang));

  it("dives that vanished say to reopen Dive Lens", () => {
    const sentence = lang === "en" ? "To see the dive log as it is now, reopen Dive Lens." : "Pour voir le carnet de plongée à jour, rouvrez Dive Lens.";
    expect(searchFailure({ error: "nothingChosen" }, "collections")).toContain(sentence);
  });

  it.each(collSearchErrors.map((e) => [JSON.stringify(e), e] as const))("%s", (_, e) => {
    const text = searchFailure(e, "collections");
    expect(text.trim()).not.toBe("");
    expect(text).not.toContain("{");
    for (const word of writeWords) expect(text).not.toMatch(word);
    const codes = (e.error === "lightroom" ? [e.error, e.reason] : [e.error]).filter((c) => /[A-Z]/.test(c));
    for (const code of codes) expect(text).not.toContain(code);
    if (e.error === "lightroom" && e.detail === null && e.reason !== "busy") expect(text).not.toContain(e.reason);
  });
});

/** The plug-in's internal details: never shown. */
const INTERNAL = ["bad group", "bad dive", "bad parent"];

const collWriteErrors: ApplyError[] = [
  { error: "nothingChosen" },
  { error: "notConnected" },
  { error: "linkLost" },
  { error: "noPhotos" },
  { error: "nothingToWrite" },
  { error: "logChanged" },
  { error: "source", detail: { kind: "database", detail: DETAIL } },
  ...["busy", "error", "badRequest", "gpsNotWritten", "staleSearch", "parentNotFound", "somethingNew"].flatMap(
    (reason): ApplyError[] => [
      { error: "lightroom", reason, detail: DETAIL },
      { error: "lightroom", reason, detail: null },
      ...INTERNAL.map((detail): ApplyError => ({ error: "lightroom", reason, detail })),
    ],
  ),
];

describe.each(["en", "fr"] as const)("Create Collections write failure texts (%s)", (lang) => {
  beforeEach(() => setLanguage(lang));
  const nothingWritten = lang === "en" ? "Nothing was written" : "Rien n'a été écrit";
  const undo = lang === "en" ? "Edit › Undo Dive Lens in Lightroom" : "Édition › Annuler Dive Lens dans Lightroom";

  it.each(collWriteErrors.map((e) => [JSON.stringify(e), e] as const))("%s", (_, e) => {
    const text = applyFailure(e, "collections");
    expect(text.trim()).not.toBe("");
    expect(text).not.toContain("{");
    const codes = (e.error === "lightroom" ? [e.error, e.reason] : [e.error]).filter((c) => /[A-Z]/.test(c));
    for (const code of codes) expect(text).not.toContain(code);
    if (e.error === "lightroom" && e.detail === null && e.reason !== "busy") expect(text).not.toContain(e.reason);
    // A request the plug-in could not use: its detail is internal.
    if (e.error === "lightroom" && e.reason === "badRequest" && e.detail) expect(text).not.toContain(e.detail);
  });

  it("gpsNotWritten and linkLost say what was written, never that nothing was", () => {
    for (const detail of [DETAIL, null]) {
      const text = applyFailure({ error: "lightroom", reason: "gpsNotWritten", detail }, "collections");
      expect(text).not.toContain(nothingWritten);
      expect(text).toContain(undo);
    }
    expect(applyFailure({ error: "linkLost" }, "collections")).not.toContain(nothingWritten);
  });
});
