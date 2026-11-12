// Dive list filters and sorting (pure functions, no UI).

import { fold } from "./format";
import type { Dive, Sort } from "./types";

export interface Filters {
  query: string;
  /** "YYYY-MM-DD", inclusive. */
  from: string;
  to: string;
  /** Selected countries; "" stands for dives without a country. */
  countries: string[];
  /** Metres. */
  depthMin: number | null;
  depthMax: number | null;
  minRating: number | null;
  withProfile: boolean;
}

export const NO_FILTERS: Filters = {
  query: "",
  from: "",
  to: "",
  countries: [],
  depthMin: null,
  depthMax: null,
  minRating: null,
  withProfile: false,
};

export type { Sort } from "./types";

export function hasFilters(f: Filters): boolean {
  return (
    f.query.trim() !== "" ||
    f.from !== "" ||
    f.to !== "" ||
    f.countries.length > 0 ||
    f.depthMin !== null ||
    f.depthMax !== null ||
    f.minRating !== null ||
    f.withProfile
  );
}

/** Pre-computed search text per dive (number, site, location, country). */
export function searchIndex(dives: Dive[]): Map<string, string> {
  return new Map(
    dives.map((d) => [
      d.id,
      fold([d.number ?? "", d.site.name ?? "", d.site.location ?? "", d.site.country ?? ""].join(" ")),
    ]),
  );
}

export function applyFilters(dives: Dive[], f: Filters, index: Map<string, string>): Dive[] {
  const words = fold(f.query).split(/\s+/).filter(Boolean);
  const countries = new Set(f.countries);
  return dives.filter((d) => {
    if (words.length) {
      const text = index.get(d.id) ?? "";
      if (!words.every((w) => text.includes(w))) return false;
    }
    const day = d.start.slice(0, 10);
    if (f.from && day < f.from) return false;
    if (f.to && day > f.to) return false;
    if (countries.size && !countries.has(d.site.country ?? "")) return false;
    if (f.depthMin !== null && (d.maxDepthM === null || d.maxDepthM < f.depthMin)) return false;
    if (f.depthMax !== null && (d.maxDepthM === null || d.maxDepthM > f.depthMax)) return false;
    if (f.minRating !== null && (d.rating ?? 0) < f.minRating) return false;
    if (f.withProfile && !d.hasProfile) return false;
    return true;
  });
}

export function sortDives(dives: Dive[], sort: Sort): Dive[] {
  const out = [...dives];
  const byDate = (a: Dive, b: Dive) => a.start.localeCompare(b.start) || (a.number ?? 0) - (b.number ?? 0);
  if (sort === "oldest") out.sort(byDate);
  else if (sort === "recent") out.sort((a, b) => byDate(b, a));
  else out.sort((a, b) => (b.maxDepthM ?? -1) - (a.maxDepthM ?? -1) || byDate(b, a));
  return out;
}

/** Countries with their dive count, most frequent first ("" = no country). */
export function countryCounts(dives: Dive[]): [string, number][] {
  const m = new Map<string, number>();
  for (const d of dives) {
    const c = d.site.country ?? "";
    m.set(c, (m.get(c) ?? 0) + 1);
  }
  return [...m].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]));
}
