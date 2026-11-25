import { describe, expect, it } from "vitest";
import { NO_PICK } from "./SitePicker";
import { defaultOptions, reduce, start, stepStates, type Action, type Journey } from "./state";

/** Runs the actions one after the other from `from`. */
const run = (from: Journey, ...actions: Action[]) => actions.reduce(reduce, from);

const catalog = run(start, { type: "choosePath", path: "catalog" });
const selection = run(start, { type: "choosePath", path: "selection" });
/** Catalog path at step 3, dives d1 and d2 ticked. */
const catalogAt3 = run(catalog, { type: "next" }, { type: "setDives", ids: ["d1", "d2"] }, { type: "next" });
/** Selection path at step 3, dive d1 chosen. */
const selectionAt3 = run(selection, { type: "next" }, { type: "pickDive", pick: { kind: "dive", id: "d1" } }, { type: "next" });
/** Selection path at step 4 with "no dive — location only". */
const noDiveAt4 = run(selection, { type: "next" }, { type: "pickDive", pick: { kind: "noDive" } }, { type: "next" });
const changed = { place: false, altitude: true } as const;

describe("start", () => {
  it("opens on step 1 with nothing chosen and the default options", () => {
    expect(start).toEqual({ step: 1, path: null, dives: new Set(), dive: { kind: "none" }, options: defaultOptions, run: 0 });
  });

  it("ticks every option but the altitude, on each dive's site", () => {
    expect(defaultOptions).toEqual({
      dive: true,
      altitude: false,
      place: true,
      gps: true,
      collections: true,
      site: "diveSite",
      pick: NO_PICK,
    });
  });
});

describe("choosePath", () => {
  it("starts the next steps again from scratch for another path", () => {
    const before = run(catalogAt3, { type: "setOptions", options: changed }, { type: "goTo", step: 1 });
    const after = reduce(before, { type: "choosePath", path: "selection" });
    expect(after).toEqual({ ...before, path: "selection", dives: new Set(), dive: { kind: "none" }, options: defaultOptions, run: before.run + 1 });
  });

  it("changes nothing for the same path", () => {
    const before = run(catalog, { type: "setDives", ids: ["d1"] }, { type: "setOptions", options: changed });
    expect(reduce(before, { type: "choosePath", path: "catalog" })).toBe(before);
  });
});

describe("next", () => {
  it("goes from step 1 to step 2 once a path is chosen", () => {
    expect(reduce(start, { type: "next" })).toBe(start);
    expect(reduce(catalog, { type: "next" }).step).toBe(2);
    expect(reduce(selection, { type: "next" }).step).toBe(2);
  });

  it("goes from step 2 to step 3 once dives are ticked or a dive is chosen", () => {
    const catalogAt2 = reduce(catalog, { type: "next" });
    const selectionAt2 = reduce(selection, { type: "next" });
    expect(reduce(catalogAt2, { type: "next" })).toBe(catalogAt2);
    expect(reduce(selectionAt2, { type: "next" })).toBe(selectionAt2);
    expect(catalogAt3.step).toBe(3);
    expect(selectionAt3.step).toBe(3);
  });

  it("skips the clocks for no dive: step 2 leads to step 4", () => {
    expect(noDiveAt4.step).toBe(4);
  });

  it("goes from step 3 to step 4, and no further: Apply leads to the summary", () => {
    const at4 = reduce(catalogAt3, { type: "next" });
    expect(at4.step).toBe(4);
    expect(reduce(at4, { type: "next" })).toBe(at4);
    const at5 = reduce(at4, { type: "applied" });
    expect(reduce(at5, { type: "next" })).toBe(at5);
  });

  it("keeps the run going", () => {
    expect(reduce(catalogAt3, { type: "next" }).run).toBe(catalogAt3.run);
  });
});

describe("back", () => {
  it("goes back one step, from step 4 to step 2 for no dive", () => {
    const at4 = reduce(selectionAt3, { type: "next" });
    expect(reduce(at4, { type: "back" }).step).toBe(3);
    expect(reduce(selectionAt3, { type: "back" }).step).toBe(2);
    expect(reduce(reduce(selection, { type: "next" }), { type: "back" }).step).toBe(1);
    expect(reduce(noDiveAt4, { type: "back" }).step).toBe(2);
  });

  it("has no back at step 1 nor from the summary", () => {
    expect(reduce(catalog, { type: "back" })).toBe(catalog);
    const at5 = run(catalogAt3, { type: "next" }, { type: "applied" });
    expect(reduce(at5, { type: "back" })).toBe(at5);
  });
});

