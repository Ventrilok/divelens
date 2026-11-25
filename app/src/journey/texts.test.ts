// Runs in UTC+14 (vitest.config.ts): every time below is a wall-clock time
// and must come out as written.
import { beforeEach, describe, expect, it } from "vitest";
import { setLanguage } from "../i18n";
import { reduce, start, type Action, type Journey } from "./state";
import { diveLabel, outsideLines, outsideSentence, photosLine, photoTime, selectionSummary } from "./texts";
import type { CameraRange, Dive, LightroomStatus, OutsidePhoto, Period } from "../types";

beforeEach(() => setLanguage("en"));

const run = (from: Journey, ...actions: Action[]) => actions.reduce(reduce, from);

/** A fictional dive. */
const dive = (id: string, number: number | null, site: string | null, start = "2026-03-06T09:00:00"): Dive => ({
  id,
  number,
  start,
  end: `${start.slice(0, 10)}T09:50:00`,
  durationS: 3000,
  maxDepthM: 18,
  tempHighC: 27,
  tempLowC: 26,
  rating: null,
  site: { id: site && `s-${id}`, name: site, location: null, country: null, lat: null, lon: null },
  hasProfile: true,
  invalid: false,
  notes: null,
});

const dives = [dive("d1", 12, "Blue Arch"), dive("d2", 13, "Sand Patch", "2026-03-07T09:00:00")];

const camera = (model: string | null, serial: string | null = null): CameraRange => ({
  model,
  serial,
  from: "2026-03-06T08:40:00",
  to: "2026-03-06T10:10:00",
});

/** Five photos selected in Lightroom, one camera, on 6 March 2026. */
const lr: Pick<LightroomStatus, "selected" | "selectionCameras" | "selectionFrom" | "selectionTo"> = {
  selected: 5,
  selectionCameras: [camera("Olympus TG-6")],
  selectionFrom: "2026-03-06T08:40:00",
  selectionTo: "2026-03-06T10:10:00",
};

/** The photos found for d1 and d2: the search looks one day wider on each side than the dives. */
const period: Period = {
  search: 3,
  from: "2026-03-05",
  to: "2026-03-08",
  photos: 40,
  undated: 0,
  cameras: [
    { key: "a", model: "Olympus TG-6", serial: null, photos: 30, remembered: null },
    { key: "b", model: "Phone", serial: null, photos: 10, remembered: null },
  ],
};

const catalogAt2 = run(start, { type: "choosePath", path: "catalog" }, { type: "next" }, { type: "setDives", ids: ["d1", "d2"] });
const catalogAt3 = reduce(catalogAt2, { type: "next" });
const selectionAt2 = run(start, { type: "choosePath", path: "selection" }, { type: "next" });
const selectionAt3 = run(selectionAt2, { type: "pickDive", pick: { kind: "dive", id: "d1" } }, { type: "next" });
const noDiveAt4 = run(selectionAt2, { type: "pickDive", pick: { kind: "noDive" } }, { type: "next" });

describe("diveLabel", () => {
  it("is the dive's number and site, as the screens name a dive", () => {
    expect(diveLabel(dive("d", 707, "Coral Steps"))).toBe("707 Coral Steps");
    expect(diveLabel(dive("d", null, "Coral Steps"))).toBe("Coral Steps");
    expect(diveLabel(dive("d", 14, null))).toBe("14");
    expect(diveLabel(dive("d", null, null))).toBe("—");
  });
});

describe("photoTime", () => {
  it("is the hour and minute of a wall-clock time, never shifted", () => {
    expect(photoTime("2026-03-06T15:45:10")).toBe("15:45");
    expect(photoTime("2026-03-06T00:05:59")).toBe("00:05");
  });
});

describe("outsideSentence", () => {
  const after: OutsidePhoto = { time: "2026-03-06T15:45:00", camera: "a", minutes: 7 };

  it("says when a photo was taken after getting out of the water, in both languages", () => {
    expect(outsideSentence(after)).toBe(
      "Taken at 15:45, 7 minutes after getting out of the water. Attached to the dive anyway: it gets its data and its location, without depth.",
    );
    setLanguage("fr");
    expect(outsideSentence(after)).toBe(
      "Prise à 15:45, 7 minutes après la sortie de l'eau. Rattachée quand même à la plongée : elle reçoit ses infos et son lieu, sans profondeur.",
    );
  });

  it("says when it was taken before going into the water", () => {
    const before: OutsidePhoto = { time: "2026-03-06T08:57:00", camera: "a", minutes: -3 };
    expect(outsideSentence(before)).toBe(
      "Taken at 08:57, 3 minutes before going into the water. Attached to the dive anyway: it gets its data and its location, without depth.",
    );
    setLanguage("fr");
    expect(outsideSentence({ ...before, minutes: -1 })).toBe(
      "Prise à 08:57, 1 minute avant la mise à l'eau. Rattachée quand même à la plongée : elle reçoit ses infos et son lieu, sans profondeur.",
    );
  });
});

