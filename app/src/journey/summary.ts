// Step 5 of the journey, "Summary" (spec F5), as the screen shows the rules of
// rules/summary.ts: the collections written, the warnings, and the buttons of
// the catalog path for the dives that got no photo. Pure, so that every text
// is tested.

import { t, tn } from "../i18n";
import type { Dive, WrittenCollection } from "../types";
import { summary, type Applied, type SummaryInput } from "./rules";
import { diveLabel } from "./texts";

/** The collections written, each under its name: never a line without one. */
export function writtenCollections(a: Applied): WrittenCollection[] {
  const all = a.path === "catalog" ? a.result.collections : a.result.collection ? [a.result.collection] : [];
  return all.filter((c) => c.collection !== null && c.collection !== "");
}

/** "20 photos · created" or "… · added to". */
export function collectionLine(c: WrittenCollection): string {
  return `${tn("coll.photos", c.count)} · ${t(c.created ? "coll.result.created" : "coll.result.completed")}`;
}

/** Catalog path: the dives ticked that got no photo, as the log has them; none on the selection path. */
export function emptyDives(a: Applied, dives: Dive[]): Dive[] {
  if (a.path !== "catalog") return [];
  const byId = new Map(dives.map((d) => [d.id, d]));
  return a.result.emptyDives.map((id) => byId.get(id)).filter((d): d is Dive => d !== undefined);
}

/** "Handle dive N separately" or "Handle these dives separately"; null without a dive to handle. */
export function separatelyLabel(empty: Dive[]): string | null {
  if (empty.length === 0) return null;
  return empty.length === 1 ? t("coll.result.retryOne", { dive: diveLabel(empty[0]) }) : t("coll.result.retryMany");
}

/** The summary's warnings, the dives without photo in orange (they come first, catalog path). */
export function summaryNotices(i: SummaryInput): { text: string; warn: boolean }[] {
  const empty = emptyDives(i.applied, i.dives).length;
  return summary(i).warnings.map((text, n) => ({ text, warn: n < empty }));
}
