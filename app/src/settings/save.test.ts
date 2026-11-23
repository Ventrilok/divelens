import { describe, expect, it } from "vitest";
import { applyChange, revertChange } from "./save";
import type { Settings } from "../types";

const base = {
  version: 1,
  source: { kind: "macdive", locations: {} },
  units: { depth: "source", temp: "source" },
  language: "en",
  depthToAltitude: false,
  collections: { parent: [], group: "" },
  logLevel: "normal",
  diveSort: null,
  legacyImported: false,
  checkUpdates: true,
} as unknown as Settings;

describe("saving a setting", () => {
  it("shows the new value, and keeps it when the save works", () => {
    const { next, previous } = applyChange(base, { language: "fr" });
    expect(next.language).toBe("fr");
    expect(previous).toEqual({ language: "en" });
  });

  it("shows the previous value again when the save fails", () => {
    const { next, previous } = applyChange(base, { language: "fr" });
    expect(revertChange(next, { language: "fr" }, previous).language).toBe("en");
  });

  it("does not undo a newer change of the same setting", () => {
    const { next, previous } = applyChange(base, { language: "fr" });
    const newer = { ...next, language: "system" as const };
    expect(revertChange(newer, { language: "fr" }, previous).language).toBe("system");
  });

  it("does not undo a change of another setting", () => {
    const { next, previous } = applyChange(base, { language: "fr" });
    const other = { ...next, logLevel: "detailed" as const };
    const back = revertChange(other, { language: "fr" }, previous);
    expect(back.language).toBe("en");
    expect(back.logLevel).toBe("detailed");
  });

  it("restores an object setting by value", () => {
    const patch = { units: { depth: "ft", temp: "source" } as Settings["units"] };
    const { next, previous } = applyChange(base, patch);
    const copy = { ...next, units: { ...next.units } };
    expect(revertChange(copy, patch, previous).units).toEqual({ depth: "source", temp: "source" });
    const coll = { collections: { parent: ["a"], group: "g" } };
    const c = applyChange(base, coll);
    const same = { ...c.next, collections: { parent: ["a"], group: "g" } };
    expect(revertChange(same, coll, c.previous).collections).toEqual(base.collections);
  });

  it("applies a dive-log change only once it is saved", () => {
    const patch = { source: { ...base.source, kind: "subsurface" } } as Partial<Settings>;
    const { next, previous } = applyChange(base, patch);
    expect(next).toBe(base);
    expect(previous).toEqual({});
    expect(revertChange(next, patch, previous)).toBe(next);
  });
});
