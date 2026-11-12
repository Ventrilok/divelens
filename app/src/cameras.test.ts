import { describe, expect, it } from "vitest";
import { clampMargin, clampOffset, formatOffset, initialOffsets } from "./cameras";
import type { Camera } from "./types";

describe("clock offsets", () => {
  it("formats offsets with their sign", () => {
    expect(formatOffset(0)).toBe("0 min");
    expect(formatOffset(45)).toBe("+45 min");
    expect(formatOffset(-30)).toBe("−30 min");
    expect(formatOffset(240)).toBe("+4 h 00");
    expect(formatOffset(241)).toBe("+4 h 01");
    expect(formatOffset(-75)).toBe("−1 h 15");
  });

  it("keeps offsets and margin in range", () => {
    expect(clampOffset(25 * 60)).toBe(24 * 60);
    expect(clampOffset(-25 * 60)).toBe(-24 * 60);
    expect(clampMargin(-5)).toBe(0);
    expect(clampMargin(500)).toBe(120);
  });

  it("starts from each camera's remembered offset", () => {
    const cam = (key: string, remembered: number | null): Camera => ({ key, model: key, serial: null, photos: 1, remembered });
    expect(initialOffsets([cam("a", 240), cam("b", null)])).toEqual({ a: 240, b: 0 });
  });
});
