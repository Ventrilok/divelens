// What the journey's rules share (spec F5): what is written, where the site
// comes from, and the dives and site chosen as the log has them. Every rule
// reads what is written from writeOptions, never from the boxes, so that
// "no dive" (dive details and collections greyed) holds everywhere.

import { gpsInput } from "../sites";
import type { Dive, LocationChoice, OutsidePhoto, Site, WriteOptions } from "../../types";
import { noDive, type Journey, type Options } from "../state";

/** "Site details" is ticked while "Location" or "GPS" is. */
export function siteDetails(o: Pick<Options, "place" | "gps">): boolean {
  return o.place || o.gps;
}

/** What is written, as the engine takes it: no altitude without the dive details; only the site for no dive. */
export function writeOptions(j: Journey): WriteOptions {
  const o = j.options;
  if (noDive(j)) return { dive: false, altitude: false, place: o.place, gps: o.gps, collections: false };
  return { dive: o.dive, altitude: o.dive && o.altitude, place: o.place, gps: o.gps, collections: o.collections };
}

/**
 * Where the site written comes from on this run: each dive's own (catalog) or the
 * chosen dive's (selection, only when that dive has a site), else the site picked.
 */
export function siteSource(j: Journey, dive: Dive | null): "diveSite" | "other" {
  if (j.options.site === "other" || noDive(j)) return "other";
  if (j.path === "selection" && (dive === null || dive.site.id === null)) return "other";
  return "diveSite";
}

/**
 * What a site gives when written, as the engine builds it: place names (country, city,
 * location) and a whole GPS position, the position typed replacing the site's own.
 */
export function siteGives(site: Site, edited?: { lat: number | null; lon: number | null }): { place: boolean; gps: boolean } {
  const lat = edited?.lat ?? site.lat;
  const lon = edited?.lon ?? site.lon;
  return { place: site.country !== null || site.location !== null || site.name !== null, gps: lat !== null && lon !== null };
}

/** The place written is another site, one is picked, and the position typed is not valid: Apply is refused. */
export function gpsRefused(s: {
  withPlace: boolean;
  placeKind: "diveSite" | "site";
  diveSiteUsable: boolean;
  siteId: string | null;
  gpsValid: boolean;
}): boolean {
  const usesDiveSite = s.withPlace && s.placeKind === "diveSite" && s.diveSiteUsable;
  return s.withPlace && !usesDiveSite && s.siteId !== null && !s.gpsValid;
}

/** The site picked under "Other site", its GPS as typed; null until a site is picked. */
export function pickedLocation(o: Options): LocationChoice | null {
  if (o.pick.siteId === null) return null;
  const gps = gpsInput(o.pick.lat, o.pick.lon);
  return { kind: "site", siteId: o.pick.siteId, lat: gps?.lat ?? null, lon: gps?.lon ?? null };
}

/** The site picked under "Other site", among the log's sites. */
export function pickedSite(j: Journey, sites: Site[]): Site | null {
  const id = j.options.pick.siteId;
  return id === null ? null : (sites.find((s) => s.id === id) ?? null);
}

/** The dives ticked (catalog), as the log has them, by start time. */
export function tickedDives(j: Journey, dives: Dive[]): Dive[] {
  return dives.filter((d) => j.dives.has(d.id)).sort((a, b) => (a.start < b.start ? -1 : a.start > b.start ? 1 : 0));
}

/** The dive chosen (selection), as the log has it. */
export function chosenDive(j: Journey, dives: Dive[]): Dive | null {
  const pick = j.dive;
  return pick.kind === "dive" ? (dives.find((d) => d.id === pick.id) ?? null) : null;
}

/** Photos without depth by side of the dive, when every one of them is listed; otherwise null. */
export function bySide(
  noDepth: number,
  outside: OutsidePhoto[],
  more: number,
): { before: OutsidePhoto[]; after: OutsidePhoto[] } | null {
  if (more > 0 || outside.length !== noDepth) return null;
  return { before: outside.filter((p) => p.minutes < 0), after: outside.filter((p) => p.minutes > 0) };
}
