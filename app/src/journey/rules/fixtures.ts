// A fictional dive log, journeys and photo readings for the tests of the
// journey's rules: invented sites and dives only.
import { reduce, start, type Action, type Journey, type Options } from "../state";
import type { ReviewInput, SelectionReview } from "../rules";
import type { Search } from "../../collections/rules";
import type { CollectionsPreview, Dive, OutsidePhoto, Site, TagPreview } from "../../types";

export const run = (from: Journey, ...actions: Action[]) => actions.reduce(reduce, from);
export const withOptions = (j: Journey, options: Partial<Options>) => reduce(j, { type: "setOptions", options });

// --- A fictional dive log ----------------------------------------------------

export const blueArch: Site = { id: "s1", name: "Blue Arch", location: "North Bay", country: "Nowhere Islands", lat: -8.5, lon: 115.25 };
/** A site without GPS. */
export const sandPatch: Site = { id: "s2", name: "Sand Patch", location: "North Bay", country: "Nowhere Islands", lat: null, lon: null };
export const noSite: Site = { id: null, name: null, location: null, country: null, lat: null, lon: null };
/** A site of the log with a position but no name, location or country. */
export const nameless: Site = { id: "s3", name: null, location: null, country: null, lat: -8.4, lon: 115.3 };

export const dive = (id: string, number: number, site: Site, start: string, end: string, hasProfile: boolean): Dive => ({
  id,
  number,
  start,
  end,
  durationS: 3000,
  maxDepthM: 18,
  tempHighC: 27,
  tempLowC: 26,
  rating: null,
  site,
  hasProfile,
  invalid: false,
  notes: null,
});

export const d1 = dive("d1", 12, blueArch, "2025-12-30T09:00:00", "2025-12-30T09:50:00", true);
export const d2 = dive("d2", 13, sandPatch, "2025-12-31T10:00:00", "2025-12-31T10:45:00", false);
export const d3 = dive("d3", 14, noSite, "2026-01-02T08:00:00", "2026-01-02T08:40:00", true);
export const d4 = dive("d4", 15, nameless, "2026-01-03T08:00:00", "2026-01-03T08:40:00", false);
export const log = { dives: [d1, d2, d3, d4], sites: [blueArch, sandPatch, nameless] };

/** The site picked under "Other site", its GPS as the picker fills it. */
export const pickBlueArch = { country: "Nowhere Islands", location: "North Bay", siteId: "s1", lat: "-8.5", lon: "115.25" };
export const pickSandPatch = { country: "Nowhere Islands", location: "North Bay", siteId: "s2", lat: "", lon: "" };

// --- Journeys ------------------------------------------------------------------

export const catalog = reduce(start, { type: "choosePath", path: "catalog" });
export const selection = reduce(start, { type: "choosePath", path: "selection" });
/** Catalog path at step 4, dives 12 and 13 ticked. */
export const catalogAt4 = run(catalog, { type: "next" }, { type: "setDives", ids: ["d1", "d2"] }, { type: "next" }, { type: "next" });
/** Selection path at step 4, dive 12 chosen. */
export const selectionAt4 = run(selection, { type: "next" }, { type: "pickDive", pick: { kind: "dive", id: "d1" } }, { type: "next" }, { type: "next" });
/** Selection path at step 4 with "no dive — location only". */
export const noDiveAt4 = run(selection, { type: "next" }, { type: "pickDive", pick: { kind: "noDive" } }, { type: "next" });

// --- What the photo readings show ---------------------------------------------

/** Dive 12: 20 photos, 15 in the dive; dive 13: 6 photos; 9 outside the dives. */
export const preview: CollectionsPreview = {
  dives: [
    { id: "d1", photos: 20, inDive: 15, cameras: 1 },
    { id: "d2", photos: 6, inDive: 6, cameras: 1 },
  ],
  cameras: [],
  total: 26,
  outside: 9,
  ignored: 0,
};
export const ready: Search = {
  state: "ready",
  period: { search: 4, from: "2025-12-30", to: "2025-12-31", photos: 35, undated: 0, cameras: [] },
};

export const after7: OutsidePhoto = { time: "2025-12-30T09:57:00", camera: "a", minutes: 7 };
export const before3: OutsidePhoto = { time: "2025-12-30T08:57:00", camera: "a", minutes: -3 };
/** Three photos selected, two in dive 12, one taken after it. */
export const counts: TagPreview = { total: 3, inDive: 2, cameras: [], coverFrom: null, coverTo: null, outside: [after7], outsideMore: 0 };

export const nothingRead: SelectionReview = { needsValues: false, profile: { status: "loading" }, readFailure: null, current: null, freshCounts: null };
export const read: SelectionReview = {
  needsValues: true,
  profile: { status: "ok" },
  readFailure: null,
  current: { search: 7, photos: 3, undated: 0, cameras: [] },
  freshCounts: counts,
};

export const input = (journey: Journey, over: Partial<ReviewInput> = {}): ReviewInput => ({
  journey,
  lr: { connected: true, selected: journey.path === "selection" ? 3 : 0 },
  log,
  parentMissing: false,
  parent: ["Trips", "2025"],
  catalog: { search: ready, preview, previewFailed: false },
  selection: journey.path === "selection" && journey.dive.kind === "dive" ? read : nothingRead,
  ...over,
});
