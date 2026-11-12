import { describe, expect, it } from "vitest";
import { coveringDive } from "./covers";
import type { Dive } from "./types";

const dive = (id: string, start: string, end: string, durationS = 3000): Dive => ({
  id,
  number: null,
  start,
  end,
  durationS,
  maxDepthM: null,
  tempHighC: null,
  tempLowC: null,
  rating: null,
  site: { id: null, name: null, location: null, country: null, lat: null, lon: null },
  hasProfile: false,
  invalid: false,
});

const dives = [
  dive("697", "2025-12-30T07:25:00", "2025-12-30T08:21:00"),
  dive("698", "2025-12-30T11:19:00", "2025-12-30T12:17:00"),
];

describe("coveringDive", () => {
  it("finds the dive that contains every photo", () => {
    expect(coveringDive(dives, "2025-12-30T07:31:12", "2025-12-30T08:12:40")?.id).toBe("697");
    expect(coveringDive(dives, "2025-12-30T07:25:00", "2025-12-30T08:21:00")?.id).toBe("697");
  });
  it("finds none when photos spill out of the dive or span two dives", () => {
    expect(coveringDive(dives, "2025-12-30T07:20:00", "2025-12-30T08:00:00")).toBeNull();
    expect(coveringDive(dives, "2025-12-30T08:00:00", "2025-12-30T11:30:00")).toBeNull();
    expect(coveringDive(dives, null, null)).toBeNull();
  });
  it("prefers the shortest when dives overlap", () => {
    const long = dive("long", "2025-12-30T07:00:00", "2025-12-30T09:00:00", 7200);
    expect(coveringDive([long, ...dives], "2025-12-30T07:31:00", "2025-12-30T08:00:00")?.id).toBe("697");
  });
});
