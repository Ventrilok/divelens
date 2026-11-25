// Create Collections, steps 2 and 3 (spec F5, F6): the dives checked in
// step 1, the cameras found in Lightroom over their period with one clock
// offset each, the surface margin, live counts per dive, then the write in
// Lightroom (one Undo step) and the summary. Only shown with at least one
// dive checked.

import { useEffect, useMemo, useState } from "react";
import { applyCollections, collectionSets, onApplyProgress } from "../api";
import { MARGIN_STEP_MIN, MAX_MARGIN_MIN } from "../apply/cameras";
import { placeLabel, setExists } from "./collectionPlace";
import { barText, canApplyCollections, collectionsBlocker, writesNote } from "./rules";
import { logRef } from "../diveLog";
import { CameraClock } from "../apply/CameraClock";
import { applyFailure } from "../apply/failure";
import { useCatalogPhotos } from "../journey/useCatalogPhotos";
import { ProgressBar } from "../apply/ProgressBar";
import { Icon } from "../ui/Icon";
import { Steps } from "../ui/Steps";
import { formatDay, formatRange, formatRowSpan } from "../ui/format";
import { formatInt, t, tn } from "../i18n";
import type { ApplyError, Camera, CollectionChoices, CollectionsResult, Dive, DiveLog, LightroomStatus, Settings } from "../types";

/** Where step 2 stands, for the app around it. */
export type Phase = "adjust" | "writing" | "summary";

type Run =
  | { state: "idle" }
  | { state: "writing"; done: number; total: number }
  | { state: "done"; result: CollectionsResult; choices: CollectionChoices; cameras: Camera[] }
  | { state: "failed"; message: string };

function diveName(d: Dive): string {
  return [d.number !== null ? String(d.number) : null, d.site.name].filter(Boolean).join(" ");
}

function ResultView({
  result,
  dives,
  choices,
  cameras,
  onBack,
  onRetry,
}: {
  result: CollectionsResult;
  dives: Dive[];
  choices: CollectionChoices;
  cameras: Camera[];
  /** Back to step 1, with no dive checked. */
  onBack: () => void;
  onRetry: (ids: string[]) => void;
}) {
  const byId = new Map(dives.map((d) => [d.id, d]));
  const empty = result.emptyDives.map((id) => byId.get(id)).filter((d): d is Dive => d !== undefined);
  const chosen = dives.filter((d) => byId.has(d.id));
  const start = chosen.reduce((a, d) => (d.start < a ? d.start : a), chosen[0]?.start ?? "");
  const end = chosen.reduce((a, d) => (d.end > a ? d.end : a), chosen[0]?.end ?? "");
  const ignored = cameras.filter((c) => choices.ignored.includes(c.key));
  const used = cameras.length - ignored.length;
  return (
    <main className="screen screen-scroll">
      <Steps current={3} />
      <header className="screen-head">
        <div>
          <p className="muted">
            {start && formatRange(start, end)} · {tn("coll.cameras", used)}
            {ignored.length > 0 && ` · ${t("coll.result.ignored", { names: ignored.map((c) => c.model ?? t("coll.unknownCamera")).join(", ") })}`}
          </p>
        </div>
      </header>
      <div className="result">
        <div className="result-head">
          <div className="result-icon">
            <Icon name="check" size={26} />
          </div>
          <h2>{t("coll.result.title")}</h2>
        </div>
        <div className="stats">
          <div className="card stat">
            <div className="lbl">{t("coll.result.photos")}</div>
            <div className="stat-n">{formatInt(result.photos)}</div>
          </div>
          <div className="card stat">
            <div className="lbl">{t("coll.result.withDepth")}</div>
            <div className="stat-n">{formatInt(result.withDepth)}</div>
          </div>
          <div className="card stat">
            <div className="lbl">{t("coll.result.collections")}</div>
            <div className="stat-n">{formatInt(result.collections.length)}</div>
          </div>
        </div>
        <section className="card result-list" aria-labelledby="h-set">
          <h2 id="h-set">Dive Lens</h2>
          <ul>
            {result.collections.map((c) => (
              <li key={c.collection}>
                <span>{c.collection}</span>
                <span className="muted">
                  {tn("coll.photos", c.count)} · {c.created ? t("coll.result.created") : t("coll.result.completed")}
                </span>
              </li>
            ))}
          </ul>
        </section>
        {empty.map((d) => (
          <div key={d.id} className="notice notice-warn" role="status">
            <span>{t("coll.result.empty", { dive: diveName(d) })}</span>
          </div>
        ))}
      </div>
      <div className="card selection-bar apply-bar">
        <div className="selection-text">
          <span className="muted small">{t("coll.undoHint")}</span>
        </div>
        <div className="selection-actions">
          {empty.length > 0 && (
            <button type="button" className="btn btn-s" onClick={() => onRetry(empty.map((d) => d.id))}>
              {empty.length === 1 ? t("coll.result.retryOne", { dive: diveName(empty[0]) }) : t("coll.result.retryMany")}
            </button>
          )}
          <button type="button" className="btn btn-p" onClick={onBack}>
            {t("coll.result.back")}
          </button>
        </div>
      </div>
    </main>
  );
}

