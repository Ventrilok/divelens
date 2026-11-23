import { describe, expect, it } from "vitest";
import { revertChange } from "./save";
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

const shown = (s: Settings, patch: Partial<Settings>): Settings => ({ ...s, ...patch });

describe("a failed save", () => {
  it("shows the saved value again", () => {
    const patch = { language: "fr" as const };
    expect(revertChange(shown(base, patch), patch, base).language).toBe("en");
  });

  it("does not undo a newer change of the same setting", () => {
    const patch = { language: "fr" as const };
    const newer = shown(base, { language: "system" });
    expect(revertChange(newer, patch, base).language).toBe("system");
  });

  it("does not touch a setting the patch did not set", () => {
    const patch = { language: "fr" as const };
    const other = shown(shown(base, patch), { logLevel: "detailed" });
    const back = revertChange(other, patch, base);
    expect(back.language).toBe("en");
    expect(back.logLevel).toBe("detailed");
  });

  it("compares object settings by value", () => {
    const patch = { units: { depth: "ft", temp: "source" } as Settings["units"] };
    const copy = shown(base, { units: { depth: "ft", temp: "source" } });
    expect(revertChange(copy, patch, base).units).toEqual({ depth: "source", temp: "source" });
    const coll = { collections: { parent: ["a"], group: "g" } };
    const same = shown(base, { collections: { parent: ["a"], group: "g" } });
    expect(revertChange(same, coll, base).collections).toEqual(base.collections);
  });

  it("ends on the saved value when two changes of one setting both fail", () => {
    const a = { language: "fr" as const };
    const b = { language: "system" as const };
    const screen = shown(shown(base, a), b);
    const s1 = revertChange(screen, a, base);
    expect(s1.language).toBe("system");
    expect(revertChange(s1, b, base).language).toBe("en");
  });

  it("ends on the saved units when depth then temperature both fail", () => {
    const depth = { units: { depth: "ft", temp: "source" } as Settings["units"] };
    const temp = { units: { depth: "ft", temp: "f" } as Settings["units"] };
    const screen = shown(shown(base, depth), temp);
    const s1 = revertChange(screen, depth, base);
    expect(s1.units).toEqual({ depth: "ft", temp: "f" });
    expect(revertChange(s1, temp, base).units).toEqual({ depth: "source", temp: "source" });
  });

  it("restores what an unrelated save wrote, not an old capture", () => {
    const patch = { logLevel: "detailed" as const };
    const saved = shown(base, { language: "fr" });
    const back = revertChange(shown(saved, patch), patch, saved);
    expect(back.language).toBe("fr");
    expect(back.logLevel).toBe("normal");
  });

  it("leaves a dive log change alone: it is not shown before it is saved", () => {
    const patch = { source: { ...base.source, kind: "subsurface" } } as Partial<Settings>;
    expect(revertChange(base, patch, base)).toBe(base);
  });
});
