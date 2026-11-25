// The collection set chosen in Settings must still be in the catalog (spec F2):
// "Create collections" never falls back on the catalog's root. Looked up from
// step 2 on, so that step 4 opens with the answer; again on each arrival at
// step 3 from step 2 (`round`), so that a set deleted in Lightroom meanwhile
// is noticed before Apply; and when Lightroom connects, another catalog is
// open, the place changes in Settings, or the journey comes back from step 1.
// The last lookup started is the one whose answer counts.

import { useEffect, useState } from "react";
import { collectionSets } from "../api";
import { setExists } from "../collections/collectionPlace";
import type { LightroomStatus } from "../types";

/**
 * Whether the collection set `parent` (from the catalog's root) is missing; false while not
 * looked up. Each new `round` looks it up again.
 */
export function useParentMissing(
  lr: Pick<LightroomStatus, "connected" | "catalog">,
  parent: string[],
  active: boolean,
  round: number,
): boolean {
  const parentKey = JSON.stringify(parent);
  const [missing, setMissing] = useState(false);
  useEffect(() => {
    setMissing(false);
    if (!active || !lr.connected || parent.length === 0) return;
    let live = true;
    collectionSets()
      .then((sets) => live && setMissing(!setExists(sets, parent)))
      .catch(() => {});
    return () => {
      live = false;
    };
  }, [active, lr.connected, lr.catalog, parentKey, round]);
  return missing;
}
