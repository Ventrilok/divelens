// Create Collections, step 2: when Apply is allowed and what the bar says.
// Pure rules, kept apart from the screen so that they can be tested.

import { t } from "../i18n";
import type { CollectionsPreview, Period, Units } from "../types";

export type Search =
  | { state: "idle" }
  | { state: "searching" }
  | { state: "ready"; period: Period }
  | { state: "failed"; message: string };

/** What blocks Apply, in the order the user meets it; null when nothing does. */
export function collectionsBlocker(i: {
  connected: boolean;
  search: Search;
  parentMissing: boolean;
  parent: string[];
  preview: CollectionsPreview | null;
}): string | null {
  if (!i.connected) return t("coll.disconnected");
  if (i.search.state === "searching") return t("coll.searching");
  if (i.search.state === "failed") return i.search.message;
  if (i.parentMissing) return t("coll.parentMissing", { path: i.parent.join(" › ") });
  if (i.preview && i.preview.total === 0) return t("coll.err.nothingToWrite");
  return null;
}

/** Apply also waits for the counts: without them there is no blocker yet, but nothing to write either. */
export function canApplyCollections(blocker: string | null, preview: CollectionsPreview | null, writing: boolean): boolean {
  return blocker === null && preview !== null && !writing;
}

/** The bar's text: the blocker, else where the collections go. */
export function barText(blocker: string | null, place: string): string {
  return blocker ?? t("coll.writes", { place });
}

/** The bar's second line: the units and whether depth is copied to altitude. */
export function writesNote(units: Units, depthToAltitude: boolean): string {
  const u = `${units.depth === "ft" ? "ft" : "m"}, ${units.temp === "f" ? "°F" : "°C"}`;
  return t("coll.writesNote", { units: u, altitude: t(depthToAltitude ? "coll.on" : "coll.off") });
}
