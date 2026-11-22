// Tag Selection screen (spec F8): writes a dive's data and/or a location on
// the photos selected in Lightroom, whatever their time. The selection is
// made in Lightroom and followed live; Lightroom resolves it again at the
// moment of Apply, in one Undo step.
//
// When the chosen dive has a profile, the photos taken during it also get
// their per-photo values (F6): the selection is described photo by photo,
// each camera gets a clock offset as in Create Collections, and Apply
// writes only if the selection is still the one described.

import { useEffect, useMemo, useRef, useState } from "react";
import {
  findSelectionPhotos,
  onApplyProgress,
  onSelectionPreview,
  selectionPreview,
  suggestTagOffsets,
  tagPreview,
  tagSelection,
} from "../api";
import { clampOffset, keepOffsets } from "../apply/cameras";
import { coveringDive } from "./covers";
import { logRef, nextSearchNumber } from "../diveLog";
import { sourceName } from "../sourceText";
import { CameraCard } from "../apply/CameraCard";
import { DiveDetail, useProfile } from "../dives/DiveDetail";
import { LogNotice } from "../ui/LogNotice";
import { DiveList } from "../dives/DiveList";
import { FilterBar } from "../dives/FilterBar";
import { NO_PICK, pickAround, SitePicker, type SitePick } from "./SitePicker";
import { formatGps, formatPhotoSpan } from "../ui/format";
import { EmptyList } from "../dives/EmptyList";
import { useDiveBrowser } from "../dives/useDiveBrowser";
import type { Sort } from "../dives/filters";
import { formatInt, t, tn } from "../i18n";
import { gpsInput } from "./sites";
import { canApplyTag, describeAgain, gpsRefused, readingAfterFailure, selectionChanged, tagBlocker, tagSummary } from "./rules";
import type {
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
export function describeFailure(e: CollectionsError): string | null {
  switch (e.error) {
    case "noPhotos":
    case "logChanged": // the app reads the log again and starts the screen again
      return null;
    case "notConnected":
    case "linkLost":
      return t("tag.disconnected");
    case "lightroom":
      return e.detail ? t("tag.perPhoto.failed", { detail: e.detail }) : t("tag.perPhoto.failedNoDetail");
    case "source":
      return t("tag.perPhoto.failed", { detail: e.detail.detail });
    default:
      return t("tag.perPhoto.failedNoDetail");
  }
}

export function failure(e: TagError): string {
  if (selectionChanged(e)) return t("tag.err.selectionChanged");
  const key = e.error === "lightroom" ? e.reason : e.error;
  switch (key) {
    case "noSelection":
      return t("tag.err.noSelection");
    case "busy":
      return t("tag.err.busy");
    case "gpsNotWritten": {
      // Written but for the GPS and altitude: Undo, then apply again.
      const detail = e.error === "lightroom" ? e.detail : null;
      return detail ? t("tag.err.gpsNotWrittenDetail", { detail }) : t("tag.err.gpsNotWritten");
    }
    case "linkLost":
      return t("tag.err.linkLost");
    case "notConnected":
      return t("tag.err.notConnected");
    case "nothingChosen":
      return t("tag.err.nothingChosen");
    case "badGps":
      return t("tag.err.badGps");
    case "unknownDive":
      return t("tag.err.unknownDive");
    case "unknownSite":
      return t("tag.err.unknownSite");
    case "logChanged":
      return t("log.changedNothingWritten");
    case "badRequest":
      return t("tag.err.badRequest");
    default:
      if (e.error === "source") return t("tag.err.source", { detail: e.detail.detail });
      // A reason this version does not know: say what it can, never the code.
      return e.error === "lightroom" && e.detail ? t("tag.err.other", { detail: e.detail }) : t("tag.err.lightroom");
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
  const [run, setRun] = useState<Run>({ state: "idle" });

  const { filters, setFilters, shown, focusedId, setFocusedId, focused } = useDiveBrowser(dives, sort);

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
  // The selection version already described again after a lost description: at most once each.
  const retried = useRef<number | null>(null);
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
        const after = readingAfterFailure(e, live, retried.current === version);
        if (!after) return;
        setReadFailure(after.message ?? describeFailure(e));
        setReading(after.reading);
        if (after.describeAgain) {
          retried.current = version;
          setAgain((n) => n + 1);
        }
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

  const diveSiteUsable = chosen !== null && chosen.site.id !== null;
  const usesDiveSite = withPlace && placeKind === "diveSite" && diveSiteUsable;
  const pickedSite = sites.find((s) => s.id === pick.siteId) ?? null;
  const gps = gpsInput(pick.lat, pick.lon);
  const gpsError = gpsRefused({ withPlace, placeKind, diveSiteUsable, siteId: pick.siteId, gpsValid: gps !== undefined });

  // Without a usable dive site, the location can only be another site; the
  // dive's site comes back by itself once a dive with a site is chosen.
  const choosePlaceKind = (kind: "diveSite" | "site") => {
    setPlaceKind(kind);
    if (kind === "site" && pick.siteId === null) setPick(pickAround(chosen?.site ?? null));
  };

  const placeName = usesDiveSite ? chosen?.site.name : withPlace && placeKind === "site" ? pickedSite?.name : null;

  // What blocks Apply, in the order the user meets it.
  const blocker = tagBlocker({
    connected: lr.connected,
    selected: lr.selected,
    withDive,
    withPlace,
    chosen: chosen !== null,
    needsValues,
    readFailure,
    profileLoading,
    current: current !== null,
    freshCounts: !!freshCounts,
    usesDiveSite,
    pickedSite: pickedSite !== null,
    gpsError,
  });

  const summary = tagSummary(blocker, chosen, placeName);

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
      .then(({ count }) => {
        setRun({ state: "done", count });
        // Described again: the offsets applied are now the remembered ones.
        if (needsValues) setAgain((n) => n + 1);
        setTouched(new Set());
      })
      .catch((e: TagError) => {
        setRun({ state: "failed", message: failure(e) });
        // Described again: the offsets applied may be the remembered ones now,
        // or the selection changed meanwhile.
        if (describeAgain(e, needsValues)) setAgain((n) => n + 1);
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
                  <EmptyList filters={filters} onChange={setFilters} />
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
          <button type="button" className="btn btn-p" disabled={!canApplyTag(blocker, writing)} onClick={apply}>
            {tn("tag.apply", lr.selected)}
          </button>
        </div>
      </div>
    </main>
  );
}
