// How dive data is shown. One function per kind of value, used everywhere,
// so the same data looks the same on every screen (spec: consistency rule).

import { localized } from "./i18n";
import type { Units } from "./types";

const FT_PER_M = 3.28084;

const oneDecimal = localized((l) => new Intl.NumberFormat(l, { maximumFractionDigits: 1, minimumFractionDigits: 1 }));
const upToOneDecimal = localized((l) => new Intl.NumberFormat(l, { maximumFractionDigits: 1 }));
const integer = localized((l) => new Intl.NumberFormat(l, { maximumFractionDigits: 0 }));

export const DASH = "—";

/** Depth in the display unit. */
export function depthValue(m: number, units: Units): number {
  return units.depth === "ft" ? m * FT_PER_M : m;
}

/** Display-unit depth back to metres (filters). */
export function depthToMetres(v: number, units: Units): number {
  return units.depth === "ft" ? v / FT_PER_M : v;
}

export function depthUnit(units: Units): string {
  return units.depth === "ft" ? "ft" : "m";
}

/** "29,2 m" / "96 ft". */
export function formatDepth(m: number | null, units: Units): string {
  if (m === null) return DASH;
  return units.depth === "ft"
    ? `${integer.format(depthValue(m, units))} ft`
    : `${oneDecimal.format(m)} m`;
}

function tempValue(c: number, units: Units): number {
  return units.temp === "f" ? (c * 9) / 5 + 32 : c;
}

function tempUnit(units: Units): string {
  return units.temp === "f" ? "°F" : "°C";
}

/** "27 °C" / "26,5 °C". */
export function formatTemp(c: number | null, units: Units): string {
  if (c === null) return DASH;
  return `${upToOneDecimal.format(tempValue(c, units))} ${tempUnit(units)}`;
}

/** "27 → 28 °C", or a single value when both are equal or one is missing. */
/** Coldest and warmest temperatures, each with its unit. When both read the
 *  same, only `same` is given. */
export function formatTemps(
  low: number | null,
  high: number | null,
  units: Units,
): { low: string | null; high: string | null; same: string | null } {
  const l = low === null ? null : formatTemp(low, units);
  const h = high === null ? null : formatTemp(high, units);
  if (l === null && h === null) return { low: null, high: null, same: DASH };
  if (l === h) return { low: null, high: null, same: l };
  return { low: l, high: h, same: null };
}

/** "35 min" (rounded to the minute). */
export function formatDuration(seconds: number): string {
  if (seconds <= 0) return DASH;
  return `${integer.format(Math.round(seconds / 60))} min`;
}

/** One coordinate as Lightroom shows it: degrees, minutes, seconds to the
 *  tenth (about 3 m), then the hemisphere. Never localised. */
function dms(v: number, positive: string, negative: string): string {
  const tenths = Math.round(Math.abs(v) * 36000);
  const deg = Math.floor(tenths / 36000);
  const min = Math.floor((tenths % 36000) / 600);
  const sec = ((tenths % 600) / 10).toFixed(1);
  return `${deg}°${min}'${sec}" ${v < 0 && tenths > 0 ? negative : positive}`;
}

/** `20°15'11.2" S 57°30'45.0" E`, as in Lightroom's GPS field. */
export function formatGps(lat: number | null, lon: number | null): string | null {
  if (lat === null || lon === null) return null;
  return `${dms(lat, "N", "S")} ${dms(lon, "E", "W")}`;
}

// --- Dates -----------------------------------------------------------------
// Dive times are wall-clock times without a time zone. They are parsed and
// formatted as UTC so the computer's own time zone never shifts them.

export function parseWallClock(s: string): Date {
  const [d, t = "00:00:00"] = s.split("T");
  const [y, mo, day] = d.split("-").map(Number);
  const [h, mi, sec] = t.split(":").map(Number);
  return new Date(Date.UTC(y, mo - 1, day, h, mi, sec || 0));
}

const fmt = (o: Intl.DateTimeFormatOptions) => localized((l) => new Intl.DateTimeFormat(l, { timeZone: "UTC", ...o }));
const fTime = fmt({ hour: "2-digit", minute: "2-digit" });
const fDayMonth = fmt({ day: "2-digit", month: "short" });
const fFullDate = fmt({ weekday: "short", day: "numeric", month: "short", year: "numeric" });
const fDate = fmt({ day: "numeric", month: "short", year: "numeric" });
const fMonth = fmt({ month: "long", year: "numeric" });

const cap = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);

/** List row: "31 déc. · 07:14". */
export function formatRowDate(start: string): string {
  const d = parseWallClock(start);
  return `${fDayMonth.format(d)} · ${fTime.format(d)}`;
}

/** Month group header: "DÉCEMBRE 2025" (upper-cased by CSS). */
export function formatMonth(start: string): string {
  return fMonth.format(parseWallClock(start));
}

/** Month grouping key: "2025-12". */
export function monthKey(start: string): string {
  return start.slice(0, 7);
}

/** Detail: "Mer. 31 déc. 2025 · 07:14 → 07:49". */
export function formatDiveSpan(start: string, end: string): string {
  const s = parseWallClock(start);
  const e = parseWallClock(end);
  const sameDay = start.slice(0, 10) === end.slice(0, 10);
  const endText = sameDay ? fTime.format(e) : `${fDate.format(e)}, ${fTime.format(e)}`;
  return `${cap(fFullDate.format(s))} · ${fTime.format(s)} → ${endText}`;
}

/** Photos' capture times: "30 déc. 2025, 07:31 → 08:12", or both dates when they differ. */
export function formatPhotoSpan(start: string, end: string): string {
  if (start.slice(0, 10) !== end.slice(0, 10)) return formatRange(start, end);
  const s = parseWallClock(start);
  return `${fDate.format(s)}, ${fTime.format(s)} → ${fTime.format(parseWallClock(end))}`;
}

/** Selection: "31 déc. 2025, 07:14 → 1 janv. 2026, 08:23". */
export function formatRange(start: string, end: string): string {
  const s = parseWallClock(start);
  const e = parseWallClock(end);
  return `${fDate.format(s)}, ${fTime.format(s)} → ${fDate.format(e)}, ${fTime.format(e)}`;
}

/** A day: "30 déc. 2025" ("YYYY-MM-DD" or a wall-clock date-time). */
export function formatDay(day: string): string {
  return fDate.format(parseWallClock(day.slice(0, 10)));
}

/** Dive times in a table row: "31 déc. · 07:14 → 07:49". */
export function formatRowSpan(start: string, end: string): string {
  const sameDay = start.slice(0, 10) === end.slice(0, 10);
  const e = parseWallClock(end);
  return `${formatRowDate(start)} → ${sameDay ? fTime.format(e) : `${fDayMonth.format(e)} · ${fTime.format(e)}`}`;
}

/** Text used by search: lower case, accents removed ("Égypte" → "egypte"). */
export function fold(s: string): string {
  return s.normalize("NFD").replace(/\p{Diacritic}/gu, "").toLowerCase();
}
