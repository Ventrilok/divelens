// Step 4 of the journey, "Review & Apply" (spec F5), as the screen shows the
// rules of rules/review.ts: each box ticked or greyed, each effect in full or
// muted, the site written, the collection's example, the bar, and what Apply
// hands to the summary and reads again once Lightroom answered. Pure, so that
// every decision of the screen is tested.

import { searchAgain } from "../apply/failure";
import { groupName, placeLabel, type Place } from "../collections/collectionPlace";
import { t } from "../i18n";
import { DASH } from "../ui/format";
import type {
  ApplyError,
  CollectionChoices,
  CollectionsPreview,
  CollectionsResult,
  Dive,
  Period,
  TagApplied,
  TagChoice,
  TagPreview,
} from "../types";
import {
  barSummary,
  canApply,
  chosenDive,
  effects,
  goodToKnow,
  reviewBlocker,
  reviewCounts,
  siteDetails,
  siteSource,
  tickedDives,
  writeOptions,
  type Applied,
  type ReviewInput,
} from "./rules";
import { describeAgain } from "./readings";
import { pickAround } from "./SitePicker";
import { noDive, type Journey, type Options, type Path } from "./state";

/** A box as the screen shows it. */
export interface Box {
  checked: boolean;
  enabled: boolean;
}

export interface Boxes {
  dive: Box;
  altitude: Box;
  /** "Site details": ticked while "Location" or "GPS" is. */
  site: Box;
  place: Box;
  gps: Box;
  collections: Box;
}

/** The boxes of step 4. A greyed box shows what is written: nothing, so it is unticked. */
export function reviewBoxes(j: Journey): Boxes {
  const o = j.options;
  const w = writeOptions(j);
  const withDive = !noDive(j);
  const site = siteDetails(o);
  return {
    dive: { checked: w.dive, enabled: withDive },
    altitude: { checked: w.altitude, enabled: w.dive },
    site: { checked: site, enabled: true },
    place: { checked: o.place, enabled: site },
    gps: { checked: o.gps, enabled: site },
    collections: { checked: w.collections, enabled: withDive },
  };
}

/** "Site details" (un)ticked: unticking it unticks "Location" and "GPS"; ticking it ticks both. */
export function siteDetailsTicked(on: boolean): Partial<Options> {
  return { place: on, gps: on };
}

/** An option's effect on this run; `on`: it writes something (shown in full, else muted). */
export interface Effect {
  text: string;
  on: boolean;
}

export function reviewEffects(i: ReviewInput): { dive: Effect; altitude: Effect; site: Effect; collections: Effect } {
  const text = effects(i);
  const w = writeOptions(i.journey);
  const c = reviewCounts(i);
  return {
    dive: { text: text.dive, on: w.dive },
    altitude: { text: text.altitude, on: w.altitude },
    site: { text: text.site, on: c.site > 0 },
    collections: { text: text.collections, on: c.collections > 0 },
  };
}

/** The site written: two radio buttons, and the picker under "Other site". */
export interface SiteRadios {
  /** The radio button on: where the site written comes from on this run. */
  source: "diveSite" | "other";
  diveSite: { label: string; enabled: boolean };
  other: { label: string; enabled: boolean };
  /** The site picker (country › location › site, then its GPS) shows. */
  picker: boolean;
}

/**
 * Catalog path: "Each dive's site" or "Other site, the same for all photos…". Selection
 * path: "Dive's site: <name>" or "Other site…"; the dive's site greyed for a dive without a
 * site or with no dive, as before. Everything greyed with "Site details" unticked.
 */
export function siteRadios(j: Journey, dives: Dive[]): SiteRadios {
  const on = siteDetails(j.options);
  const dive = chosenDive(j, dives);
  const source = siteSource(j, dive);
  if (j.path === "catalog") {
    return {
      source,
      diveSite: { label: t("journey.site.eachDive"), enabled: on },
      other: { label: t("journey.site.otherSame"), enabled: on },
      picker: on && source === "other",
    };
  }
  const name = !dive ? t("tag.noDiveChosen") : dive.site.id === null ? t("tag.diveHasNoSite") : (dive.site.name ?? DASH);
  return {
    source,
    diveSite: { label: t("journey.site.diveSite", { name }), enabled: on && dive !== null && dive.site.id !== null },
    other: { label: t("journey.site.other"), enabled: on },
    picker: on && source === "other",
  };
}

/**
 * "Other site" chosen: with no site picked yet, the picker opens on the chosen dive's
 * country and location, as before (catalog path: empty).
 */
export function chooseOtherSite(j: Journey, dives: Dive[]): Partial<Options> {
  const pick = j.options.pick;
  return { site: "other", pick: pick.siteId === null ? pickAround(chosenDive(j, dives)?.site ?? null) : pick };
}

/**
 * A dive's collection as the engine names it (core collections::collection_name):
 * "<number> - <site>", what it has of them, or its start, read from the wall-clock text.
 */
export function collectionName(d: Dive): string {
  const start = `${d.start.slice(0, 10)} ${d.start.slice(11, 16)}`;
  if (d.number !== null) return d.site.name !== null ? `${d.number} - ${d.site.name}` : String(d.number);
  return d.site.name !== null ? `${start} - ${d.site.name}` : start;
}

