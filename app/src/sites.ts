// Sites of the dive log arranged as country › location › site, for the
// "other site" picker of Tag Selection. Missing country or location form
// their own group, listed last.

import { localized } from "./i18n";
import type { Site } from "./types";

export interface LocationGroup {
  name: string | null;
  sites: Site[];
}

export interface CountryGroup {
  name: string | null;
  /** Number of sites in the country. */
  count: number;
  locations: LocationGroup[];
}

const collator = localized((l) => new Intl.Collator(l, { sensitivity: "base", numeric: true }));

function byName(a: string | null, b: string | null): number {
  if (a === null) return b === null ? 0 : 1;
  if (b === null) return -1;
  return collator.compare(a, b);
}

export function siteTree(sites: Site[]): CountryGroup[] {
  const countries = new Map<string | null, Map<string | null, Site[]>>();
  for (const s of sites) {
    if (s.id === null || s.name === null) continue;
    let locations = countries.get(s.country);
    if (!locations) countries.set(s.country, (locations = new Map()));
    const list = locations.get(s.location);
    if (list) list.push(s);
    else locations.set(s.location, [s]);
  }
  return [...countries.entries()]
    .map(([name, locations]) => {
      const groups = [...locations.entries()]
        .map(([loc, list]) => ({ name: loc, sites: list.sort((a, b) => byName(a.name, b.name)) }))
        .sort((a, b) => byName(a.name, b.name));
      return { name, count: groups.reduce((n, g) => n + g.sites.length, 0), locations: groups };
    })
    .sort((a, b) => byName(a.name, b.name));
}

/** Parses a coordinate typed by the user ("3,4878" or "3.4878"); null if empty, NaN if not a number. */
export function parseCoordinate(text: string): number | null {
  const s = text.trim().replace(",", ".");
  if (s === "") return null;
  return /^-?\d+(\.\d+)?$/.test(s) ? Number(s) : NaN;
}

/** Coordinates to write: both null (none typed), or both valid; undefined when invalid. */
export function gpsInput(latText: string, lonText: string): { lat: number | null; lon: number | null } | undefined {
  const lat = parseCoordinate(latText);
  const lon = parseCoordinate(lonText);
  if (lat === null && lon === null) return { lat: null, lon: null };
  if (lat === null || lon === null || Number.isNaN(lat) || Number.isNaN(lon)) return undefined;
  if (lat < -90 || lat > 90 || lon < -180 || lon > 180) return undefined;
  return { lat, lon };
}
