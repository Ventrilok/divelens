// The selection path's photo reading (spec F5, steps 2 to 4), moved from Tag
// Selection: the photos selected in Lightroom, described photo by photo
// (capture time, camera), again whenever the selection changes; one clock
// offset per camera; and the live counts for the dive chosen. A camera still
// there keeps the offset shown, a new one starts from the offset last applied
// to it. The last description and counts stay until the new ones arrive.
//
// The selection is described for a dive whose profile can be read, as before,
// or as soon as `active` (the journey's selection path from step 2 on, so that
// "Covers the selection" can count; choice no. 3). A new `run` throws away
// what was read and set.

import { useEffect, useMemo, useRef, useState } from "react";
import { findSelectionPhotos, suggestTagOffsets, tagPreview } from "../api";
import { clampOffset, keepOffsets } from "../apply/cameras";
import { searchFailure } from "../apply/failure";
import { logRef, nextSearchNumber } from "../diveLog";
import { useProfile } from "../dives/DiveDetail";
import { t } from "../i18n";
import type { ApplyError, Dive, DiveLog, LightroomStatus, SelectionPhotos, TagPreview } from "../types";
import { coveringDive } from "./covers";
import type { SelectionReview } from "./rules";
import {
  camerasToSuggest,
  currentDescription,
  freshCountsOf,
  notesOfCameras,
  onlyCameras,
  readingAfterFailure,
  readsSelection,
  suggestionNotes,
  withSuggestions,
  type Counts,
  type Described,
} from "./readings";

export interface SelectionReading extends SelectionReview {
  /** The last description of the selection, kept until the next one arrives. */
  described: Described | null;
  /** The description Apply may use: of the selection shown, and done. */
  current: SelectionPhotos | null;
  reading: boolean;
  readFailure: string | null;
  /** The last counts, shown until newer ones arrive. */
  counts: TagPreview | null;
  /** The counts of `current`; null until they arrive. */
  freshCounts: TagPreview | null;
  offsets: Record<string, number>;
  /** Cameras whose offset the diver set by hand. */
  touched: Set<string>;
  /** Said under a camera's card ("no offset found"). */
  notes: Record<string, string>;
  profile: ReturnType<typeof useProfile>;
  /** The dive has a profile that can be read (or is being read): per-photo values. */
  needsValues: boolean;
  /** The dive has a profile that cannot be read. */
  unreadable: boolean;
  setOffset: (key: string, minutes: number) => void;
  suggest: (key: string) => void;
  /** Every camera whose offset was not set by hand. */
  suggestAll: () => void;
  /** Describes the selection again (failed reading, after Apply). */
  describeAgain: () => void;
  /** After Apply: the offsets applied are now the remembered ones, none is set by hand any more. */
  clearTouched: () => void;
  /** The dive whose times cover the selection, pointed out, never chosen. */
  covering: Dive | null;
}