/** "Trips › Dive Lens › 12 - Blue Arch": a collection under the place chosen in Settings; the place alone without one. */
export function collectionPath(place: Place, name: string | null): string {
  const where = placeLabel(place, t("settings.root"));
  if (name === null) return where;
  const inRoot = place.parent.length === 0 && groupName(place) === null;
  return inRoot ? name : `${where} › ${name}`;
}

/**
 * The text of "Create collections", with the place chosen in Settings. Catalog path: a
 * dive ticked as an example, the first one with photos once counted. Selection path:
 * the chosen dive's collection; the place alone for no dive.
 */
export function collectionsText(j: Journey, place: Place, dives: Dive[], preview: CollectionsPreview | null): string {
  if (j.path === "catalog") {
    const ticked = tickedDives(j, dives);
    const photos = new Map(preview?.dives.map((d) => [d.id, d.photos]) ?? []);
    const example = ticked.find((d) => (photos.get(d.id) ?? 0) > 0) ?? ticked[0];
    return t("journey.opt.collectionsCatalog", { path: collectionPath(place, example ? collectionName(example) : null) });
  }
  const dive = chosenDive(j, dives);
  return t("journey.opt.collectionsSelection", { path: collectionPath(place, dive ? collectionName(dive) : null) });
}

/**
 * The bar's main line: what blocks Apply, in place of what it writes (as before);
 * nothing on the catalog path while the photos are being counted.
 */
export function reviewBar(i: ReviewInput): { text: string | null; blocked: boolean } {
  const blocker = reviewBlocker(i);
  if (blocker !== null) return { text: blocker, blocked: true };
  if (i.journey.path === "catalog" && i.catalog.preview === null) return { text: null, blocked: false };
  return { text: barSummary(i), blocked: false };
}

/** "Good to know before applying", the dives without photo in orange (they come first). */
export function knowLines(i: ReviewInput): { text: string; warn: boolean }[] {
  const p = i.catalog.preview;
  let empty = 0;
  if (i.journey.path === "catalog" && p) {
    const photos = new Map(p.dives.map((d) => [d.id, d.photos]));
    empty = tickedDives(i.journey, i.log.dives).filter((d) => photos.get(d.id) === 0).length;
  }
  return goodToKnow(i).map((text, n) => ({ text, warn: n < empty }));
}

/** What a click on "Apply" writes: the search shown and its counts (catalog path), or the selection of the moment. */
export type ApplyPlan = { path: "catalog"; period: Period; preview: CollectionsPreview } | { path: "selection" };

/** What a click on "Apply" writes now; null exactly when "Apply" is greyed (canApply), so a click always writes. */
export function applyPlan(i: ReviewInput, writing: boolean): ApplyPlan | null {
  if (!canApply(i, writing)) return null;
  if (i.journey.path !== "catalog") return { path: "selection" };
  const s = i.catalog.search;
  // canApply asked for both on the catalog path: this only tells the types.
  return s.state === "ready" && i.catalog.preview !== null ? { path: "catalog", period: s.period, preview: i.catalog.preview } : null;
}

/**
 * Lightroom's answer is no longer wanted: it came for an earlier run of the journey,
 * or once the step was gone. The latest run wins: such an answer changes nothing.
 */
export function staleAnswer(runAtClick: number, runNow: number, mounted: boolean): boolean {
  return !mounted || runAtClick !== runNow;
}

/** Per-photo values were sent: the dive details ticked, with the description of the selection. */
export function perPhotoSent(choice: TagChoice): boolean {
  return choice.options.dive && choice.search !== null;
}

/** Catalog path, written: the summary's figures, the photos outside the dives and the cameras of the search used. */
export function catalogApplied(
  result: CollectionsResult,
  preview: CollectionsPreview,
  period: Period,
  choices: CollectionChoices,
): Applied {
  return { path: "catalog", result, outside: preview.outside, cameras: period.cameras, ignored: choices.ignored };
}

/**
 * Selection path, written: whether per-photo values were sent (perPhotoSent), and then
 * the photos outside the dive are those of the counts of the description sent.
 */
export function selectionApplied(result: TagApplied, choice: TagChoice, counts: TagPreview | null): Applied {
  const perPhoto = perPhotoSent(choice);
  return {
    path: "selection",
    result,
    outside: perPhoto && counts ? counts.outside : [],
    outsideMore: perPhoto && counts ? counts.outsideMore : 0,
    perPhoto,
  };
}

/** What the journey reads again once Lightroom answered Apply. */
export interface ReadAgain {
  /** Catalog path: look for the photos again. */
  searchAgain: boolean;
  /** Selection path: describe the selection again. */
  describeAgain: boolean;
  /** Selection path: the offsets applied are the remembered ones now; none is set by hand any more. */
  clearTouched: boolean;
}

/**
 * As before. Catalog path: the photos are looked for again when the search is gone.
 * Selection path (`perPhoto`: the description was sent): written, described again for the
 * offsets now remembered; refused, described again when the selection moved on, or when
 * Lightroom answered while per-photo values were sent. `error` null: written.
 */
export function readAgain(path: Path, perPhoto: boolean, error: ApplyError | null): ReadAgain {
  const none: ReadAgain = { searchAgain: false, describeAgain: false, clearTouched: false };
  if (path === "catalog") return { ...none, searchAgain: error !== null && searchAgain(error) };
  if (error === null) return { ...none, describeAgain: perPhoto, clearTouched: true };
  return { ...none, describeAgain: describeAgain(error, perPhoto) };
}
