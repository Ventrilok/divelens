// Tag Selection screen (spec F8): writes a dive's data and/or a location on
// the photos selected in Lightroom, whatever their time. The selection is
// made in Lightroom and followed live; Lightroom resolves it again at the
// moment of Apply, in one Undo step.
//
// When the chosen dive has a profile, the photos taken during it also get
// their per-photo values (F6): the selection is described photo by photo,
// each camera gets a clock offset as in Create Collections, and Apply
// writes only if the selection is still the one described.

import { useEffect, useState } from "react";
import { onApplyProgress, onSelectionPreview, selectionPreview, tagSelection } from "../api";
import { logRef } from "../diveLog";
import { sourceName } from "../sourceText";
import { CameraClock } from "../apply/CameraClock";
import { applyFailure } from "../apply/failure";
import { ProgressBar } from "../apply/ProgressBar";
import { DiveDetail } from "../dives/DiveDetail";
import { useSelectionPhotos } from "../journey/useSelectionPhotos";
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
import { canApplyTag, describeAgain, gpsRefused, tagBlocker, tagSummary } from "./rules";
import type { ApplyError, Dive, DiveLog, LightroomStatus, SelectionPreview, Settings, Site, TagChoice } from "../types";

const THUMBNAILS = 6;

type Run =
  | { state: "idle" }
  | { state: "writing"; done: number; total: number }
  | { state: "done"; count: number }
  | { state: "failed"; message: string };

function siteLine(site: Site): string {
  return [site.name, site.location, site.country].filter(Boolean).join(", ");
}

export function TagScreen({
  log,
  lr,
  settings,
  sort,
  notice,
  onDismissNotice,
  onCancel,
}: {
  log: DiveLog;
  lr: LightroomStatus;
  /** For the altitude option, which follows the "depth copied to altitude" setting. */
  settings: Settings;
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

  // The selection photo by photo, described for a dive whose profile can be
  // read (per-photo values), again whenever the selection changes, its
  // cameras with their offsets, and the live counts (spec F8). Not `active`:
  // this screen reads the selection only for such a dive, as before.
  const photos = useSelectionPhotos(log, lr, chosen, false, 0);
  const { described, current, readFailure, offsets, touched, notes, needsValues, unreadable, covering } = photos;
  const shownCounts = photos.counts;
  const freshCounts = photos.freshCounts;
  const profileLoading = needsValues && photos.profile.status === "loading";

  // The dive covering the selection is pointed out, never chosen for the
  // user. It is shown in the detail card only until a dive is chosen: then
  // the detail stays.
  useEffect(() => {
    if (covering && chosenId === null) setFocusedId(covering.id);
  }, [covering]);

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
      options: { dive: withDive, altitude: settings.depthToAltitude, place: withPlace, gps: withPlace, collections: false },
    };
    onDismissNotice();
    setRun({ state: "writing", done: 0, total: lr.selected });
    tagSelection(logRef(log.source), choice)
      .then(({ count }) => {
        setRun({ state: "done", count });
        // Described again: the offsets applied are now the remembered ones.
        if (needsValues) photos.describeAgain();
        photos.clearTouched();
      })
      .catch((e: ApplyError) => {
        setRun({ state: "failed", message: applyFailure(e, "tag") });
        // Described again: the offsets applied may be the remembered ones now,
        // or the selection changed meanwhile.
        if (describeAgain(e, needsValues)) photos.describeAgain();
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
                      <button type="button" className="btn btn-g" onClick={photos.describeAgain}>
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
                        <CameraClock
                          cameras={described.photos.cameras}
                          offsets={offsets}
                          touched={touched}
                          notes={notes}
                          counts={(key) => shownCounts.cameras.find((k) => k.key === key)}
                          gauge={(inDives) => t("tag.perPhoto.inDive", { n: formatInt(inDives) })}
                          onChange={photos.setOffset}
                          onSuggest={photos.suggest}
                          disabled={writing}
                        />
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
          <ProgressBar done={run.done} total={run.total} label={t("tag.writing")} />
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
