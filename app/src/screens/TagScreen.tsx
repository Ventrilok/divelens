// Tag Selection screen (spec F8): writes a dive's data and/or a location on
// the photos selected in Lightroom, whatever their time. The selection is
// made in Lightroom and followed live; Lightroom resolves it again at the
// moment of Apply, in one undoable step.

import { useDeferredValue, useEffect, useMemo, useState } from "react";
import { onApplyProgress, onSelectionPreview, selectionPreview, tagSelection } from "../api";
import { coveringDive } from "../covers";
import { DiveDetail } from "../components/DiveDetail";
import { DiveList } from "../components/DiveList";
import { FilterBar } from "../components/FilterBar";
import { NO_PICK, pickAround, SitePicker, type SitePick } from "../components/SitePicker";
import { formatGps, formatPhotoSpan } from "../format";
import { applyFilters, hasFilters, NO_FILTERS, searchIndex, sortDives, type Filters } from "../filters";
import { t, tn } from "../i18n";
import { gpsInput } from "../sites";
import type { ApplyOutcome, Dive, DiveLog, LightroomStatus, SelectionPreview, Site, TagChoice, TagError } from "../types";

const THUMBNAILS = 6;

type Run =
  | { state: "idle" }
  | { state: "writing"; done: number; total: number }
  | { state: "done"; count: number }
  | { state: "failed"; message: string };

function siteLine(site: Site): string {
  return [site.name, site.location, site.country].filter(Boolean).join(", ");
}

function failure(e: ApplyOutcome | TagError): string {
  const key = "outcome" in e ? (e.outcome === "failed" ? e.reason : "") : e.error;
  switch (key) {
    case "noSelection":
      return t("tag.err.noSelection");
    case "busy":
      return t("tag.err.busy");
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

export function TagScreen({ log, lr, onCancel }: { log: DiveLog; lr: LightroomStatus; onCancel: () => void }) {
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
  const shown = useMemo(() => sortDives(applyFilters(dives, deferredFilters, index), "recent"), [dives, deferredFilters, index]);

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

  // The dive covering the selection is pointed out and shown in the detail
  // card, never chosen for the user.
  const covering = useMemo(
    () => coveringDive(dives, lr.coverFrom, lr.coverTo),
    [dives, lr.coverFrom, lr.coverTo],
  );
  useEffect(() => {
    if (covering) setFocusedId(covering.id);
  }, [covering]);

  const focused = dives.find((d) => d.id === focusedId) ?? null;
  const chosen: Dive | null = withDive ? (dives.find((d) => d.id === chosenId) ?? null) : null;
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
    };
    setRun({ state: "writing", done: 0, total: lr.selected });
    tagSelection(choice)
      .then((outcome) =>
        setRun(outcome.outcome === "applied" ? { state: "done", count: outcome.count } : { state: "failed", message: failure(outcome) }),
      )
      .catch((e: TagError) => setRun({ state: "failed", message: failure(e) }));
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
                    grouped
                    focusedId={focusedId}
                    onFocus={setFocusedId}
                    chosenId={chosenId}
                    onChoose={setChosenId}
                    mark={covering ? { id: covering.id, label: t("tag.covers") } : null}
                    capabilities={capabilities}
                    units={units}
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
                <DiveDetail dive={focused} units={units} capabilities={capabilities} />
              </div>
              <p className="muted small">{t("tag.diveDataWrites")}</p>
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
            <span className="strong">{tn("tag.done", run.count)}</span> {t("tag.undoHint")}
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
          <button type="button" className="btn btn-g" onClick={onCancel} disabled={writing}>
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
