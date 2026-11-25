// The rules the two photo readings of the journey follow (spec F5, steps 3
// and 4), pure so that they are tested: useCatalogPhotos looks for the photos
// of the dives ticked in the whole catalog, useSelectionPhotos describes the
// photos selected in Lightroom. Taken over from Create Collections and Tag
// Selection, unchanged.

import { initialOffsets } from "../apply/cameras";
import { t } from "../i18n";
import type { ApplyError, Camera, Dive, LightroomStatus, Period, SelectionPhotos, TagPreview } from "../types";

/** The catalog path's photo search, as it stands. */
export type Search =
  | { state: "idle" }
  | { state: "searching" }
  | { state: "ready"; period: Period }
  | { state: "failed"; message: string };

/** The selection described photo by photo; `version`: the selection's, as Lightroom numbers it. */
export interface Described {
  photos: SelectionPhotos;
  version: number;
}

/** Live counts of the selection, and the description they were computed for. */
export interface Counts {
  search: number;
  preview: TagPreview;
}

/**
 * The dives a photo search names, as one text: those of the log among `ids`, by start
 * time, so that the same dives ticked in another order are the same search; "" without any.
 */
export function diveKey(dives: Dive[], ids: Iterable<string>): string {
  const wanted = new Set(ids);
  return dives
    .filter((d) => wanted.has(d.id))
    .sort((a, b) => (a.start < b.start ? -1 : 1))
    .map((d) => d.id)
    .join(",");
}

/** The cameras of `set` still among `cameras`. */
export function onlyCameras(set: Set<string>, cameras: Camera[]): Set<string> {
  const keys = new Set(cameras.map((c) => c.key));
  return new Set([...set].filter((k) => keys.has(k)));
}

/** The notes of the cameras still among `cameras`. */
export function notesOfCameras(notes: Record<string, string>, cameras: Camera[]): Record<string, string> {
  const keys = new Set(cameras.map((c) => c.key));
  return Object.fromEntries(Object.entries(notes).filter(([k]) => keys.has(k)));
}

/**
 * The offsets once the photos are found. `keep`: the same dives found again (Lightroom
 * came back), so each camera still there keeps the offset shown; otherwise, and for a new
 * camera, the offset last applied to it, else none.
 */
export function offsetsAfterSearch(shown: Record<string, number>, cameras: Camera[], keep: boolean): Record<string, number> {
  if (!keep) return initialOffsets(cameras);
  const keys = new Set(cameras.map((c) => c.key));
  const kept = Object.fromEntries(Object.entries(shown).filter(([k]) => keys.has(k)));
  return { ...initialOffsets(cameras), ...kept };
}

/** The cameras ignored, or set by hand, once the photos are found: kept as `offsetsAfterSearch` keeps offsets. */
export function keysAfterSearch(set: Set<string>, cameras: Camera[], keep: boolean): Set<string> {
  return keep ? onlyCameras(set, cameras) : new Set();
}

/**
 * The surface margin when a new run starts (catalog path). It goes on with dives
 * ticked only through "handle separately", which kept the margin in 2.1.1; every other
 * new run starts from step 1 or 2 with no dive ticked, where the margin started from 0.
 */
export function marginAfterRun(margin: number, diveIds: string[]): number {
  return diveIds.length > 0 ? margin : 0;
}

/**
 * The surface margin once the photos are found (catalog path). Kept for the same dives
 * found again (`keep`: Lightroom came back), and on the run's first search, where
 * marginAfterRun decided; other dives ticked in the same run start it from 0, as a new
 * screen did in 2.1.1: a margin forgotten would let boat photos in silently.
 */
export function marginAfterSearch(margin: number, keep: boolean, firstOfRun: boolean): number {
  return keep || firstOfRun ? margin : 0;
}

/** "Suggest for all": the cameras neither ignored nor set by hand. */
export function camerasToSuggest(cameras: Camera[], touched: Set<string>, ignored?: Set<string>): string[] {
  return cameras.filter((c) => !ignored?.has(c.key) && !touched.has(c.key)).map((c) => c.key);
}

/** The offsets once suggestions came back for `keys`: each camera gets the offset found, when one was. */
export function withSuggestions(
  offsets: Record<string, number>,
  keys: string[],
  found: Record<string, number | null>,
): Record<string, number> {
  const next = { ...offsets };
  for (const k of keys) {
    const value = found[k];
    if (value !== null && value !== undefined) next[k] = value;
  }
  return next;
}

/** The notes once suggestions came back for `keys`: `none` where no offset fits, cleared where one was found. */
export function suggestionNotes(
  notes: Record<string, string>,
  keys: string[],
  found: Record<string, number | null>,
  none: string,
): Record<string, string> {
  const next = { ...notes };
  for (const k of keys) next[k] = found[k] === null ? none : "";
  return next;
}

/**
 * Whether the selection is read photo by photo: for a dive whose profile can be read,
 * as before, or as soon as the journey asks for it (`active`: selection path, step 2
 * on, choice no. 3); always with Lightroom connected and photos selected.
 */
export function readsSelection(
  active: boolean,
  needsValues: boolean,
  lr: Pick<LightroomStatus, "connected" | "selected">,
): boolean {
  return (active || needsValues) && lr.connected && lr.selected > 0;
}

/** The description Apply may use: of the selection shown, and done. */
export function currentDescription(
  described: Described | null,
  selectionVersion: number,
  reading: boolean,
  readFailure: string | null,
): SelectionPhotos | null {
  return described && described.version === selectionVersion && !reading && !readFailure ? described.photos : null;
}

/** The counts of the description Apply may use; null until they arrive. */
export function freshCountsOf(counts: Counts | null, current: SelectionPhotos | null): TagPreview | null {
  return counts && current && counts.search === current.search ? counts.preview : null;
}

/** Nothing was written because the selection is no longer the one described: it is described again. */
export function selectionChanged(e: ApplyError): boolean {
  if (e.error === "lightroom") return e.reason === "selectionChanged" || e.reason === "staleSelection";
  return e.error === "noPhotos";
}

/**
 * The selection is described again after a failed Apply: when the selection moved on, or when Lightroom
 * answered while per-photo values were asked (the offsets applied may now be the remembered ones).
 */
export function describeAgain(e: ApplyError, needsValues: boolean): boolean {
  return selectionChanged(e) || (e.error === "lightroom" && needsValues);
}

/** What the reading does when describing the selection failed; `message` undefined: the step words the failure. */
export interface AfterFailure {
  reading: boolean;
  message?: string | null;
  /** Describe the selection once more (the answer was lost, not the selection). */
  describeAgain: boolean;
}

/**
 * The state after a failed description of the selection. null: a newer description replaced this one and owns
 * the reading, nothing changes. Otherwise the reading always ends. A lost description (noPhotos) is asked again
 * once per selection (`retried`: already done); the second time it says why. A changed log needs nothing: the
 * app reads it again and starts the journey again.
 */
export function readingAfterFailure(e: ApplyError, live: boolean, retried = false): AfterFailure | null {
  if (!live) return null;
  if (e.error === "logChanged") return { reading: false, message: null, describeAgain: false };
  if (e.error === "noPhotos") {
    return retried
      ? { reading: false, message: t("tag.perPhoto.failedNoDetail"), describeAgain: false }
      : { reading: false, message: null, describeAgain: true };
  }
  return { reading: false, describeAgain: false };
}
