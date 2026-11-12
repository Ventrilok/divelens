// "Covers the selection" hint of Tag Selection (spec F8): the dive whose
// times contain every selected photo. Only a help: the user still chooses.
// Times are wall-clock "YYYY-MM-DDTHH:MM:SS" strings, so they compare as text.
// The camera's remembered clock offset will apply here once Create
// Collections stores it.

import type { Dive } from "./types";

export function coveringDive(dives: Dive[], from: string | null, to: string | null): Dive | null {
  if (!from || !to) return null;
  let best: Dive | null = null;
  for (const d of dives) {
    if (d.start <= from && d.end >= to && (!best || d.durationS < best.durationS)) best = d;
  }
  return best;
}
