import { describe, expect, it } from "vitest";
import { profileCache } from "./profiles";
import type { LogRef, Sample } from "./types";

const macDive: LogRef = { kind: "macDive", path: "/Dives/MacDive.sqlite" };
const subsurface: LogRef = { kind: "subsurface", path: "/Logs/diver.xml" };

const samples = (depth: number): Sample[] => [
  { time_s: 0, depth_m: 0, temp_c: null },
  { time_s: 600, depth_m: depth, temp_c: null },
];

/** A fake back end: each log gives its own profile for the same dive id. */
function backEnd() {
  const asked: string[] = [];
  const fetch = (log: LogRef, id: string) => {
    asked.push(`${log.kind} ${id}`);
    return Promise.resolve(samples(log.kind === "macDive" ? 18 : 30));
  };
  return { asked, fetch };
}

describe("profile cache", () => {
  it("reads a profile once, then keeps it", async () => {
    const api = backEnd();
    const cache = profileCache(api.fetch);
    expect(cache.get(macDive, "7")).toBeUndefined();
    await cache.load(macDive, "7");
    expect(cache.get(macDive, "7")).toEqual(samples(18));
    expect(api.asked).toEqual(["macDive 7"]);
  });

  it("never gives one log's profile for another log's dive", async () => {
    const cache = profileCache(backEnd().fetch);
    await cache.load(macDive, "7");
    expect(cache.get(subsurface, "7")).toBeUndefined();
    expect(cache.get({ ...macDive, path: "/Elsewhere/MacDive.sqlite" }, "7")).toBeUndefined();
  });

  it("forgets every profile when the dive log is read again", async () => {
    const api = backEnd();
    const cache = profileCache(api.fetch);
    await cache.load(macDive, "7");
    cache.clear();
    expect(cache.get(macDive, "7"), "same log, maybe saved again").toBeUndefined();
    await cache.load(macDive, "7");
    expect(api.asked).toEqual(["macDive 7", "macDive 7"]);
  });

  it("keeps out a profile whose read started before the clear", async () => {
    let finish!: (s: Sample[]) => void;
    const cache = profileCache(() => new Promise<Sample[]>((resolve) => (finish = resolve)));
    const late = cache.load(macDive, "7");
    cache.clear();
    finish(samples(18));
    expect(await late).toEqual(samples(18));
    expect(cache.get(macDive, "7")).toBeUndefined();
  });
});