describe("goTo", () => {
  const at4 = reduce(catalogAt3, { type: "next" });

  it("returns to a step already done", () => {
    expect(reduce(at4, { type: "goTo", step: 1 }).step).toBe(1);
    expect(reduce(at4, { type: "goTo", step: 2 }).step).toBe(2);
    expect(reduce(at4, { type: "goTo", step: 3 }).step).toBe(3);
  });

  it("never goes to the current step or a later one", () => {
    expect(reduce(catalogAt3, { type: "goTo", step: 3 })).toBe(catalogAt3);
    expect(reduce(catalogAt3, { type: "goTo", step: 4 })).toBe(catalogAt3);
    expect(reduce(catalogAt3, { type: "goTo", step: 5 })).toBe(catalogAt3);
  });

  it("never leaves the summary", () => {
    const at5 = reduce(at4, { type: "applied" });
    expect(reduce(at5, { type: "goTo", step: 2 })).toBe(at5);
  });

  it("never goes to the clocks for no dive", () => {
    expect(reduce(noDiveAt4, { type: "goTo", step: 3 })).toBe(noDiveAt4);
    expect(reduce(noDiveAt4, { type: "goTo", step: 2 }).step).toBe(2);
  });
});

describe("toggleDive and setDives", () => {
  it("ticks and unticks one dive, in a new set", () => {
    const at2 = reduce(catalog, { type: "next" });
    const one = reduce(at2, { type: "toggleDive", id: "d1" });
    expect([...one.dives]).toEqual(["d1"]);
    expect(at2.dives.size).toBe(0);
    expect(reduce(one, { type: "toggleDive", id: "d1" }).dives.size).toBe(0);
  });

  it("replaces the dives ticked", () => {
    expect([...reduce(catalogAt3, { type: "setDives", ids: ["d3"] }).dives]).toEqual(["d3"]);
    expect([...catalogAt3.dives]).toEqual(["d1", "d2"]);
  });
});

describe("pickDive", () => {
  const at2 = reduce(selection, { type: "next" });

  it("puts the site on another site for no dive", () => {
    const j = reduce(at2, { type: "pickDive", pick: { kind: "noDive" } });
    expect(j.dive).toEqual({ kind: "noDive" });
    expect(j.options.site).toBe("other");
  });

  it("puts the site back on the dive's site for a dive", () => {
    const other = run(at2, { type: "pickDive", pick: { kind: "noDive" } }, { type: "setOptions", options: changed });
    const j = reduce(other, { type: "pickDive", pick: { kind: "dive", id: "d2" } });
    expect(j.dive).toEqual({ kind: "dive", id: "d2" });
    expect(j.options).toEqual({ ...other.options, site: "diveSite" });
  });

  it("keeps the site chosen when another dive is chosen, as before", () => {
    const pick = { country: "Nowhere Islands", location: "North Bay", siteId: "s1", lat: "-8.5", lon: "115.25" };
    const other = reduce(selectionAt3, { type: "setOptions", options: { site: "other", pick } });
    const j = reduce(other, { type: "pickDive", pick: { kind: "dive", id: "d2" } });
    expect(j.dive).toEqual({ kind: "dive", id: "d2" });
    expect(j.options).toBe(other.options);
  });

  it("puts the site on the dive's site for a first dive", () => {
    const first = reduce(reduce(at2, { type: "setOptions", options: { site: "other" } }), {
      type: "pickDive",
      pick: { kind: "dive", id: "d1" },
    });
    expect(first.options.site).toBe("diveSite");
  });

  it("changes nothing when the same choice is made again", () => {
    const other = reduce(selectionAt3, { type: "setOptions", options: { site: "other" } });
    expect(reduce(other, { type: "pickDive", pick: { kind: "dive", id: "d1" } })).toBe(other);
  });

  it("keeps the run going", () => {
    expect(reduce(at2, { type: "pickDive", pick: { kind: "dive", id: "d2" } }).run).toBe(at2.run);
  });
});

describe("options of step 4", () => {
  it("change only what is given", () => {
    const j = reduce(catalogAt3, { type: "setOptions", options: changed });
    expect(j.options).toEqual({ ...defaultOptions, ...changed });
    expect(j.run).toBe(catalogAt3.run);
  });

  it("are kept when going back to an earlier step and returning (choice no. 2)", () => {
    const at4 = run(catalogAt3, { type: "next" }, { type: "setOptions", options: changed });
    const back = run(at4, { type: "back" }, { type: "back" }, { type: "next" }, { type: "next" });
    expect(back.options).toEqual(at4.options);
    expect(run(at4, { type: "goTo", step: 1 }, { type: "next" }, { type: "next" }, { type: "next" }).options).toEqual(at4.options);
  });
});

describe("applied", () => {
  it("shows the summary, the options kept for it", () => {
    const at4 = run(catalogAt3, { type: "next" }, { type: "setOptions", options: changed });
    const at5 = reduce(at4, { type: "applied" });
    expect(at5).toEqual({ ...at4, step: 5 });
  });

  it("is ignored away from step 4 (the journey started again meanwhile)", () => {
    expect(reduce(catalogAt3, { type: "applied" })).toBe(catalogAt3);
    const restarted = reduce(catalog, { type: "logChanged" });
    expect(reduce(restarted, { type: "applied" })).toBe(restarted);
  });
});

