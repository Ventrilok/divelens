// What the journey's rules share. Taken over with the rules of Tag
// Selection, unchanged; what blocks Apply with it is in review.test.ts.
import { describe, expect, it } from "vitest";
import { gpsRefused } from "./shared";

describe("gpsRefused", () => {
  const base = { withPlace: true, placeKind: "diveSite" as const, diveSiteUsable: false, siteId: "s1", gpsValid: false };
  it("flags an invalid position as soon as another site is written, whatever the place kind", () => {
    expect(gpsRefused(base)).toBe(true);
    expect(gpsRefused({ ...base, placeKind: "site" })).toBe(true);
    expect(gpsRefused({ ...base, gpsValid: true })).toBe(false);
  });
  it("does not flag when the dive's site is used, no site is picked or no place is written", () => {
    expect(gpsRefused({ ...base, diveSiteUsable: true })).toBe(false);
    expect(gpsRefused({ ...base, siteId: null })).toBe(false);
    expect(gpsRefused({ ...base, withPlace: false })).toBe(false);
  });
});
