// Tag Selection screen (spec F8): writes a dive's data and/or a location on
// the photos selected in Lightroom, whatever their time. The selection is
// made in Lightroom and followed live; Lightroom resolves it again at the
// moment of Apply, in one Undo step.
//
// When the chosen dive has a profile, the photos taken during it also get
// their per-photo values (F6): the selection is described photo by photo,
// each camera gets a clock offset as in Create Collections, and Apply
// writes only if the selection is still the one described.

import { useDeferredValue, useEffect, useMemo, useRef, useState } from "react";
import {
  findSelectionPhotos,
  onApplyProgress,
  onSelectionPreview,
  selectionPreview,
  suggestTagOffsets,
  tagPreview,
  tagSelection,
} from "../api";
import { clampOffset, keepOffsets } from "../cameras";
import { coveringDive } from "../covers";
import { logRef, nextSearchNumber } from "../diveLog";
import { sourceName } from "../sourceText";
import { CameraCard } from "../components/CameraCard";
import { DiveDetail, useProfile } from "../components/DiveDetail";
import { LogNotice } from "../components/LogNotice";
import { DiveList } from "../components/DiveList";
import { FilterBar } from "../components/FilterBar";
import { NO_PICK, pickAround, SitePicker, type SitePick } from "../components/SitePicker";
import { formatGps, formatPhotoSpan } from "../format";
import { applyFilters, hasFilters, NO_FILTERS, searchIndex, sortDives, type Filters, type Sort } from "../filters";
import { formatInt, t, tn } from "../i18n";
import { gpsInput } from "../sites";
import type {
  ApplyOutcome,
  CollectionsError,
  Dive,
  DiveLog,
  LightroomStatus,
  SelectionPhotos,
  SelectionPreview,
  Site,
  TagChoice,
  TagError,
  TagPreview,
} from "../types";

const THUMBNAILS = 6;

type Run =
  | { state: "idle" }
  | { state: "writing"; done: number; total: number }
  | { state: "done"; count: number }
  | { state: "failed"; message: string };

/** The selection described photo by photo, for the per-photo values; `version`: the selection's. */
interface Described {
  photos: SelectionPhotos;
  version: number;
}

/** Live counts, and the description they were computed for. */
interface Counts {
  search: number;
  preview: TagPreview;
}

function siteLine(site: Site): string {
  return [site.name, site.location, site.country].filter(Boolean).join(", ");
}

/** Why the selection could not be described; null: a newer description replaced this one. */
function describeFailure(e: CollectionsError): string | null {
  switch (e.error) {
    case "noPhotos":
    case "logChanged": // the app reads the log again and starts the screen again
      return null;
    case "notConnected":
    case "linkLost":
      return t("tag.disconnected");
    case "lightroom":
      return t("tag.perPhoto.failed", { detail: e.detail ?? e.reason });
    case "source":
      return t("tag.perPhoto.failed", { detail: e.detail.detail });
    default:
      return t("tag.perPhoto.failed", { detail: e.error });
  }
}

/** Nothing was written because the selection is no longer the one described: it is described again. */
function selectionChanged(e: ApplyOutcome | TagError): boolean {
  if ("outcome" in e) return e.outcome === "failed" && (e.reason === "selectionChanged" || e.reason === "staleSelection");
  return e.error === "noPhotos";
}

function failure(e: ApplyOutcome | TagError): string {
  if (selectionChanged(e)) return t("tag.err.selectionChanged");
  const key = "outcome" in e ? (e.outcome === "failed" ? e.reason : "") : e.error;
  switch (key) {
    case "noSelection":
      return t("tag.err.noSelection");
    case "busy":
      return t("tag.err.busy");
    case "gpsNotWritten": {
      // Written but for the GPS and altitude: Undo, then apply again.
      const detail = "outcome" in e && e.outcome === "failed" ? e.detail : null;
      return detail ? t("tag.err.gpsNotWrittenDetail", { detail }) : t("tag.err.gpsNotWritten");
    }
    case "linkLost":
      return t("tag.err.linkLost");
    case "notConnected":
      return t("tag.err.notConnected");
    default: {
      const detail = "outcome" in e ? (e.outcome === "failed" ? (e.detail ?? e.reason) : "") : JSON.stringify(e);
      return t("tag.err.other", { detail });
    }
  }
}