describe("outsideLines", () => {
  const photo = (minutes: number): OutsidePhoto => ({ time: "2026-03-06T10:00:00", camera: "a", minutes });

  it("details at most five photos, then tells how many more (choice no. 4)", () => {
    const lines = outsideLines([1, 2, 3, 4, 5].map(photo), 2);
    expect(lines).toHaveLength(6);
    expect(lines[0]).toBe(outsideSentence(photo(1)));
    expect(lines[5]).toBe("and 2 more");
    setLanguage("fr");
    expect(outsideLines([photo(1)], 2)[1]).toBe("et 2 autres");
    expect(outsideLines([photo(1)], 1)[1]).toBe("et 1 autre");
  });

  it("adds nothing when every photo is detailed", () => {
    expect(outsideLines([photo(4)], 0)).toEqual([outsideSentence(photo(4))]);
  });

  it("never details more than five photos, the others counted", () => {
    const lines = outsideLines([1, 2, 3, 4, 5, 6, 7].map(photo), 1);
    expect(lines).toHaveLength(6);
    expect(lines[5]).toBe("and 3 more");
  });
});

describe("selectionSummary", () => {
  it("names the cameras and the day of the selection, in both languages", () => {
    expect(selectionSummary(lr)).toBe("Olympus TG-6 · Friday, 6 March 2026");
    setLanguage("fr");
    expect(selectionSummary(lr)).toBe("Olympus TG-6 · vendredi 6 mars 2026");
  });

  it("names each camera model once, an unknown one as such", () => {
    const cameras = [camera("Olympus TG-6", "1"), camera("Olympus TG-6", "2"), camera(null)];
    expect(selectionSummary({ ...lr, selectionCameras: cameras })).toBe("Olympus TG-6, Unknown camera · Friday, 6 March 2026");
  });

  it("gives the first and last day of a selection over several days", () => {
    expect(selectionSummary({ ...lr, selectionTo: "2026-03-08T07:00:00" })).toBe("Olympus TG-6 · 6 Mar 2026 → 8 Mar 2026");
  });

  it("leaves out what Lightroom did not give", () => {
    expect(selectionSummary({ ...lr, selectionFrom: null, selectionTo: null })).toBe("Olympus TG-6");
    expect(selectionSummary({ ...lr, selectionCameras: [] })).toBe("Friday, 6 March 2026");
  });
});

describe("photosLine", () => {
  const i = { lr, dives, period: null };

  it("is not shown at step 1 nor on the summary", () => {
    expect(photosLine(start, i)).toBeNull();
    const at5 = run(catalogAt3, { type: "next" }, { type: "applied" });
    expect(photosLine(at5, i)).toBeNull();
  });

  it("step 2, catalog path: the whole catalog, dives to tick", () => {
    expect(photosLine(catalogAt2, i)).toBe(
      "Photos: entire catalog · tick the dives, Dive Lens finds the photos taken during each one",
    );
    setLanguage("fr");
    expect(photosLine(catalogAt2, i)).toBe(
      "Photos : tout le catalogue · cochez les plongées, Dive Lens retrouve les photos prises pendant chacune",
    );
  });

  it("step 2, selection path: the photos selected, their cameras and day", () => {
    expect(photosLine(selectionAt2, i)).toBe("Photos: the 5 selected in Lightroom · Olympus TG-6 · Friday, 6 March 2026");
    setLanguage("fr");
    expect(photosLine(selectionAt2, i)).toBe("Photos : les 5 sélectionnées dans Lightroom · Olympus TG-6 · vendredi 6 mars 2026");
    expect(photosLine(selectionAt2, { ...i, lr: { ...lr, selected: 1 } })).toBe(
      "Photos : la photo sélectionnée dans Lightroom · Olympus TG-6 · vendredi 6 mars 2026",
    );
  });

  it("step 3, catalog path: the days of the dives ticked, the photos found and the dives ticked", () => {
    expect(photosLine(catalogAt3, { ...i, period })).toBe(
      "Photos: entire catalog, from 6 Mar 2026 to 7 Mar 2026 · 40 photos from 2 cameras · 2 dives ticked",
    );
    setLanguage("fr");
    expect(photosLine(catalogAt3, { ...i, period })).toBe(
      "Photos : tout le catalogue, du 6 mars 2026 au 7 mars 2026 · 40 photos de 2 appareils · 2 plongées cochées",
    );
    // Whatever the order of the log.
    expect(photosLine(catalogAt3, { ...i, period, dives: [...dives].reverse() })).toBe(
      "Photos : tout le catalogue, du 6 mars 2026 au 7 mars 2026 · 40 photos de 2 appareils · 2 plongées cochées",
    );
  });

  it("step 3, catalog path, while the photos are looked for: the dives ticked", () => {
    expect(photosLine(catalogAt3, i)).toBe("Photos: entire catalog · 2 dives ticked");
  });

  it("step 3, selection path: the photos selected, their cameras and the dive", () => {
    expect(photosLine(selectionAt3, i)).toBe("Photos: the 5 selected in Lightroom · 1 camera · dive 12 Blue Arch");
    setLanguage("fr");
    expect(photosLine(selectionAt3, i)).toBe("Photos : les 5 sélectionnées dans Lightroom · 1 appareil · plongée 12 Blue Arch");
  });

  it("step 4 repeats step 3's line; step 2's for no dive", () => {
    expect(photosLine(reduce(selectionAt3, { type: "next" }), i)).toBe(photosLine(selectionAt3, i));
    expect(photosLine(reduce(catalogAt3, { type: "next" }), { ...i, period })).toBe(photosLine(catalogAt3, { ...i, period }));
    expect(photosLine(noDiveAt4, i)).toBe(photosLine(selectionAt2, i));
  });
});
