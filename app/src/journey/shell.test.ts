// The journey's shell: what it hands the two photo readings and the step
// strip, at each step of each path. Fictional dives only.
import { describe, expect, it } from "vitest";
import { appliedFor, looksUpSetAgain, readingDive, screenClass, searchedDives, selectionReads, stripLeadsBack } from "./shell";
import { reduce, start, type Action, type Journey } from "./state";
import type { Applied } from "./rules";
import type { Dive } from "../types";

const run = (from: Journey, ...actions: Action[]) => actions.reduce(reduce, from);

const dive = (id: string): Dive => ({
  id,
  number: 12,
  start: "2026-03-06T09:00:00",
  end: "2026-03-06T09:50:00",
  durationS: 3000,
  maxDepthM: 18,
  tempHighC: null,
  tempLowC: null,
  rating: null,
  site: { id: "s1", name: "Blue Arch", location: null, country: null, lat: null, lon: null },
  hasProfile: true,
  invalid: false,
  notes: null,
});
const dives = [dive("d1"), dive("d2")];

const catalogAt2 = run(start, { type: "choosePath", path: "catalog" }, { type: "next" }, { type: "setDives", ids: ["d1", "d2"] });
const catalogAt3 = reduce(catalogAt2, { type: "next" });
const catalogAt4 = reduce(catalogAt3, { type: "next" });
const catalogAt5 = reduce(catalogAt4, { type: "applied" });
const selectionAt2 = run(start, { type: "choosePath", path: "selection" }, { type: "next" }, { type: "pickDive", pick: { kind: "dive", id: "d1" } });
const selectionAt4 = run(selectionAt2, { type: "next" }, { type: "next" });
const selectionAt5 = reduce(selectionAt4, { type: "applied" });

describe("searchedDives: the dives the catalog's photo search looks for", () => {
  it("none before step 3, so that ticking dives starts no search", () => {
    expect(searchedDives(start)).toEqual([]);
    expect(searchedDives(catalogAt2)).toEqual([]);
  });

  it("the dives ticked at steps 3 and 4, so that Apply never takes counts of an idle search", () => {
    expect(searchedDives(catalogAt3)).toEqual(["d1", "d2"]);
    expect(searchedDives(catalogAt4)).toEqual(["d1", "d2"]);
  });

  it("none on the summary: \"handle separately\" then shows no card of the previous run", () => {
    expect(searchedDives(catalogAt5)).toEqual([]);
    const separately = reduce(catalogAt5, { type: "handleSeparately", ids: ["d2"] });
    expect(separately.step).toBe(3);
    expect(searchedDives(separately)).toEqual(["d2"]);
  });

  it("none on the selection path", () => {
    expect(searchedDives(selectionAt4)).toEqual([]);
  });
});

describe("selectionReads: the selection is read photo by photo", () => {
  it("on the selection path, from step 2 on (choice no. 3)", () => {
    expect(selectionReads(reduce(start, { type: "choosePath", path: "selection" }))).toBe(false);
    expect(selectionReads(selectionAt2)).toBe(true);
    expect(selectionReads(selectionAt4)).toBe(true);
    expect(selectionReads(selectionAt5)).toBe(true);
  });

  it("never on the catalog path, nor before a path is chosen", () => {
    expect(selectionReads(start)).toBe(false);
    expect(selectionReads(catalogAt3)).toBe(false);
  });
});

describe("readingDive: the dive the selection is read for", () => {
  it("the dive chosen, whatever the options say", () => {
    expect(readingDive(selectionAt4, dives)?.id).toBe("d1");
    expect(readingDive(reduce(selectionAt4, { type: "setOptions", options: { dive: false } }), dives)?.id).toBe("d1");
  });

  it("none for no dive, nothing chosen, a dive gone from the log, or the catalog path", () => {
    expect(readingDive(reduce(selectionAt2, { type: "pickDive", pick: { kind: "noDive" } }), dives)).toBeNull();
    expect(readingDive(run(start, { type: "choosePath", path: "selection" }, { type: "next" }), dives)).toBeNull();
    expect(readingDive(selectionAt4, [dive("d2")])).toBeNull();
    expect(readingDive(catalogAt4, dives)).toBeNull();
  });
});

describe("stripLeadsBack: the steps done can be clicked", () => {
  it("before the summary, while nothing is being written", () => {
    expect(stripLeadsBack(catalogAt4, false)).toBe(true);
    expect(stripLeadsBack(catalogAt4, true)).toBe(false);
    expect(stripLeadsBack(catalogAt5, false)).toBe(false);
  });
});

describe("appliedFor: what the summary shows", () => {
  const applied: Applied = { path: "selection", result: { count: 3, withDepth: 2, collection: null }, outside: [], outsideMore: 0, perPhoto: true };

  it("the result of this run's Apply", () => {
    expect(appliedFor({ run: selectionAt5.run, applied }, selectionAt5)).toBe(applied);
  });

  it("nothing once a run starts again, nor before anything was written", () => {
    const again = reduce(selectionAt5, { type: "done" });
    expect(appliedFor({ run: selectionAt5.run, applied }, again)).toBeNull();
    expect(appliedFor(null, selectionAt5)).toBeNull();
  });
});

describe("looksUpSetAgain: the collection set is looked up again before each review", () => {
  it("on each arrival at step 3 from step 2, on either path", () => {
    expect(looksUpSetAgain(catalogAt2.step, catalogAt3.step)).toBe(true);
    const selectionAt3 = reduce(selectionAt2, { type: "next" });
    expect(looksUpSetAgain(selectionAt2.step, selectionAt3.step)).toBe(true);
    // Back at step 2 and forward again: once more.
    const back = reduce(catalogAt3, { type: "back" });
    expect(looksUpSetAgain(back.step, reduce(back, { type: "next" }).step)).toBe(true);
  });

  it("not on the other moves between steps", () => {
    expect(looksUpSetAgain(1, 2)).toBe(false);
    expect(looksUpSetAgain(3, 4)).toBe(false);
    expect(looksUpSetAgain(4, 3)).toBe(false);
    expect(looksUpSetAgain(3, 2)).toBe(false);
    // "No dive" skips the clocks: no collection is created, the set does not matter.
    expect(looksUpSetAgain(2, 4)).toBe(false);
    // From the summary ("handle separately"): the lookup starts again anyway, from outside steps 2 to 4.
    expect(looksUpSetAgain(5, 3)).toBe(false);
  });
});

describe("screenClass", () => {
  it("the dive list fills the screen at step 2; the other steps scroll", () => {
    expect(screenClass(catalogAt2)).toBe("screen");
    expect(screenClass(start)).toBe("screen screen-scroll");
    expect(screenClass(catalogAt4)).toBe("screen screen-scroll");
  });
});
