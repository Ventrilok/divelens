// Runs in a far-off time zone (UTC+14, see vitest.config.ts) to prove dive
// times are never shifted by the computer's time zone.
import { describe, expect, it, vi } from "vitest";

vi.stubGlobal("navigator", { language: "fr-FR" });
const f = await import("./format");

const metric = { depth: "m", temp: "c" } as const;
const imperial = { depth: "ft", temp: "f" } as const;

describe("dates are wall-clock times", () => {
  it("runs in a time zone far from UTC", () => {
    expect(new Date(Date.UTC(2025, 0, 1)).getTimezoneOffset()).toBe(-840);
  });
  it("shows the photos' time span", () => {
    expect(f.formatPhotoSpan("2025-12-30T07:31:12", "2025-12-30T08:12:40")).toBe("30 déc. 2025, 07:31 → 08:12");
    expect(f.formatPhotoSpan("2025-12-31T23:50:00", "2026-01-01T00:10:00")).toBe("31 déc. 2025, 23:50 → 1 janv. 2026, 00:10");
  });
  it("keeps the dive computer's time", () => {
    expect(f.formatRowDate("2025-12-31T07:14:03")).toBe("31 déc. · 07:14");
    expect(f.formatDiveSpan("2025-12-31T07:14:03", "2025-12-31T07:49:13")).toBe("Mer. 31 déc. 2025 · 07:14 → 07:49");
  });
  it("groups by the dive's own month", () => {
    expect(f.monthKey("2025-12-31T23:50:00")).toBe("2025-12");
    expect(f.formatMonth("2025-12-31T23:50:00")).toBe("décembre 2025");
  });
  it("shows the end date when a dive crosses midnight", () => {
    expect(f.formatDiveSpan("2025-12-31T23:40:00", "2026-01-01T00:20:00")).toBe(
      "Mer. 31 déc. 2025 · 23:40 → 1 janv. 2026, 00:20",
    );
  });
});

describe("units", () => {
  it("formats depth", () => {
    expect(f.formatDepth(31.2, metric)).toBe("31,2 m");
    expect(f.formatDepth(18, metric)).toBe("18,0 m");
    expect(f.formatDepth(30, imperial)).toBe("98 ft");
    expect(f.formatDepth(null, metric)).toBe("—");
  });
  it("formats temperatures, including zero and below", () => {
    expect(f.formatTemp(27, metric)).toBe("27 °C");
    expect(f.formatTemp(0, metric)).toBe("0 °C");
    expect(f.formatTemp(-1.5, metric)).toBe("-1,5 °C");
    expect(f.formatTemp(25, imperial)).toBe("77 °F");
    expect(f.formatTempRange(27, 28, metric)).toBe("27 → 28 °C");
    expect(f.formatTempRange(27, 27, metric)).toBe("27 °C");
    expect(f.formatTempRange(null, 28, metric)).toBe("28 °C");
  });
  it("converts filter depths back to metres", () => {
    expect(f.depthToMetres(f.depthValue(20, imperial), imperial)).toBeCloseTo(20);
  });
  it("formats duration and GPS", () => {
    expect(f.formatDuration(2110)).toBe("35 min");
    expect(f.formatDuration(0)).toBe("—");
    expect(f.formatGps(3.5985166, 73.5032833)).toBe("3.5985, 73.5033");
    expect(f.formatGps(null, null)).toBeNull();
  });
});

describe("search text", () => {
  it("ignores accents and case", () => {
    expect(f.fold("Égypte")).toBe("egypte");
  });
});