export function CollectionsScreen({
  log,
  lr,
  settings,
  chosen,
  onChooseDives,
  onDone,
  onRetryDives,
  onOpenSettings,
  onPhase,
}: {
  log: DiveLog;
  lr: LightroomStatus;
  settings: Settings;
  chosen: Set<string>;
  /** Back to step 1, the checked dives kept. */
  onChooseDives: () => void;
  /** From the summary: back to step 1, with no dive checked. */
  onDone: () => void;
  onRetryDives: (ids: string[]) => void;
  onOpenSettings: () => void;
  /** Told when a write starts and when the summary shows. */
  onPhase: (phase: Phase) => void;
}) {
  const dives = useMemo(
    () => log.dives.filter((d) => chosen.has(d.id)).sort((a, b) => (a.start < b.start ? -1 : 1)),
    [log.dives, chosen],
  );
  const ids = useMemo(() => [...chosen], [chosen]);
  // The photos, their cameras, offsets, ignored cameras, margin and live counts.
  const photos = useCatalogPhotos(log, lr, ids, 0);
  const { search, period, cameras, offsets, ignored, touched, notes, margin, preview, previewFailed, choices } = photos;
  const [run, setRun] = useState<Run>({ state: "idle" });

  // The collection set chosen in Settings must still be in the catalog
  // (spec F2): checked when Lightroom connects, never replaced silently.
  const parent = settings.collections.parent;
  const parentKey = JSON.stringify(parent);
  const [parentMissing, setParentMissing] = useState(false);
  useEffect(() => {
    setParentMissing(false);
    if (!lr.connected || parent.length === 0) return;
    let live = true;
    collectionSets()
      .then((sets) => live && setParentMissing(!setExists(sets, parent)))
      .catch(() => {});
    return () => {
      live = false;
    };
  }, [lr.connected, lr.catalog, parentKey]);

  useEffect(() => {
    const stop = onApplyProgress((done, total) =>
      setRun((r) => (r.state === "writing" ? { state: "writing", done, total } : r)),
    );
    return () => {
      stop.then((f) => f());
    };
  }, []);

  const phase: Phase = run.state === "writing" ? "writing" : run.state === "done" ? "summary" : "adjust";
  useEffect(() => {
    onPhase(phase);
    return () => onPhase("adjust");
  }, [phase, onPhase]);

  const apply = () => {
    if (!preview || !period) return;
    const cams = period.cameras;
    setRun({ state: "writing", done: 0, total: preview.total });
    const options = { dive: true, altitude: settings.depthToAltitude, place: true, gps: true, collections: true };
    applyCollections(logRef(log.source), period.search, choices, options)
      .then((result) => setRun({ state: "done", result, choices, cameras: cams }))
      .catch((e: ApplyError) => {
        setRun({ state: "failed", message: applyFailure(e, "collections") });
        if (e.error === "noPhotos" || (e.error === "lightroom" && e.reason === "staleSearch")) photos.searchAgain();
      });
  };

  if (run.state === "done") {
    return (
      <ResultView
        result={run.result}
        dives={dives}
        choices={run.choices}
        cameras={run.cameras}
        onBack={onDone}
        onRetry={(ids) => {
          setRun({ state: "idle" });
          onRetryDives(ids);
        }}
      />
    );
  }

  const start = dives[0].start;
  const end = dives.reduce((a, d) => (d.end > a ? d.end : a), dives[0].end);
  const counts = new Map(preview?.dives.map((c) => [c.id, c]) ?? []);
  const camCounts = new Map(preview?.cameras.map((c) => [c.key, c]) ?? []);
  const writing = run.state === "writing";
  const total = preview?.total ?? 0;
  const withPhotos = preview?.dives.filter((d) => d.photos > 0).length ?? 0;

  const blocker = collectionsBlocker({ connected: lr.connected, search, parentMissing, parent, preview, previewFailed });

  const place = placeLabel(settings.collections, t("settings.root"));

  return (
    <main className="screen screen-scroll">
      <Steps current={2} onGo={writing ? undefined : onChooseDives} />
      <header className="screen-head">
        <div>
          <p className="muted">
            {tn("coll.dives", dives.length)} · {formatRange(start, end)}
            {period &&
              ` · ${t("coll.period", {
                photos: tn("coll.photos", period.photos),
                cameras: tn("coll.cameras", cameras.length),
                from: formatDay(period.from),
                to: formatDay(period.to),
              })}`}
          </p>
        </div>
      </header>

      {!lr.connected && (
        <div className="notice notice-warn" role="status">
          <span>{t("coll.disconnected")}</span>
        </div>
      )}
      {search.state === "searching" && <p className="muted">{t("coll.searching")}</p>}
      {search.state === "failed" && (
        <div className="notice notice-warn" role="alert">
          <span>{search.message}</span>
          <button type="button" className="btn btn-g" onClick={photos.searchAgain}>
            {t("coll.retry")}
          </button>
        </div>
      )}

      {period && (
        <>
          <section className="card coll-sec" aria-labelledby="h-clock">
            <div className="coll-sec-head">
              <div>
                <h2 id="h-clock">{t("coll.clock.title")}</h2>
                <p className="muted small">{t("coll.clock.text")}</p>
              </div>
              {cameras.length > 0 && (
                <button type="button" className="btn btn-s" onClick={photos.suggestAll} disabled={writing}>
                  <Icon name="spark" />
                  {t("coll.suggestAll")}
                </button>
              )}
            </div>
            {cameras.length === 0 ? (
              <p className="muted">{t("coll.noPhotos")}</p>
            ) : (
              <CameraClock
                cameras={cameras}
                offsets={offsets}
                touched={touched}
                notes={notes}
                counts={(key) => camCounts.get(key)}
                gauge={(inDives, outside) =>
                  inDives === 0 ? t("coll.noneInDives") : t("coll.inDives", { n: formatInt(inDives), out: formatInt(outside) })
                }
                ignored={ignored}
                onChange={photos.setOffset}
                onIgnore={photos.setIgnored}
                onSuggest={photos.suggest}
                disabled={writing}
              />
            )}
          </section>

          <section className="card coll-sec coll-margin" aria-labelledby="h-margin">
            <div className="coll-margin-text">
              <h2 id="h-margin">{t("coll.margin.title")}</h2>
              <p className="muted small">{t("coll.margin.text")}</p>
            </div>
            <div className="coll-margin-ctl">
              <span>{t("coll.margin.upTo")}</span>
              <button
                type="button"
                className="step"
                aria-label={t("coll.margin.less")}
                disabled={margin <= 0}
                onClick={() => photos.setMargin(margin - MARGIN_STEP_MIN)}
              >
                −
              </button>
              <span className="coll-margin-n">{margin} min</span>
              <button
                type="button"
                className="step"
                aria-label={t("coll.margin.more")}
                disabled={margin >= MAX_MARGIN_MIN}
                onClick={() => photos.setMargin(margin + MARGIN_STEP_MIN)}
              >
                +
              </button>
              <span>{t("coll.margin.around")}</span>
            </div>
          </section>

          <section className="card coll-table" aria-labelledby="h-dives">
            <h2 id="h-dives">{t("coll.byDive")}</h2>
            <div className="coll-table-scroll">
              <table className="tbl">
                <thead>
                  <tr>
                    <th className="num">{t("col.number")}</th>
                    <th>{t("col.site")}</th>
                    <th>{t("coll.col.dive")}</th>
                    <th className="num">{t("coll.col.photos")}</th>
                    <th>{t("coll.col.cameras")}</th>
                    <th>{t("coll.col.perPhoto")}</th>
                  </tr>
                </thead>
                <tbody>
                  {dives.map((d) => {
                    const c = counts.get(d.id);
                    const none = c !== undefined && c.photos === 0;
                    return (
                      <tr key={d.id} className={none ? "row-warn" : ""}>
                        <td className="num">{d.number ?? "—"}</td>
                        <td className="site-name">{d.site.name ?? "—"}</td>
                        <td>{formatRowSpan(d.start, d.end)}</td>
                        <td className={`num strong ${none ? "warn" : ""}`}>{c ? formatInt(c.photos) : "…"}</td>
                        <td className="muted">{c && c.cameras > 0 ? tn("coll.cameras", c.cameras) : "—"}</td>
                        <td className={none ? "warn" : ""}>
                          {none
                            ? t("coll.noPhotoHint")
                            : !d.hasProfile
                              ? t("coll.noProfile")
                              : c && c.inDive === 0
                                ? t("coll.surfaceOnly")
                                : t("coll.perPhoto")}
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
            {preview && (
              <p className="muted small coll-total">
                {t("coll.total", {
                  photos: tn("coll.photos", preview.total),
                  dives: tn("coll.dives", withPhotos),
                  outside: formatInt(preview.outside),
                })}
                {preview.ignored > 0 && ` · ${t("coll.totalIgnored", { n: formatInt(preview.ignored) })}`}
                {` · ${t("coll.noCollectionHint")}`}
              </p>
            )}
          </section>
        </>
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
          <div className="selection-text">
            <span className="small">{barText(blocker, place)}</span>
            <span className="muted small">
              {writesNote(log.units, settings.depthToAltitude)} ·{" "}
              {t("coll.undoShort")}
            </span>
          </div>
        )}
        <div className="selection-actions">
          {parentMissing && !writing && (
            <button type="button" className="btn btn-s" onClick={onOpenSettings}>
              {t("coll.openSettings")}
            </button>
          )}
          <button type="button" className="btn btn-s" onClick={onChooseDives} disabled={writing}>
            <Icon name="back" />
            {t("coll.back")}
          </button>
          <button
            type="button"
            className="btn btn-p"
            disabled={!canApplyCollections(blocker, preview, writing)}
            onClick={apply}
          >
            {tn("tag.apply", total)}
            <Icon name="arrow" />
          </button>
        </div>
      </div>
    </main>
  );
}
