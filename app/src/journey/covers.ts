// "Covers the selection" mark of step 2 of the journey (spec F5): the dive
// whose times contain every selected photo. Only a help: the user still
// chooses. Times are wall-clock "YYYY-MM-DDTHH:MM:SS" strings, so they
// compare as text. Each camera's photos are moved by its clock offset (the
// one last applied to it, or the one shown) before they get here.

import type { Dive } from "../types";

export function coveringDive(dives: Dive[], from: string | null, to: string | null): Dive | null {
  if (!from || !to) return null;
  let best: Dive | null = null;
  for (const d of dives) {
    if (d.start <= from && d.end >= to && (!best || d.durationS < best.durationS)) best = d;
  }
  return best;
}
