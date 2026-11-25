// The catalog path's photo reading (spec F5, steps 3 and 4), moved from Create
// Collections: the photos of the period of the dives ticked, found in the
// whole catalog, the cameras among them with one clock offset each, the
// cameras ignored, the surface margin, and the live counts per dive. The
// latest search and the latest counts win. When Lightroom comes back, the
// photos are found again and what the diver set is kept.
//
// It looks for the photos only while `diveIds` is not empty: the journey
// passes the dives ticked from step 3 on and none before, or every tick at
// step 2 would start a search. Other dives in the same run reset the offsets,
// ignored cameras and margin; a new `run` throws away what was found and set,
// except the surface margin when the run goes on with dives ticked ("handle
// separately", as in 2.1.1), so that its first search keeps it.

import { useEffect, useMemo, useRef, useState } from "react";
import { collectionsPreview, findCollectionPhotos, suggestOffsets } from "../api";
import { clampMargin, clampOffset } from "../apply/cameras";
import { searchFailure } from "../apply/failure";
import { logRef, nextSearchNumber } from "../diveLog";
import { t } from "../i18n";
import type {
  ApplyError,
  Camera,
  CollectionChoices,
  CollectionsPreview,
  DiveLog,
  LightroomStatus,
  Period,
} from "../types";
import type { CatalogReview } from "./rules";
import {
  camerasToSuggest,
  diveKey as searchedDives,
  keysAfterSearch,
  marginAfterRun,
  marginAfterSearch,
  offsetsAfterSearch,
  suggestionNotes,
  withSuggestions,
  type Search,
} from "./readings";

export interface CatalogReading extends CatalogReview {
  search: Search;
  /** The photos found; null until they are. */
  period: Period | null;
  /** Their cameras, sorted by name. */
  cameras: Camera[];
  offsets: Record<string, number>;
  ignored: Set<string>;
  /** Cameras whose offset the diver set by hand. */
  touched: Set<string>;
  /** Said under a camera's card ("no offset found"). */
  notes: Record<string, string>;
  /** Surface margin, in minutes. */
  margin: number;
  preview: CollectionsPreview | null;
  previewFailed: boolean;
  setOffset: (key: string, minutes: number) => void;
  setIgnored: (key: string, on: boolean) => void;
  setMargin: (minutes: number) => void;
  suggest: (key: string) => void;
  /** Every camera neither ignored nor set by hand. */
  suggestAll: () => void;
  /** Looks for the photos again (failed search, or Apply refused for a lost search). */
  searchAgain: () => void;
  /** What the counts and Apply take. */
  choices: CollectionChoices;
}

export function useCatalogPhotos(
  log: DiveLog,
  lr: Pick<LightroomStatus, "connected">,
  diveIds: string[],
  run: number,
): CatalogReading {
  const idsKey = diveIds.join(",");
  const diveKey = useMemo(() => searchedDives(log.dives, diveIds), [log.dives, idsKey]);

  const [search, setSearch] = useState<Search>({ state: "idle" });
  const [again, setAgain] = useState(0);
  const [offsets, setOffsets] = useState<Record<string, number>>({});
  const [ignored, setIgnoredSet] = useState<Set<string>>(new Set());
  const [touched, setTouched] = useState<Set<string>>(new Set());
  const [notes, setNotes] = useState<Record<string, string>>({});
  const [margin, setMarginMin] = useState(0);
  const [preview, setPreview] = useState<CollectionsPreview | null>(null);
  const [previewFailed, setPreviewFailed] = useState(false);
  const searchedFor = useRef<string | null>(null);
  const asked = useRef(0);

  // A new run: what was found and set is thrown away, and a count asked before
  // is no longer wanted. Before the search below, which starts again.
  const runSeen = useRef(run);
  useEffect(() => {
    if (runSeen.current === run) return;
    runSeen.current = run;
    searchedFor.current = null;
    asked.current++;
    setOffsets({});
    setIgnoredSet(new Set());
    setTouched(new Set());
    setNotes({});
    setPreview(null);
    setPreviewFailed(false);
    setMarginMin((m) => marginAfterRun(m, diveIds));
  }, [run]);

  // Photos of the period, found again when the dives change or Lightroom
  // comes back (its photo ids only hold for one link session).
  useEffect(() => {
    if (!diveKey || !lr.connected) {
      setSearch({ state: "idle" });
      return;
    }
    let live = true;
    setSearch({ state: "searching" });
    // Counts still asked for the search before this one are no longer wanted.
    asked.current++;
    setPreview(null);
    setPreviewFailed(false);
    findCollectionPhotos(logRef(log.source), nextSearchNumber(), diveKey.split(","))
      .then((period) => {
        if (!live) return;
        // Same dives found again (link restarted): the user's settings stay.
        const keep = searchedFor.current === diveKey;
        const firstOfRun = searchedFor.current === null;
        searchedFor.current = diveKey;
        setOffsets((o) => offsetsAfterSearch(o, period.cameras, keep));
        setIgnoredSet((s) => keysAfterSearch(s, period.cameras, keep));
        setTouched((s) => keysAfterSearch(s, period.cameras, keep));
        if (!keep) setNotes({});
        setMarginMin((m) => marginAfterSearch(m, keep, firstOfRun));
        setSearch({ state: "ready", period });
      })
      .catch((e: ApplyError) => live && setSearch({ state: "failed", message: searchFailure(e, "collections") }));
    return () => {
      live = false;
    };
  }, [diveKey, lr.connected, again, run]);

  const choices: CollectionChoices = useMemo(
    () => ({ offsets, ignored: [...ignored], marginMin: margin }),
    [offsets, ignored, margin],
  );

  // Live counts: the latest answer wins.
  useEffect(() => {
    if (search.state !== "ready") return;
    const n = ++asked.current;
    setPreviewFailed(false);
    collectionsPreview(search.period.search, choices)
      .then((p) => n === asked.current && setPreview(p))
      .catch(() => {
        if (n !== asked.current) return;
        setPreview(null);
        setPreviewFailed(true);
      });
  }, [choices, search]);

  const period = search.state === "ready" ? search.period : null;
  const cameras = period?.cameras ?? [];

  const setOffset = (key: string, min: number) => {
    setOffsets((o) => ({ ...o, [key]: clampOffset(min) }));
    setTouched((s) => new Set(s).add(key));
    setNotes((n) => ({ ...n, [key]: "" }));
  };

  const setIgnored = (key: string, on: boolean) => {
    setIgnoredSet((s) => {
      const next = new Set(s);
      if (on) next.add(key);
      else next.delete(key);
      return next;
    });
  };

  const suggestFor = (keys: string[]) => {
    if (keys.length === 0 || !period) return;
    suggestOffsets(period.search, keys)
      .then((found) => {
        setOffsets((o) => withSuggestions(o, keys, found));
        setNotes((n) => suggestionNotes(n, keys, found, t("coll.noSuggestion")));
      })
      // Refused once a newer search replaced this one: nothing to suggest.
      .catch(() => {});
  };

  return {
    search,
    period,
    cameras,
    offsets,
    ignored,
    touched,
    notes,
    margin,
    preview,
    previewFailed,
    setOffset,
    setIgnored,
    setMargin: (min) => setMarginMin(clampMargin(min)),
    suggest: (key) => suggestFor([key]),
    // "Suggest for all" leaves ignored cameras and hand-set offsets alone.
    suggestAll: () => suggestFor(camerasToSuggest(cameras, touched, ignored)),
    searchAgain: () => setAgain((n) => n + 1),
    choices,
  };
}
