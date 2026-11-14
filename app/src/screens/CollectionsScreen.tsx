// Create Collections, steps 2 and 3 (spec F5, F6): the dives checked in
// step 1, the cameras found in Lightroom over their period with one clock
// offset each, the surface margin, live counts per dive, then the write in
// Lightroom (one Undo step) and the summary. Only shown with at least one
// dive checked.

import { useEffect, useMemo, useRef, useState } from "react";
import { applyCollections, collectionSets, collectionsPreview, findCollectionPhotos, onApplyProgress, suggestOffsets } from "../api";
import {
  clampMargin,
  clampOffset,
  initialOffsets,
  MARGIN_STEP_MIN,
  MAX_MARGIN_MIN,
  offsetField,
  parseOffset,
  SLIDER_RANGE_MIN,
  SLIDER_STEP_MIN,
} from "../cameras";
import { placeLabel, setExists } from "../collectionPlace";
import { logRef, nextSearchNumber } from "../diveLog";
import { Icon } from "../components/Icon";
import { Steps } from "../components/Steps";
import { formatDay, formatRange, formatRowSpan } from "../format";
import { formatInt, t, tn } from "../i18n";
import type {
  Camera,
  CollectionChoices,
  CollectionsError,
  CollectionsPreview,
  CollectionsResult,
  Dive,
  DiveLog,
  LightroomStatus,
  Period,
  Settings,
} from "../types";

type Search =
  | { state: "idle" }
  | { state: "searching" }
  | { state: "ready"; period: Period }
  | { state: "failed"; message: string };

/** Where step 2 stands, for the app around it. */
export type Phase = "adjust" | "writing" | "summary";

type Run =
  | { state: "idle" }
  | { state: "writing"; done: number; total: number }
  | { state: "done"; result: CollectionsResult; choices: CollectionChoices; cameras: Camera[] }
  | { state: "failed"; message: string };

function failure(e: CollectionsError): string {
  switch (e.error) {
    case "notConnected":
      return t("coll.err.notConnected");
    case "linkLost":
      return t("coll.err.linkLost");
    case "noPhotos":
      return t("coll.err.noPhotos");
    case "nothingToWrite":
      return t("coll.err.nothingToWrite");
    case "nothingChosen":
      return t("coll.err.nothingChosen");
    case "logChanged":
      // Not seen: the app reads the log again and says why on step 1.
      return t("log.changedNothingWritten");
    case "lightroom":
      if (e.reason === "busy") return t("coll.err.busy");
      // Written but for the GPS and altitude: Undo, then apply again.
      if (e.reason === "gpsNotWritten") return t("coll.err.gpsNotWritten");
      if (e.reason === "staleSearch") return t("coll.err.noPhotos");
      if (e.reason === "parentNotFound") return t("coll.parentMissing", { path: e.detail ?? "" });
      return t("coll.err.other", { detail: e.detail ?? e.reason });
    case "source":
      return t("coll.err.other", { detail: e.detail.detail });
  }
}

/** The entries of `o` whose key is in `keys`. */
function pick(o: Record<string, number>, keys: Set<string>): Record<string, number> {
  return Object.fromEntries(Object.entries(o).filter(([k]) => keys.has(k)));
}

function diveName(d: Dive): string {
  return [d.number !== null ? String(d.number) : null, d.site.name].filter(Boolean).join(" ");
}

/** Offset typed to the minute; arrows add or remove a minute. */
function OffsetField({ offset, label, onOffset }: { offset: number; label: string; onOffset: (min: number) => void }) {
  const [text, setText] = useState(offsetField(offset));
  const [editing, setEditing] = useState(false);
  useEffect(() => {
    if (!editing) setText(offsetField(offset));
  }, [offset, editing]);
  const commit = () => {
    const value = parseOffset(text);
    if (value !== null) onOffset(value);
    else setText(offsetField(offset));
    setEditing(false);
  };
  const bad = editing && parseOffset(text) === null;
  return (
    <input
      className="inp off-field"
      aria-label={label}
      aria-invalid={bad}
      value={text}
      onFocus={() => setEditing(true)}
      onChange={(e) => setText(e.target.value)}
      onBlur={commit}
      onKeyDown={(e) => {
        if (e.key === "Enter") e.currentTarget.blur();
        else if (e.key === "Escape") {
          setText(offsetField(offset));
          setEditing(false);
          e.currentTarget.blur();
        } else if (e.key === "ArrowUp" || e.key === "ArrowDown") {
          e.preventDefault();
          const next = offset + (e.key === "ArrowUp" ? 1 : -1);
          onOffset(next);
          setText(offsetField(clampOffset(next)));
        }
      }}
    />
  );
}