export function useSelectionPhotos(
  log: DiveLog,
  lr: Pick<LightroomStatus, "connected" | "selected" | "selectionVersion" | "coverFrom" | "coverTo">,
  dive: Dive | null,
  active: boolean,
  run: number,
): SelectionReading {
  const span = dive && { start: dive.start, end: dive.end };
  const spanKey = span ? `${span.start}|${span.end}` : "";

  // The dive the notes and suggestions belong to: the last one chosen.
  const lastDive = useRef<string | null>(null);
  if (dive) lastDive.current = dive.id;
  const forDive = lastDive.current;
  const runNow = useRef(run);
  runNow.current = run;

  // Per-photo values: for a dive with a profile, unless it cannot be read
  // (the dive data are then written alone, as before per-photo values).
  const profile = useProfile(logRef(log.source), dive);
  const hasProfile = dive !== null && dive.hasProfile;
  const unreadable = hasProfile && profile.status === "error";
  const needsValues = hasProfile && !unreadable;

  const [described, setDescribed] = useState<Described | null>(null);
  const [reading, setReading] = useState(false);
  const [readFailure, setReadFailure] = useState<string | null>(null);
  const [again, setAgain] = useState(0);
  // The selection version already described again after a lost description: at most once each.
  const retried = useRef<number | null>(null);
  const [offsets, setOffsets] = useState<Record<string, number>>({});
  const [touched, setTouched] = useState<Set<string>>(new Set());
  const [notes, setNotes] = useState<Record<string, string>>({});
  const [counts, setCounts] = useState<Counts | null>(null);
  const asked = useRef(0);

  // A new run: what was read and set is thrown away, and counts asked before
  // are no longer wanted. Before the reading below, which starts again.
  const runSeen = useRef(run);
  useEffect(() => {
    if (runSeen.current === run) return;
    runSeen.current = run;
    asked.current++;
    retried.current = null;
    setDescribed(null);
    setCounts(null);
    setReading(false);
    setReadFailure(null);
    setOffsets({});
    setTouched(new Set());
    setNotes({});
  }, [run]);

  const wanted = readsSelection(active, needsValues, lr);
  useEffect(() => {
    if (!wanted) {
      setDescribed(null);
      setCounts(null);
      setReading(false);
      setReadFailure(null);
      return;
    }
    let live = true;
    const version = lr.selectionVersion;
    setReading(true);
    setReadFailure(null);
    findSelectionPhotos(logRef(log.source), nextSearchNumber())
      .then((photos) => {
        if (!live) return;
        setOffsets((o) => keepOffsets(o, photos.cameras));
        setTouched((s) => onlyCameras(s, photos.cameras));
        setNotes((n) => notesOfCameras(n, photos.cameras));
        setDescribed({ photos, version });
        setReading(false);
      })
      .catch((e: ApplyError) => {
        const after = readingAfterFailure(e, live, retried.current === version);
        if (!after) return;
        setReadFailure(after.message !== undefined ? after.message : searchFailure(e, "tag"));
        setReading(after.reading);
        if (after.describeAgain) {
          retried.current = version;
          setAgain((n) => n + 1);
        }
      });
    return () => {
      live = false;
    };
  }, [wanted, lr.selectionVersion, again, run]);
  const current = currentDescription(described, lr.selectionVersion, reading, readFailure);

  // Live counts for the dive chosen and the offsets shown: the latest answer
  // wins. Refused once a newer description replaced this one: the last
  // counts stay until the newer ones arrive.
  useEffect(() => {
    if (!described) return;
    const n = ++asked.current;
    const search = described.photos.search;
    tagPreview(search, span, offsets)
      .then((preview) => n === asked.current && setCounts({ search, preview }))
      .catch(() => {});
  }, [described, offsets, spanKey]);
  const shownCounts = counts?.preview ?? null;

  // A note on a camera (no offset found) belongs to the dive it was asked for.
  useEffect(() => setNotes({}), [forDive]);

  // The dive covering the selection follows the offsets shown once the
  // selection is described; before, the offsets last applied.
  const coverFrom = described && shownCounts ? shownCounts.coverFrom : lr.coverFrom;
  const coverTo = described && shownCounts ? shownCounts.coverTo : lr.coverTo;
  const covering = useMemo(() => coveringDive(log.dives, coverFrom, coverTo), [log.dives, coverFrom, coverTo]);

  const setOffset = (key: string, min: number) => {
    setOffsets((o) => ({ ...o, [key]: clampOffset(min) }));
    setTouched((s) => new Set(s).add(key));
    setNotes((n) => ({ ...n, [key]: "" }));
  };

  const suggestFor = (keys: string[]) => {
    if (keys.length === 0 || !current || !span) return;
    const askedFor = forDive;
    const askedIn = run;
    suggestTagOffsets(current.search, span, keys)
      .then((found) => {
        // Another dive chosen meanwhile, or a new run: this suggestion was for the previous one.
        if (askedFor !== lastDive.current || askedIn !== runNow.current) return;
        setOffsets((o) => withSuggestions(o, keys, found));
        setNotes((n) => suggestionNotes(n, keys, found, t("tag.perPhoto.noSuggestion")));
      })
      // Refused once a newer description replaced this one: nothing to suggest.
      .catch(() => {});
  };

  return {
    described,
    current,
    reading,
    readFailure,
    counts: shownCounts,
    freshCounts: freshCountsOf(counts, current),
    offsets,
    touched,
    notes,
    profile,
    needsValues,
    unreadable,
    setOffset,
    suggest: (key) => suggestFor([key]),
    suggestAll: () => suggestFor(camerasToSuggest(described?.photos.cameras ?? [], touched)),
    describeAgain: () => setAgain((n) => n + 1),
    clearTouched: () => setTouched(new Set()),
    covering,
  };
}
