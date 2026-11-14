// Cameras and clock offsets on the Create Collections screen (spec F5).

import type { Camera } from "./types";

/** Offsets are whole minutes, at most ±24 h. */
export const MAX_OFFSET_MIN = 24 * 60;
/** The slider covers every time zone, in quarter hours. */
export const SLIDER_RANGE_MIN = 14 * 60;
export const SLIDER_STEP_MIN = 15;

/** Surface photos margin: 5-minute steps up to 2 h. */
export const MARGIN_STEP_MIN = 5;
export const MAX_MARGIN_MIN = 120;

const MINUS = "−";

export function clampOffset(min: number): number {
  return Math.max(-MAX_OFFSET_MIN, Math.min(MAX_OFFSET_MIN, min));
}

export function clampMargin(min: number): number {
  return Math.max(0, Math.min(MAX_MARGIN_MIN, min));
}

/** Offset a camera starts with: the one last applied to it, else none. */
export function initialOffsets(cameras: Camera[]): Record<string, number> {
  return Object.fromEntries(cameras.map((c) => [c.key, c.remembered ?? 0]));
}

/**
 * Offsets once the cameras change (Tag Selection, a new selection): a
 * camera still there keeps the offset shown, a new one starts from the
 * offset last applied to it, else none.
 */
export function keepOffsets(shown: Record<string, number>, cameras: Camera[]): Record<string, number> {
  return Object.fromEntries(cameras.map((c) => [c.key, shown[c.key] ?? c.remembered ?? 0]));
}

/** Offset as typed in its field: "+4:01", "0:00", "−0:30". */
export function offsetField(min: number): string {
  const sign = min < 0 ? MINUS : min > 0 ? "+" : "";
  const a = Math.abs(min);
  return `${sign}${Math.floor(a / 60)}:${String(a % 60).padStart(2, "0")}`;
}

/**
 * Offset typed by the user, in minutes, or null when unreadable:
 * "+4:01", "4:1", "-0:30", "−1:15", "4h01", "4 h", "+4" (hours).
 */
export function parseOffset(text: string): number | null {
  const m = text
    .trim()
    .replace(/\u2212/g, "-")
    .match(/^([+-]?)\s*(\d{1,2})\s*(?:(?::|h)\s*(\d{1,2})?)?\s*(?:min)?$/i);
  if (!m) return null;
  const minutes = Number(m[3] ?? 0);
  if (minutes > 59) return null;
  const value = Number(m[2]) * 60 + minutes;
  if (value > MAX_OFFSET_MIN) return null;
  return m[1] === "-" ? -value : value;
}