function CameraCard({
  camera,
  offset,
  ignored,
  remembered,
  inDives,
  outside,
  note,
  onOffset,
  onIgnore,
  onSuggest,
}: {
  camera: Camera;
  offset: number;
  ignored: boolean;
  remembered: boolean;
  inDives: number;
  outside: number;
  note: string | null;
  onOffset: (min: number) => void;
  onIgnore: (ignored: boolean) => void;
  onSuggest: () => void;
}) {
  const warn = !ignored && inDives === 0;
  const name = camera.model ?? t("coll.unknownCamera");
  const total = inDives + outside;
  return (
    <div className={`cam ${warn ? "cam-warn" : ""} ${ignored ? "cam-off" : ""}`}>
      <div className="cam-h">
        {warn ? (
          <Icon name="warn" className="warn" />
        ) : ignored ? (
          <Icon name="close" className="muted" />
        ) : (
          <Icon name="check" className="accent" />
        )}
        <div className="cam-n">
          <b>{name}</b>
          <span>
            {camera.serial && `${t("coll.serial", { n: camera.serial })} · `}
            {tn("coll.photos", camera.photos)}
          </span>
        </div>
        <label className="ign">
          <input type="checkbox" checked={ignored} onChange={(e) => onIgnore(e.target.checked)} />
          {t("coll.ignore")}
        </label>
      </div>
      {ignored ? (
        <p className="muted small">{t("coll.ignored")}</p>
      ) : (
        <>
          <div className="cam-c">
            <div className="slider">
              <input
                type="range"
                min={-SLIDER_RANGE_MIN}
                max={SLIDER_RANGE_MIN}
                step={SLIDER_STEP_MIN}
                value={Math.max(-SLIDER_RANGE_MIN, Math.min(SLIDER_RANGE_MIN, offset))}
                aria-label={t("coll.offset", { camera: name })}
                aria-valuetext={offsetField(offset)}
                onChange={(e) => onOffset(Number(e.target.value))}
              />
              <div className="slider-scale" aria-hidden="true">
                <span>−14 h</span>
                <span>0</span>
                <span>+14 h</span>
              </div>
            </div>
            <OffsetField offset={offset} label={t("coll.offsetField", { camera: name })} onOffset={onOffset} />
            <button type="button" className="mini" onClick={onSuggest}>
              <Icon name="spark" size={15} />
              {t("coll.suggest")}
            </button>
          </div>
          {remembered && <span className="cam-tag">{t("coll.remembered")}</span>}
          <div className={`cam-f ${warn ? "warn" : ""}`}>
            <span className="bar">
              <span className="bar-fill" style={{ width: `${total > 0 ? (100 * inDives) / total : 0}%` }} />
            </span>
            <span>
              {inDives === 0 ? t("coll.noneInDives") : t("coll.inDives", { n: formatInt(inDives), out: formatInt(outside) })}
            </span>
          </div>
          {note && <p className="muted small">{note}</p>}
        </>
      )}
    </div>
  );
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
  const diveKey = dives.map((d) => d.id).join(",");

  const [search, setSearch] = useState<Search>({ state: "idle" });
  const [again, setAgain] = useState(0);
  const [offsets, setOffsets] = useState<Record<string, number>>({});
  const [ignored, setIgnored] = useState<Set<string>>(new Set());
  const [touched, setTouched] = useState<Set<string>>(new Set());
  const [notes, setNotes] = useState<Record<string, string>>({});
  const [margin, setMargin] = useState(0);
  const [preview, setPreview] = useState<CollectionsPreview | null>(null);
  const [run, setRun] = useState<Run>({ state: "idle" });
  const searchedFor = useRef<string | null>(null);

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

  // Photos of the period, found again when the dives change or Lightroom
  // comes back (its photo ids only hold for one link session).
  useEffect(() => {
    if (!diveKey || !lr.connected) {
      setSearch({ state: "idle" });
      return;
    }
    let live = true;
    setSearch({ state: "searching" });
    setPreview(null);
    findCollectionPhotos(logRef(log.source), nextSearchNumber(), diveKey.split(","))
      .then((period) => {
        if (!live) return;
        // Same dives found again (link restarted): the user's settings stay.
        const keep = searchedFor.current === diveKey;
        searchedFor.current = diveKey;
        const keys = new Set(period.cameras.map((c) => c.key));
        const kept = <T,>(set: Set<T>) => new Set([...set].filter((k) => keys.has(k as string)));
        setOffsets((o) => (keep ? { ...initialOffsets(period.cameras), ...pick(o, keys) } : initialOffsets(period.cameras)));
        setIgnored((s) => (keep ? kept(s) : new Set()));
        setTouched((s) => (keep ? kept(s) : new Set()));
        if (!keep) setNotes({});
        setSearch({ state: "ready", period });
      })
      .catch((e: CollectionsError) => live && setSearch({ state: "failed", message: failure(e) }));
    return () => {
      live = false;
    };
  }, [diveKey, lr.connected, again]);

  const choices: CollectionChoices = useMemo(
    () => ({ offsets, ignored: [...ignored], marginMin: margin }),
    [offsets, ignored, margin],
  );

  // Live counts: the latest answer wins.
  const asked = useRef(0);
  useEffect(() => {
    if (search.state !== "ready") return;
    const n = ++asked.current;
    collectionsPreview(search.period.search, choices)
      .then((p) => n === asked.current && setPreview(p))
      .catch(() => n === asked.current && setPreview(null));
  }, [choices, search]);

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

  const period = search.state === "ready" ? search.period : null;
  const cameras = period?.cameras ?? [];

  const setOffset = (key: string, min: number) => {
    setOffsets((o) => ({ ...o, [key]: clampOffset(min) }));
    setTouched((s) => new Set(s).add(key));
    setNotes((n) => ({ ...n, [key]: "" }));
  };

  const setIgnoredCamera = (key: string, on: boolean) => {
    setIgnored((s) => {
      const next = new Set(s);
      if (on) next.add(key);
      else next.delete(key);
      return next;
    });
  };

  const suggest = (keys: string[]) => {
    if (keys.length === 0 || !period) return;
    suggestOffsets(period.search, keys)
      .then((found) => {
        setOffsets((o) => {
          const next = { ...o };
          for (const k of keys) if (found[k] !== null && found[k] !== undefined) next[k] = found[k] as number;
          return next;
        });
        setNotes((n) => {
          const next = { ...n };
          for (const k of keys) next[k] = found[k] === null ? t("coll.noSuggestion") : "";
          return next;
        });
      })
      // Refused once a newer search replaced this one: nothing to suggest.
      .catch(() => {});
  };
  // "Suggest for all" leaves ignored cameras and hand-set offsets alone.
  const suggestAll = () => suggest(cameras.filter((c) => !ignored.has(c.key) && !touched.has(c.key)).map((c) => c.key));

  const apply = () => {
    if (!preview || !period) return;
    const cams = period.cameras;
    setRun({ state: "writing", done: 0, total: preview.total });
    applyCollections(logRef(log.source), period.search, choices)
      .then((result) => setRun({ state: "done", result, choices, cameras: cams }))
      .catch((e: CollectionsError) => {
        setRun({ state: "failed", message: failure(e) });
        if (e.error === "noPhotos" || (e.error === "lightroom" && e.reason === "staleSearch")) setAgain((n) => n + 1);
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

  let blocker: string | null = null;
  if (!lr.connected) blocker = t("coll.disconnected");
  else if (search.state === "searching") blocker = t("coll.searching");
  else if (search.state === "failed") blocker = search.message;
  else if (parentMissing) blocker = t("coll.parentMissing", { path: parent.join(" › ") });
  else if (preview && total === 0) blocker = t("coll.err.nothingToWrite");

  const units = `${log.units.depth === "ft" ? "ft" : "m"}, ${log.units.temp === "f" ? "°F" : "°C"}`;
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
          <button type="button" className="btn btn-g" onClick={() => setAgain((n) => n + 1)}>
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
                <button type="button" className="btn btn-s" onClick={suggestAll} disabled={writing}>
                  <Icon name="spark" />
                  {t("coll.suggestAll")}
                </button>
              )}
            </div>
            {cameras.length === 0 ? (
              <p className="muted">{t("coll.noPhotos")}</p>
            ) : (
              <div className="cams">
                {cameras.map((c) => (
                  <CameraCard
                    key={c.key}
                    camera={c}
                    offset={offsets[c.key] ?? 0}
                    ignored={ignored.has(c.key)}
                    remembered={c.remembered !== null && offsets[c.key] === c.remembered && !touched.has(c.key)}
                    inDives={camCounts.get(c.key)?.inDives ?? 0}
                    outside={camCounts.get(c.key)?.outside ?? 0}
                    note={notes[c.key] || null}
                    onOffset={(m) => setOffset(c.key, m)}
                    onIgnore={(on) => setIgnoredCamera(c.key, on)}
                    onSuggest={() => suggest([c.key])}
                  />
                ))}
              </div>
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
                onClick={() => setMargin((m) => clampMargin(m - MARGIN_STEP_MIN))}
              >
                −
              </button>
              <span className="coll-margin-n">{margin} min</span>
              <button
                type="button"
                className="step"
                aria-label={t("coll.margin.more")}
                disabled={margin >= MAX_MARGIN_MIN}
                onClick={() => setMargin((m) => clampMargin(m + MARGIN_STEP_MIN))}
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
          <div className="selection-text">
            <span className="small">{blocker ?? t("coll.writes", { place })}</span>
            <span className="muted small">
              {t("coll.writesNote", { units, altitude: t(settings.depthToAltitude ? "coll.on" : "coll.off") })} ·{" "}
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
            disabled={blocker !== null || !preview || writing}
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
