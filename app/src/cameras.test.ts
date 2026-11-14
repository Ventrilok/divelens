import { describe, expect, it } from "vitest";
import { clampMargin, clampOffset, initialOffsets, keepOffsets, offsetField, parseOffset } from "./cameras";
import type { Camera } from "./types";

describe("clock offsets", () => {
  it("keeps offsets and margin in range", () => {
    expect(clampOffset(25 * 60)).toBe(24 * 60);
    expect(clampOffset(-25 * 60)).toBe(-24 * 60);
    expect(clampMargin(-5)).toBe(0);
    expect(clampMargin(500)).toBe(120);
  });

  it("writes and reads the offset field", () => {
    expect(offsetField(241)).toBe("+4:01");
    expect(offsetField(0)).toBe("0:00");
    expect(offsetField(-30)).toBe("\u22120:30");
    expect(parseOffset("+4:01")).toBe(241);
    expect(parseOffset("4:1")).toBe(241);
    expect(parseOffset("\u22120:30")).toBe(-30);
    expect(parseOffset("-1:15")).toBe(-75);
    expect(parseOffset("4h01")).toBe(241);
    expect(parseOffset("4 h")).toBe(240);
    expect(parseOffset("+4")).toBe(240);
    expect(parseOffset("0:00")).toBe(0);
    expect(parseOffset("24:00")).toBe(1440);
    expect(parseOffset("24:01")).toBe(null);
    expect(parseOffset("4:75")).toBe(null);
    expect(parseOffset("abc")).toBe(null);
    expect(parseOffset("")).toBe(null);
  });

  it("starts from each camera's remembered offset", () => {
    const cam = (key: string, remembered: number | null): Camera => ({ key, model: key, serial: null, photos: 1, remembered });
    expect(initialOffsets([cam("a", 240), cam("b", null)])).toEqual({ a: 240, b: 0 });
  });

  it("keeps the offsets shown when the selection's cameras change", () => {
    const cam = (key: string, remembered: number | null): Camera => ({ key, model: key, serial: null, photos: 1, remembered });
    // The TG-7 set by hand stays; the GoPro arrives with its remembered
    // offset, the phone with none; the camera no longer selected goes.
    expect(keepOffsets({ tg7: 245, gone: 60 }, [cam("tg7", 240), cam("gopro", -30), cam("phone", null)])).toEqual({
      tg7: 245,
      gopro: -30,
      phone: 0,
    });
    // A shown offset of 0 is kept too, not replaced by the remembered one.
    expect(keepOffsets({ tg7: 0 }, [cam("tg7", 240)])).toEqual({ tg7: 0 });
  });
});
