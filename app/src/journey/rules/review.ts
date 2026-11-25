// Step 4 of the journey, "Review & Apply" (spec F5): what greys "Apply", the
// effect of each option on this run, "Good to know before applying" and the
// bar's summary. Pure, taken over from Create Collections and Tag Selection
// and extended to the options.

import { formatInt, t, tn } from "../../i18n";
import type { Search } from "../../collections/rules";
import { gpsRefused } from "../../tag/rules";
import { gpsInput } from "../../tag/sites";
import { depthUnit } from "../../ui/format";
import type { CollectionsPreview, DiveLog, LightroomStatus, OutsidePhoto, SelectionPhotos, TagPreview, Units } from "../../types";
import type { Journey } from "../state";
import { diveLabel, photoTime } from "../texts";
import { bySide, chosenDive, pickedSite, siteDetails, siteGives, siteSource, tickedDives, writeOptions } from "./shared";

/** What the catalog path's photo search shows (useCatalogPhotos). */
export interface CatalogReview {
  search: Search;
  preview: CollectionsPreview | null;
  /** The counts were asked for and failed: not the same as not arrived yet. */
  previewFailed: boolean;
}

/** What the selection path's reading shows (useSelectionPhotos). */
export interface SelectionReview {
  /** The chosen dive has a profile that can be read (or is being read). */
  needsValues: boolean;
  profile: { status: "loading" | "ok" | "error" };
  readFailure: string | null;
  /** The selection's description Apply may use: of the selection shown, and done. */
  current: SelectionPhotos | null;
  /** The counts of that description; null until they arrive. */
  freshCounts: TagPreview | null;
}

/** What step 4 knows, on either path. */
export interface ReviewInput {
  journey: Journey;
  lr: Pick<LightroomStatus, "connected" | "selected">;
  log: Pick<DiveLog, "dives" | "sites">;
  /** The collection set chosen in Settings is no longer in the catalog (spec F2). */
  parentMissing: boolean;
  /** That collection set, from the catalog's root. */
  parent: string[];
  catalog: CatalogReview;
  selection: SelectionReview;
}

/** What one run writes, counted. */
export interface ReviewCounts {
  /** Photos that get the dive details. */
  dive: number;
  /** Of which with per-photo values and a depth. */
  depth: number;
  /** Photos that get something of the site, and which part. */
  site: number;
  place: boolean;
  gps: boolean;
  /** Collections created or added to. */
  collections: number;
  /** Photos that get something written on them. */
  photos: number;
}

export function reviewCounts(i: ReviewInput): ReviewCounts {
  const j = i.journey;
  const o = writeOptions(j);
  const c: ReviewCounts = { dive: 0, depth: 0, site: 0, place: false, gps: false, collections: 0, photos: 0 };
  const other = siteDetails(o) ? pickedSite(j, i.log.sites) : null;
  const otherGives = other && siteGives(other, gpsInput(j.options.pick.lat, j.options.pick.lon));
  /** `photos` get the ticked parts of a site that gives them. */
  const addSite = (gives: { place: boolean; gps: boolean } | null, photos: number) => {
    const place = o.place && !!gives?.place;
    const gps = o.gps && !!gives?.gps;
    if (!place && !gps) return;
    c.site += photos;
    c.place ||= place;
    c.gps ||= gps;
  };

  if (j.path === "catalog") {
    const p = i.catalog.preview;
    const byId = new Map(p?.dives.map((d) => [d.id, d]) ?? []);
    const withPhotos = tickedDives(j, i.log.dives).filter((d) => (byId.get(d.id)?.photos ?? 0) > 0);
    const total = p?.total ?? 0;
    c.dive = o.dive ? total : 0;
    // Only the dives with a profile give the photos taken during them a depth.
    if (o.dive) for (const d of withPhotos) if (d.hasProfile) c.depth += byId.get(d.id)?.inDive ?? 0;
    if (siteDetails(o)) {
      if (siteSource(j, null) === "other") addSite(otherGives, total);
      else for (const d of withPhotos) addSite(siteGives(d.site), byId.get(d.id)?.photos ?? 0);
    }
    c.collections = o.collections ? withPhotos.length : 0;
  } else {
    const dive = chosenDive(j, i.log.dives);
    const n = i.lr.selected;
    c.dive = o.dive && dive ? n : 0;
    c.depth = o.dive && i.selection.needsValues ? (i.selection.freshCounts?.inDive ?? 0) : 0;
    if (siteDetails(o)) addSite(siteSource(j, dive) === "diveSite" && dive ? siteGives(dive.site) : otherGives, n);
    c.collections = o.collections && dive ? 1 : 0;
  }
  c.photos = Math.max(c.dive, c.site);
  return c;
}

/**
 * The position typed under "Other site" is refused: a site is picked and its GPS is not
 * valid. It matters only when the GPS is written: otherwise none is sent. What greys
 * "Apply" and what the site picker shows in red.
 */
export function gpsError(i: ReviewInput): boolean {
  const j = i.journey;
  const other = siteSource(j, j.path === "catalog" ? null : chosenDive(j, i.log.dives)) === "other";
  return gpsRefused({
    withPlace: writeOptions(j).gps,
    placeKind: other ? "site" : "diveSite",
    diveSiteUsable: !other,
    siteId: j.options.pick.siteId,
    gpsValid: gpsInput(j.options.pick.lat, j.options.pick.lon) !== undefined,
  });
}

/**
 * What greys "Apply", in the order the diver meets it; null when nothing does. The
 * missing collection set blocks only when collections are created; the reading of the
 * selection only when per-photo values are written. An empty write is never allowed:
 * it would leave an Undo step that changes nothing.
 */
