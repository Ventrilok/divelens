// Cameras and clock offsets on the Create Collections screen (spec F5).

import type { Camera } from "./types";

/** Offsets are whole minutes, at most ±24 h. */
export const MAX_OFFSET_MIN = 24 * 60;
/** Surface photos margin: 5-minute steps up to 2 h. */
export const MARGIN_STEP_MIN = 5;
export const MAX_MARGIN_MIN = 120;

const MINUS = "−";

/** "0 min", "+45 min", "−30 min", "+4 h 00", "−1 h 15". */
export function formatOffset(min: number): string {
  if (min === 0) return "0 min";
  const sign = min < 0 ? MINUS : "+";
  const a = Math.abs(min);
  if (a < 60) return `${sign}${a} min`;
  return `${sign}${Math.floor(a / 60)} h ${String(a % 60).padStart(2, "0")}`;
}

export function clampOffset(min: number): number {
  return Math.max(-MAX_OFFSET_MIN, Math.min(MAX_OFFSET_MIN, min));
}

export function clampMargin(min: number): number {
  return Math.max(0, Math.min(MAX_MARGIN_MIN, min));
}

/** End of the serial number, enough to tell two bodies apart: "…1325". */
export function serialTail(serial: string | null): string | null {
  if (!serial) return null;
  return serial.length > 4 ? `…${serial.slice(-4)}` : serial;
}

/** Offset a camera starts with: the one last applied to it, else none. */
export function initialOffsets(cameras: Camera[]): Record<string, number> {
  return Object.fromEntries(cameras.map((c) => [c.key, c.remembered ?? 0]));
}
