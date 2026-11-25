// Step 5 of the journey, as the screen shows it. Fictional sites and dives only
// (rules/fixtures.ts). Texts are compared word for word with the specification (F5).
import { beforeEach, describe, expect, it } from "vitest";
import { setLanguage } from "../i18n";
import { reduce } from "./state";
import type { Applied } from "./rules";
import { collectionLine, emptyDives, separatelyLabel, summaryNotices, writtenCollections } from "./summary";
import { after7, catalogAt4, d3, d4, log, run, selectionAt4 } from "./rules/fixtures";

beforeEach(() => setLanguage("en"));

const catalogDone: Applied = {
  path: "catalog",
  result: {
    collections: [
      { collection: "12 - Blue Arch", count: 20, created: true },
      { collection: "13 - Sand Patch", count: 6, created: false },
    ],
    photos: 26,
    withDepth: 15,
    emptyDives: ["d3"],
  },
  outside: 9,
  cameras: [],
  ignored: [],
};

const selectionDone: Applied = {
  path: "selection",
  result: { count: 3, withDepth: 2, collection: { collection: "12 - Blue Arch", count: 3, created: true } },
  outside: [after7],
  outsideMore: 0,
  perPhoto: true,
};

describe("writtenCollections", () => {
  it("the collections written, never one without a name", () => {
    const unnamed: Applied = {
      ...catalogDone,
      result: {
        ...catalogDone.result,
        collections: [...(catalogDone as Extract<Applied, { path: "catalog" }>).result.collections, { collection: null, count: 4, created: false }, { collection: "", count: 1, created: true }],
      },
    };
    expect(writtenCollections(unnamed).map((c) => c.collection)).toEqual(["12 - Blue Arch", "13 - Sand Patch"]);
  });

  it("selection path: the dive's collection, or none", () => {
    expect(writtenCollections(selectionDone)).toEqual([{ collection: "12 - Blue Arch", count: 3, created: true }]);
    const none: Applied = { ...selectionDone, result: { count: 3, withDepth: 2, collection: null } };
    expect(writtenCollections(none)).toEqual([]);
  });
});

describe("collectionLine", () => {
  it("its photos and whether it was created or added to, in both languages", () => {
    expect(collectionLine({ collection: "12 - Blue Arch", count: 20, created: true })).toBe("20 photos · created");
    expect(collectionLine({ collection: "13 - Sand Patch", count: 1, created: false })).toBe("1 photo · added to");
    setLanguage("fr");
    expect(collectionLine({ collection: "12 - Blue Arch", count: 20, created: true })).toBe("20 photos · créée");
    expect(collectionLine({ collection: "13 - Sand Patch", count: 6, created: false })).toBe("6 photos · complétée");
  });
});

describe("emptyDives and separatelyLabel", () => {
  it("catalog path: the dives that got no photo, as the log has them", () => {
    const gone: Applied = { ...catalogDone, result: { ...catalogDone.result, emptyDives: ["d3", "no-longer-in-the-log"] } } as Applied;
    expect(emptyDives(gone, log.dives)).toEqual([d3]);
    expect(emptyDives(selectionDone, log.dives)).toEqual([]);
  });

  it("one dive by its name, several together, none without a button", () => {
    expect(separatelyLabel([d3])).toBe("Handle dive 14 separately");
    expect(separatelyLabel([d3, d4])).toBe("Handle these dives separately");
    expect(separatelyLabel([])).toBeNull();
    setLanguage("fr");
    expect(separatelyLabel([d3])).toBe("Traiter la plongée 14 à part");
    expect(separatelyLabel([d3, d4])).toBe("Traiter ces plongées à part");
  });
});

describe("summaryNotices", () => {
  it("catalog path: a dive without photo in orange, the photos outside the dives not", () => {
    const j = run(catalogAt4, { type: "goTo", step: 2 }, { type: "toggleDive", id: "d3" }, { type: "next" }, { type: "next" }, { type: "applied" });
    expect(summaryNotices({ journey: j, applied: catalogDone, dives: log.dives })).toEqual([
      {
        text: "Dive 14: no photo, no collection created. If the clock changed that day, handle it separately with its own offset.",
        warn: true,
      },
      { text: "9 photos outside the dives got nothing.", warn: false },
    ]);
  });

  it("selection path: the photos without depth, not in orange", () => {
    const j = reduce(selectionAt4, { type: "applied" });
    expect(summaryNotices({ journey: j, applied: selectionDone, dives: log.dives })).toEqual([
      { text: "1 photo without depth: taken after getting out of the water.", warn: false },
    ]);
  });
});