export function reviewBlocker(i: ReviewInput): string | null {
  const j = i.journey;
  const o = writeOptions(j);
  const catalog = j.path === "catalog";
  if (!i.lr.connected) return t(catalog ? "coll.disconnected" : "tag.disconnected");
  if (catalog) {
    const s = i.catalog.search;
    if (s.state === "searching") return t("coll.searching");
    if (s.state === "failed") return s.message;
  } else {
    if (i.lr.selected === 0) return t("tag.selectInLr");
    const r = i.selection;
    const perPhoto = o.dive && r.needsValues;
    if (perPhoto && r.readFailure) return r.readFailure;
    if (perPhoto && r.profile.status === "loading") return t("detail.profileLoading");
    if (perPhoto && (!r.current || !r.freshCounts)) return t("tag.perPhoto.reading");
  }
  if (o.collections && i.parentMissing) return t("coll.parentMissing", { path: i.parent.join(" › ") });
  if (catalog && i.catalog.previewFailed) return t("coll.previewFailed");
  if (!o.dive && !siteDetails(o) && !o.collections) return t("journey.review.tickOne");

  const other = siteSource(j, catalog ? null : chosenDive(j, i.log.dives)) === "other";
  if (siteDetails(o) && other && pickedSite(j, i.log.sites) === null) return t("tag.chooseSite");
  if (gpsError(i)) return t("tag.badGps");

  const p = i.catalog.preview;
  const c = reviewCounts(i);
  if (catalog) {
    // Without the counts nothing can be said yet: Apply waits for them anyway.
    if (p && (p.total === 0 || (c.photos === 0 && c.collections === 0))) return t("coll.err.nothingToWrite");
  } else if (c.photos === 0 && c.collections === 0) {
    return t("tag.err.nothingChosen");
  }
  return null;
}

/**
 * "Apply" can be clicked: nothing blocks it and no write is in progress; on the catalog
 * path the photos must be found and counted too (without them there is no blocker yet,
 * but nothing to write either), as before. Counts kept from a search no longer shown
 * are not enough: Apply writes the search shown.
 */
export function canApply(i: ReviewInput, writing: boolean): boolean {
  if (writing || reviewBlocker(i) !== null) return false;
  return i.journey.path !== "catalog" || (i.catalog.search.state === "ready" && i.catalog.preview !== null);
}

/**
 * The number on "Apply to N photos", as before: every photo found in the dives (catalog
 * path, 0 until counted), or the photos selected in Lightroom (selection path).
 */
export function applyCount(i: ReviewInput): number {
  return i.journey.path === "catalog" ? (i.catalog.preview?.total ?? 0) : i.lr.selected;
}

/** The effect of each option on this run, shown on its right. */
export interface Effects {
  dive: string;
  altitude: string;
  site: string;
  collections: string;
}

export function effects(i: ReviewInput): Effects {
  const o = writeOptions(i.journey);
  const c = reviewCounts(i);
  const part = c.place && c.gps ? "journey.effect.placeAndGps" : c.place ? "journey.effect.place" : "journey.opt.gps";
  return {
    dive: o.dive ? tn("journey.effect.dive", c.dive, { depth: formatInt(c.depth) }) : t("journey.effect.nothing"),
    altitude: o.altitude ? tn("coll.photos", c.depth) : t("journey.effect.no"),
    site: c.site > 0 ? tn("journey.effect.site", c.site, { what: t(part) }) : t("journey.effect.nothing"),
    collections: c.collections > 0 ? tn("journey.effect.collections", c.collections) : t("journey.effect.none"),
  };
}

const times = (photos: OutsidePhoto[]) => photos.map((p) => photoTime(p.time)).join(", ");

/**
 * "Good to know before applying", without the units (unitsText): dives without photo
 * and photos outside the dives (catalog), photos without depth (selection).
 */
export function goodToKnow(i: ReviewInput): string[] {
  const j = i.journey;
  const lines: string[] = [];
  if (j.path === "catalog") {
    const p = i.catalog.preview;
    if (!p) return lines;
    const byId = new Map(p.dives.map((d) => [d.id, d]));
    for (const d of tickedDives(j, i.log.dives)) {
      if (byId.get(d.id)?.photos === 0) lines.push(t("journey.know.emptyDive", { dive: diveLabel(d) }));
    }
    if (p.outside > 0) lines.push(tn("journey.know.outside", p.outside));
    return lines;
  }
  const counts = i.selection.freshCounts;
  if (!writeOptions(j).dive || !i.selection.needsValues || !counts) return lines;
  const noDepth = counts.total - counts.inDive;
  if (noDepth <= 0) return lines;
  const sides = bySide(noDepth, counts.outside, counts.outsideMore);
  if (!sides) return [tn("journey.know.noDepth", noDepth)];
  if (sides.before.length > 0) lines.push(tn("journey.know.before", sides.before.length, { times: times(sides.before) }));
  if (sides.after.length > 0) lines.push(tn("journey.know.after", sides.after.length, { times: times(sides.after) }));
  return lines;
}

/** "Units: m, °C". */
export function unitsText(units: Units): string {
  return t("journey.know.units", { units: `${depthUnit(units)}, ${units.temp === "f" ? "°F" : "°C"}` });
}

/** The bar's summary: "N photos · N collections", "N photos · no collection" or "N collections · nothing is written on the photos". */
export function barSummary(i: ReviewInput): string {
  const c = reviewCounts(i);
  const photos = tn("coll.photos", c.photos);
  if (c.collections === 0) return `${photos} · ${t("journey.bar.noCollection")}`;
  const collections = tn("journey.effect.collections", c.collections);
  return c.photos === 0 ? `${collections} · ${t("journey.bar.nothingOnPhotos")}` : `${photos} · ${collections}`;
}