export function TagScreen({
  log,
  lr,
  sort,
  notice,
  onDismissNotice,
  onCancel,
}: {
  log: DiveLog;
  lr: LightroomStatus;
  /** Same order as the dive list of Create Collections (one list everywhere). */
  sort: Sort;
  /** Why the screen started again (the dive log changed), shown once. */
  notice: string | null;
  onDismissNotice: () => void;
  onCancel: () => void;
}) {
  const units = log.units;
  const { dives, sites, capabilities } = log;

  const [withDive, setWithDive] = useState(true);
  const [withPlace, setWithPlace] = useState(true);
  const [placeKind, setPlaceKind] = useState<"diveSite" | "site">("diveSite");
  const [pick, setPick] = useState<SitePick>(NO_PICK);
  const [chosenId, setChosenId] = useState<string | null>(null);
  const [focusedId, setFocusedId] = useState<string | null>(null);
  const [filters, setFilters] = useState<Filters>(NO_FILTERS);
  const [run, setRun] = useState<Run>({ state: "idle" });

  const deferredFilters = useDeferredValue(filters);
  const index = useMemo(() => searchIndex(dives), [dives]);
  const shown = useMemo(() => sortDives(applyFilters(dives, deferredFilters, index), sort), [dives, deferredFilters, index, sort]);

  useEffect(() => {
    if (!shown.some((d) => d.id === focusedId)) setFocusedId(shown[0]?.id ?? null);
  }, [shown, focusedId]);

  useEffect(() => {
    const stop = onApplyProgress((done, total) =>
      setRun((r) => (r.state === "writing" ? { state: "writing", done, total } : r)),
    );
    return () => {
      stop.then((f) => f());
    };
  }, []);

  // Thumbnails of the current selection, as they arrive.
  const [preview, setPreview] = useState<SelectionPreview>({ version: 0, thumbnails: [] });
  useEffect(() => {
    selectionPreview().then(setPreview);
    const stop = onSelectionPreview(setPreview);
    return () => {
      stop.then((f) => f());
    };
  }, []);
  const thumbnails = preview.version === lr.selectionVersion ? preview.thumbnails : [];

  const chosen: Dive | null = withDive ? (dives.find((d) => d.id === chosenId) ?? null) : null;
  const chosenRef = useRef(chosenId);
  chosenRef.current = chosenId;
  const span = chosen && { start: chosen.start, end: chosen.end };
  const spanKey = span ? `${span.start}|${span.end}` : "";

  // Per-photo values: for a dive with a profile, unless it cannot be read
  // (the dive data are then written alone, as before per-photo values).
  const profile = useProfile(logRef(log.source), chosen);
  const hasProfile = chosen !== null && chosen.hasProfile;
  const unreadable = hasProfile && profile.status === "error";
  const needsValues = hasProfile && !unreadable;
  const profileLoading = needsValues && profile.status === "loading";

  // The selection photo by photo, described for such a dive, again whenever
  // the selection changes, and its cameras: each keeps the offset shown, a
  // new one starts from the offset last applied to it (spec F8). The last
  // description and counts stay on screen until the new ones arrive.
  const [described, setDescribed] = useState<Described | null>(null);
  const [reading, setReading] = useState(false);
  const [readFailure, setReadFailure] = useState<string | null>(null);
  const [again, setAgain] = useState(0);
  const [offsets, setOffsets] = useState<Record<string, number>>({});
  const [touched, setTouched] = useState<Set<string>>(new Set());
  const [notes, setNotes] = useState<Record<string, string>>({});
  const [counts, setCounts] = useState<Counts | null>(null);
  const wanted = needsValues && lr.connected && lr.selected > 0;
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
        const keys = new Set(photos.cameras.map((c) => c.key));
        setOffsets((o) => keepOffsets(o, photos.cameras));
        setTouched((s) => new Set([...s].filter((k) => keys.has(k))));
        setNotes((n) => Object.fromEntries(Object.entries(n).filter(([k]) => keys.has(k))));
        setDescribed({ photos, version });
        setReading(false);
      })
      .catch((e: CollectionsError) => {
        const message = live ? describeFailure(e) : null;
        if (!message) return;
        setReadFailure(message);
        setReading(false);
      });
    return () => {
      live = false;
    };
  }, [wanted, lr.selectionVersion, again]);
  // The description Apply may use: of the selection shown, and done.
  const current =
    described && described.version === lr.selectionVersion && !reading && !readFailure ? described.photos : null;

  // Live counts for the dive chosen and the offsets shown: the latest answer
  // wins. Refused once a newer description replaced this one: the last
  // counts stay until the newer ones arrive.
  const asked = useRef(0);
  useEffect(() => {
    if (!described) return;
    const n = ++asked.current;
    const search = described.photos.search;
    tagPreview(search, span, offsets)
      .then((preview) => n === asked.current && setCounts({ search, preview }))
      .catch(() => {});
  }, [described, offsets, spanKey]);
  const shownCounts = counts?.preview ?? null;
  const freshCounts = counts && current && counts.search === current.search ? counts.preview : null;

  // A note on a camera (no offset found) belongs to the dive it was asked for.
  useEffect(() => setNotes({}), [chosenId]);

  // The dive covering the selection is pointed out, never chosen for the
  // user. It follows the offsets shown once the selection is described for
  // a dive with a profile; before, the offsets last applied. It is shown in
  // the detail card only until a dive is chosen: then the detail stays.
  const coverFrom = described && shownCounts ? shownCounts.coverFrom : lr.coverFrom;
  const coverTo = described && shownCounts ? shownCounts.coverTo : lr.coverTo;
  const covering = useMemo(() => coveringDive(dives, coverFrom, coverTo), [dives, coverFrom, coverTo]);
  useEffect(() => {
    if (covering && chosenId === null) setFocusedId(covering.id);
  }, [covering]);

  const setOffset = (key: string, min: number) => {
    setOffsets((o) => ({ ...o, [key]: clampOffset(min) }));
    setTouched((s) => new Set(s).add(key));
    setNotes((n) => ({ ...n, [key]: "" }));
  };
  const suggest = (key: string) => {
    if (!current || !span) return;
    const forDive = chosenId;
    suggestTagOffsets(current.search, span, [key])
      .then((found) => {
        // Another dive chosen meanwhile: this suggestion was for the previous one.
        if (forDive !== chosenRef.current) return;
        const value = found[key];
        if (value !== null && value !== undefined) setOffsets((o) => ({ ...o, [key]: value }));
        setNotes((n) => ({ ...n, [key]: value === null ? t("tag.perPhoto.noSuggestion") : "" }));
      })
      // Refused once a newer description replaced this one: nothing to suggest.
      .catch(() => {});
  };

  const focused = dives.find((d) => d.id === focusedId) ?? null;
  const diveSiteUsable = chosen !== null && chosen.site.id !== null;
  const usesDiveSite = withPlace && placeKind === "diveSite" && diveSiteUsable;
  const pickedSite = sites.find((s) => s.id === pick.siteId) ?? null;
  const gps = gpsInput(pick.lat, pick.lon);
  const gpsError = withPlace && placeKind === "site" && pick.siteId !== null && gps === undefined;

  // Without a usable dive site, the location can only be another site; the
  // dive's site comes back by itself once a dive with a site is chosen.
  const choosePlaceKind = (kind: "diveSite" | "site") => {
    setPlaceKind(kind);
    if (kind === "site" && pick.siteId === null) setPick(pickAround(chosen?.site ?? null));
  };

  const placeName = usesDiveSite ? chosen?.site.name : withPlace && placeKind === "site" ? pickedSite?.name : null;
  const diveLabel = chosen ? String(chosen.number ?? chosen.site.name ?? "—") : "";

  // What blocks Apply, in the order the user meets it.
  let blocker: string | null = null;
  if (!lr.connected) blocker = t("tag.disconnected");
  else if (lr.selected === 0) blocker = t("tag.selectInLr");
  else if (!withDive && !withPlace) blocker = t("tag.chooseSomething");
  else if (withDive && !chosen) blocker = t("tag.chooseDive");
  else if (needsValues && readFailure) blocker = readFailure;
  else if (profileLoading) blocker = t("detail.profileLoading");
  else if (needsValues && (!current || !freshCounts)) blocker = t("tag.perPhoto.reading");
  else if (withPlace && !usesDiveSite && !pickedSite) blocker = t("tag.chooseSite");
  else if (gpsError) blocker = t("tag.badGps");

  const summary = !blocker
    ? chosen && placeName
      ? t("tag.sumDiveAndPlace", { n: diveLabel, site: placeName })
      : chosen
        ? t("tag.sumDive", { n: diveLabel })
        : t("tag.sumPlace", { site: placeName ?? "" })
    : blocker;

  const apply = () => {
    const choice: TagChoice = {
      diveId: chosen?.id ?? null,
      location: !withPlace
        ? null
        : usesDiveSite
          ? { kind: "diveSite" }
          : { kind: "site", siteId: pick.siteId as string, lat: gps?.lat ?? null, lon: gps?.lon ?? null },
      search: needsValues && current ? current.search : null,
      offsets: needsValues ? offsets : {},
    };
    onDismissNotice();
    setRun({ state: "writing", done: 0, total: lr.selected });
    tagSelection(logRef(log.source), choice)
      .then((outcome) => {
        setRun(
          outcome.outcome === "applied"
            ? { state: "done", count: outcome.count }
            : { state: "failed", message: failure(outcome) },
        );
        // Described again: the offsets applied are now the remembered ones,
        // or the selection changed meanwhile.
        if (needsValues) setAgain((n) => n + 1);
        if (outcome.outcome === "applied") setTouched(new Set());
      })
      .catch((e: TagError) => {
        setRun({ state: "failed", message: failure(e) });
        if (selectionChanged(e)) setAgain((n) => n + 1);
      });
  };

  const writing = run.state === "writing";

  return (
    <main className="screen screen-scroll">
      <header className="screen-head">
        <div>
          <h1>{t("tag.title")}</h1>
          <p className="muted">{t("tag.subtitle")}</p>
        </div>
      </header>
      {notice && <LogNotice text={notice} onDismiss={onDismissNotice} />}

      <section className="card tag-selection" aria-label={t("tag.live")}>
        <div className="tag-selection-text">
          <span className="tag-live">
            <span className={`dot ${lr.connected ? "" : "dot-warn"}`} />
            {t("tag.live")}
          </span>
          <span className="tag-count">
            {!lr.connected ? t("tag.disconnected") : lr.selected > 0 ? tn("tag.photos", lr.selected) : t("tag.selectInLr")}
            {lr.connected && lr.selected > 0 && lr.selectionFrom && lr.selectionTo && (
              <span className="tag-span"> · {formatPhotoSpan(lr.selectionFrom, lr.selectionTo)}</span>
            )}
          </span>
        </div>
        {lr.connected && lr.selected > 0 && (
          <div className="thumbs" aria-label={t("tag.preview")}>
            {Array.from({ length: Math.min(THUMBNAILS, lr.selected) }, (_, i) =>
              thumbnails[i] ? (
                <img key={i} className="thumb" src={thumbnails[i] as string} alt="" />
              ) : (
                <div key={i} className="thumb" aria-hidden="true" />
              ),
            )}
            {lr.selected > THUMBNAILS && <div className="thumb thumb-more">+{lr.selected - THUMBNAILS}</div>}
          </div>
        )}
      </section>

      <section className="card tag-what" aria-label={t("tag.what")}>
        <div className="blk">
          <label className="chk">
            <input type="checkbox" checked={withDive} onChange={(e) => setWithDive(e.target.checked)} />
            {t("tag.diveData")}
          </label>
          {withDive ? (
            <>
              <FilterBar
                dives={dives}
                filters={filters}
                onChange={setFilters}
                capabilities={capabilities}
                units={units}
                shown={shown.length}
              />
              <div className="dives-body tag-dives">
                {shown.length > 0 ? (
                  <DiveList
                    dives={shown}
                    mode="single"
                    grouped={sort !== "depth"}
                    focusedId={focusedId}
                    onFocus={setFocusedId}
                    chosenId={chosenId}
                    onChoose={setChosenId}
                    mark={covering ? { id: covering.id, label: t("tag.covers") } : null}
                    capabilities={capabilities}
                    units={units}
                    source={sourceName(log.source.kind)}
                  />
                ) : (
                  <div className="card list-card list-empty">
                    <p className="muted">{t("filter.none")}</p>
                    {hasFilters(filters) && (
                      <button type="button" className="btn btn-s" onClick={() => setFilters(NO_FILTERS)}>
                        {t("filter.clearAll")}
                      </button>
                    )}
                  </div>
                )}
                <DiveDetail log={log.source} dive={focused} units={units} capabilities={capabilities} />
              </div>
              {chosen && !chosen.hasProfile && <p className="small">{t("tag.perPhoto.noProfile")}</p>}
              {unreadable && <p className="small">{t("tag.perPhoto.unreadable")}</p>}
              {needsValues && (
                <section className="tag-clock" aria-labelledby="h-tag-clock">
                  <div className="tag-clock-head">
                    <h2 id="h-tag-clock">{t("coll.clock.title")}</h2>
                    <p className="muted small">{t("coll.clock.text")}</p>
                  </div>
                  {readFailure ? (
                    <div className="notice notice-warn" role="alert">
                      <span>{readFailure}</span>
                      <button type="button" className="btn btn-g" onClick={() => setAgain((n) => n + 1)}>
                        {t("coll.retry")}
                      </button>
                    </div>
                  ) : !described || !shownCounts ? (
                    <p className="muted small">
                      {profileLoading ? t("detail.profileLoading") : t("tag.perPhoto.reading")}
                    </p>
                  ) : (
                    <>
                      <p className="strong" aria-live="polite">
                        {tn("tag.perPhoto.count", shownCounts.inDive, { total: formatInt(shownCounts.total) })}
                      </p>
                      {described.photos.cameras.length > 0 && (
                        <div className="cams">
                          {described.photos.cameras.map((c) => {
                            const count = shownCounts.cameras.find((k) => k.key === c.key);
                            const inDive = count?.inDives ?? 0;
                            return (
                              <CameraCard
                                key={c.key}
                                camera={c}
                                offset={offsets[c.key] ?? 0}
                                remembered={
                                  c.remembered !== null && offsets[c.key] === c.remembered && !touched.has(c.key)
                                }
                                inDives={inDive}
                                outside={count?.outside ?? 0}
                                gauge={t("tag.perPhoto.inDive", { n: formatInt(inDive) })}
                                note={notes[c.key] || null}
                                onOffset={(m) => setOffset(c.key, m)}
                                onSuggest={() => suggest(c.key)}
                              />
                            );
                          })}
                        </div>
                      )}
                    </>
                  )}
                </section>
              )}
              <p className="muted small">{t(needsValues ? "tag.diveDataWritesPerPhoto" : "tag.diveDataWrites")}</p>
            </>
          ) : (
            <p className="muted small">{t("tag.diveDataOff")}</p>
          )}
        </div>

        <div className="blk blk-sep">
          <label className="chk">
            <input type="checkbox" checked={withPlace} onChange={(e) => setWithPlace(e.target.checked)} />
            {t("tag.location")}
          </label>
          {withPlace ? (
            <>
              <div className="opts" role="radiogroup" aria-label={t("tag.location")}>
                <label className={`opt opt-wide ${usesDiveSite ? "opt-on" : ""} ${diveSiteUsable ? "" : "opt-off"}`}>
                  <input
                    type="radio"
                    name="place"
                    checked={usesDiveSite}
                    disabled={!diveSiteUsable}
                    onChange={() => choosePlaceKind("diveSite")}
                  />
                  <span>
                    <span className="strong">{t("tag.diveSite")}</span>{" "}
                    <span className="muted small">
                      ·{" "}
                      {chosen && diveSiteUsable
                        ? [siteLine(chosen.site), formatGps(chosen.site.lat, chosen.site.lon)].filter(Boolean).join(" · ")
                        : chosen
                          ? t("tag.diveHasNoSite")
                          : t("tag.noDiveChosen")}
                    </span>
                  </span>
                </label>
                <label className={`opt ${!usesDiveSite ? "opt-on" : ""}`}>
                  <input type="radio" name="place" checked={!usesDiveSite} onChange={() => choosePlaceKind("site")} />
                  <span className="strong">{t("tag.otherSite")}</span>
                </label>
              </div>
              {!usesDiveSite && <SitePicker sites={sites} pick={pick} onChange={setPick} gpsError={gpsError} />}
              <p className="muted small">{t("tag.locationWrites")}</p>
            </>
          ) : (
            <p className="muted small">{t("tag.locationOff")}</p>
          )}
        </div>
      </section>

      {run.state === "done" && (
        <div className="notice notice-ok" role="status">
          <span>
            <span className="strong">{tn("tag.done", run.count)}</span>{" "}
            {t("tag.undoHint")}
          </span>
          <button type="button" className="btn btn-g" onClick={() => setRun({ state: "idle" })}>
            {t("tag.dismiss")}
          </button>
        </div>
      )}
      {run.state === "failed" && (
        <div className="notice notice-warn" role="alert">
          <span>{run.message}</span>
          <button type="button" className="btn btn-g" onClick={() => setRun({ state: "idle" })}>
            {t("tag.dismiss")}
          </button>
        </div>
      )}

      <div className="card selection-bar apply-bar">
        {writing ? (
          <div className="apply-progress">
            <div className="apply-progress-text">
              <span className="strong">{t("tag.writing")}</span>
              {run.done > 0 && <span className="muted">{t("tag.progress", { done: run.done, total: run.total })}</span>}
            </div>
            <div
              className="bar"
              role="progressbar"
              aria-label={t("tag.writing")}
              aria-valuemin={0}
              aria-valuemax={run.total}
              aria-valuenow={run.done}
            >
              <div className="bar-fill" style={{ width: `${run.total > 0 ? (100 * run.done) / run.total : 0}%` }} />
            </div>
            <span className="muted small">{t("tag.writingHint")}</span>
          </div>
        ) : (
          <span className={blocker ? "muted" : ""}>
            <span className={blocker ? "" : "strong"}>{summary}</span>
            {!blocker && <span className="muted"> → {tn("tag.photos", lr.selected)}</span>}
          </span>
        )}
        <div className="selection-actions">
          <button type="button" className="btn btn-s" onClick={onCancel} disabled={writing}>
            {t("tag.cancel")}
          </button>
          <button type="button" className="btn btn-p" disabled={blocker !== null || writing} onClick={apply}>
            {tn("tag.apply", lr.selected)}
          </button>
        </div>
      </div>
    </main>
  );
}