describe("leaving the summary", () => {
  const at5 = run(catalogAt3, { type: "next" }, { type: "setOptions", options: changed }, { type: "applied" });
  const selectionAt5 = run(selectionAt3, { type: "next" }, { type: "setOptions", options: changed }, { type: "applied" });

  it("back to dives: step 2, no dive ticked, default options, a new run", () => {
    expect(reduce(at5, { type: "backToDives" })).toEqual({
      ...at5,
      step: 2,
      dives: new Set(),
      options: defaultOptions,
      run: at5.run + 1,
    });
  });

  it("handle separately: step 3 with these dives only, default options, a new run", () => {
    expect(reduce(at5, { type: "handleSeparately", ids: ["d2"] })).toEqual({
      ...at5,
      step: 3,
      dives: new Set(["d2"]),
      options: defaultOptions,
      run: at5.run + 1,
    });
  });

  it("done: step 1, everything from scratch but the path, a new run", () => {
    expect(reduce(selectionAt5, { type: "done" })).toEqual({ ...start, path: "selection", run: selectionAt5.run + 1 });
  });
});

describe("menu", () => {
  it("returns to step 1 keeping the choices", () => {
    const at4 = run(catalogAt3, { type: "next" }, { type: "setOptions", options: changed });
    expect(reduce(at4, { type: "menu" })).toEqual({ ...at4, step: 1 });
  });

  it("changes nothing at step 1", () => {
    expect(reduce(catalog, { type: "menu" })).toBe(catalog);
  });

  it("from the summary, unticks the dives as back to dives does, then step 1", () => {
    const at5 = run(catalogAt3, { type: "next" }, { type: "applied" });
    expect(reduce(at5, { type: "menu" })).toEqual({ ...reduce(at5, { type: "backToDives" }), step: 1 });
  });

  it("from the summary of the selection path, starts again as done does", () => {
    const at5 = run(selectionAt3, { type: "next" }, { type: "applied" });
    expect(reduce(at5, { type: "menu" })).toEqual(reduce(at5, { type: "done" }));
  });
});

describe("logChanged", () => {
  it("starts again from step 1, nothing of the old log chosen, with a new run", () => {
    const at4 = run(catalogAt3, { type: "next" }, { type: "setOptions", options: changed });
    expect(reduce(at4, { type: "logChanged" })).toEqual({ ...start, run: at4.run + 1 });
    expect(reduce(selectionAt3, { type: "logChanged" })).toEqual({ ...start, run: selectionAt3.run + 1 });
  });
});

describe("logRead", () => {
  it("goes back to step 2 at steps 3 or 4 when no ticked dive is left in the log (catalog)", () => {
    const at4 = reduce(catalogAt3, { type: "next" });
    for (const j of [catalogAt3, at4]) {
      const after = reduce(j, { type: "logRead", ids: ["d9"] });
      expect(after.step).toBe(2);
      expect(after.dives.size).toBe(0);
      expect(after.options).toEqual(j.options);
      expect(after.run).toBe(j.run);
    }
  });

  it("stays on the step while one ticked dive is left, the others dropped", () => {
    const after = reduce(catalogAt3, { type: "logRead", ids: ["d2", "d9"] });
    expect(after.step).toBe(3);
    expect([...after.dives]).toEqual(["d2"]);
  });

  it("changes nothing when every ticked dive is still there", () => {
    expect(reduce(catalogAt3, { type: "logRead", ids: ["d1", "d2", "d3"] })).toBe(catalogAt3);
    expect(reduce(selectionAt3, { type: "logRead", ids: ["d1"] })).toBe(selectionAt3);
  });

  it("drops a chosen dive no longer in the log and goes back to step 2 (selection)", () => {
    const after = reduce(selectionAt3, { type: "logRead", ids: ["d2"] });
    expect(after.step).toBe(2);
    expect(after.dive).toEqual({ kind: "none" });
  });

  it("leaves the summary as it is: what was written does not change", () => {
    const at5 = run(catalogAt3, { type: "next" }, { type: "applied" });
    expect(reduce(at5, { type: "logRead", ids: [] })).toBe(at5);
    const selectionAt5 = run(selectionAt3, { type: "next" }, { type: "applied" });
    expect(reduce(selectionAt5, { type: "logRead", ids: [] })).toBe(selectionAt5);
  });

  it("leaves step 2 where it is", () => {
    const at2 = reduce(catalogAt3, { type: "back" });
    expect(reduce(at2, { type: "logRead", ids: [] }).step).toBe(2);
  });
});

describe("stepStates", () => {
  it("marks the steps done, current and next", () => {
    expect(stepStates(catalogAt3)).toEqual(["done", "done", "on", "next", "next"]);
    expect(stepStates(start)).toEqual(["on", "next", "next", "next", "next"]);
  });

  it("greys the clocks for no dive", () => {
    expect(stepStates(noDiveAt4)).toEqual(["done", "done", "skipped", "on", "next"]);
    expect(stepStates(run(noDiveAt4, { type: "applied" }))).toEqual(["done", "done", "skipped", "done", "on"]);
  });
});
