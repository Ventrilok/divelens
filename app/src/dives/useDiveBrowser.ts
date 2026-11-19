// The dive list's wiring, shared by Create Collections step 1 and Tag
// Selection: filters, sort, and the focused dive that follows the visible list.

import { useDeferredValue, useEffect, useMemo, useState } from "react";
import { applyFilters, NO_FILTERS, searchIndex, sortDives, type Filters, type Sort } from "./filters";
import type { Dive } from "../types";

export function useDiveBrowser(dives: Dive[], sort: Sort) {
  const [focusedId, setFocusedId] = useState<string | null>(null);
  const [filters, setFilters] = useState<Filters>(NO_FILTERS);

  // Typing stays fluid: the list catches up with the filters just after.
  const deferredFilters = useDeferredValue(filters);
  const index = useMemo(() => searchIndex(dives), [dives]);
  const shown = useMemo(
    () => sortDives(applyFilters(dives, deferredFilters, index), sort),
    [dives, deferredFilters, index, sort],
  );

  // Keep the detail on a visible dive.
  useEffect(() => {
    if (!shown.some((d) => d.id === focusedId)) setFocusedId(shown[0]?.id ?? null);
  }, [shown, focusedId]);

  const focused = dives.find((d) => d.id === focusedId) ?? null;
  return { filters, setFilters, shown, focusedId, setFocusedId, focused };
}
