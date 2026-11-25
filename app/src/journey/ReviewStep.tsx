// Step 4 of the journey, "Review & Apply" (spec F5), one screen for both
// paths: what Dive Lens writes (ReviewOptions), "Good to know before
// applying", then the bar with "Apply to N photos". Apply writes in one
// Lightroom Undo step: the photos found in the dives ticked (catalog path) or
// the photos selected at the moment of the click (selection path), with the
// options ticked. Once written, the summary; refused, the reason here, and
// the photos looked for or the selection described again as before.
//
// Rendered inside the journey's screen, under the step strip: it brings the
// line "Photos: …", its blocks and the bottom bar. While Apply writes, nothing
// here can be clicked, and `onWriting` tells the journey (its menu waits).

import { useEffect, useRef, useState } from "react";
import { applyCollections, collectionSets, onApplyProgress, tagSelection } from "../api";
import { applyFailure } from "../apply/failure";
import { ProgressBar } from "../apply/ProgressBar";
import { setExists } from "../collections/collectionPlace";
import { logRef } from "../diveLog";
import { t, tn } from "../i18n";
import { Icon } from "../ui/Icon";
import type { ApplyError, DiveLog, LightroomStatus, Settings } from "../types";
import { catalogApplied, knowLines, readAgain, reviewBar, selectionApplied } from "./review";
import { ReviewOptions } from "./ReviewOptions";
import {
  applyCount,
  canApply,
  chosenDive,
  otherSite,
  tagChoice,
  unitsText,
  writeOptions,
  type Applied,
  type ReviewInput,
} from "./rules";
import type { Action, Journey, Path } from "./state";
import { photosLine } from "./texts";
import type { CatalogReading } from "./useCatalogPhotos";
import type { SelectionReading } from "./useSelectionPhotos";

type Run = { state: "idle" } | { state: "writing"; done: number; total: number } | { state: "failed"; message: string };

/** A line of "Good to know before applying": in orange for a dive without photo. */
function KnowLine({ text, warn }: { text: string; warn: boolean }) {
  return (
    <div className="review-line">
      <Icon name={warn ? "warn" : "info"} className={warn ? "warn" : "muted"} />
      <span>{text}</span>
    </div>
  );
}

export function ReviewStep({
  path,
  journey,
  dispatch,
  catalog,
  selection,
  log,
  lr,
  settings,
  onOpenSettings,
  onApplied,
  onWriting,
}: {
  path: Path;
  journey: Journey;
  dispatch: (a: Action) => void;
  catalog: CatalogReading;
  selection: SelectionReading;
  log: DiveLog;
  lr: LightroomStatus;
  settings: Settings;
  onOpenSettings: () => void;
  /** Written: the journey shows the summary of `applied` (it dispatches "applied"). */
  onApplied: (applied: Applied) => void;
  /** Told when a write starts and ends, and `false` when the step goes away. */
  onWriting: (writing: boolean) => void;
}) {
  const [run, setRun] = useState<Run>({ state: "idle" });
  const writing = run.state === "writing";
  // Set at the click, before the screen shows the write: a second click does nothing.
  const busy = useRef(false);

  // The collection set chosen in Settings must still be in the catalog (spec F2):
  // checked when Lightroom connects, and again when the place changes.
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

  const tell = useRef(onWriting);
  tell.current = onWriting;
  useEffect(() => {
    tell.current(writing);
  }, [writing]);
  useEffect(
    () => () => {
      tell.current(false);
    },
    [],
  );

  const input: ReviewInput = { journey, lr, log, parentMissing, parent, catalog, selection };

  const begin = (total: number) => {
    busy.current = true;
    setRun({ state: "writing", done: 0, total });
  };
  /** Lightroom answered (`error` null: written): what is read again, as before. */
  const answered = (error: ApplyError | null, perPhoto: boolean) => {
    busy.current = false;
    const again = readAgain(path, perPhoto, error);
    if (again.searchAgain) catalog.searchAgain();
    if (again.describeAgain) selection.describeAgain();
    if (again.clearTouched) selection.clearTouched();
  };

  const apply = () => {
    if (busy.current || !canApply(input, writing)) return;
    if (path === "catalog") {
      // The search shown, its counts and the choices of this click: the summary tells them.
      const { preview, period, choices } = catalog;
      if (!preview || !period) return;
      begin(preview.total);
      applyCollections(logRef(log.source), period.search, choices, writeOptions(journey), otherSite(journey))
        .then((result) => {
          setRun({ state: "idle" });
          answered(null, false);
          onApplied(catalogApplied(result, preview, period, choices));
        })
        .catch((e: ApplyError) => {
          setRun({ state: "failed", message: applyFailure(e, "collections") });
          answered(e, false);
        });
      return;
    }
    // The description the blocker checked, and its counts: the summary tells its photos outside the dive.
    const choice = tagChoice({ journey, dive: chosenDive(journey, log.dives), selection, offsets: selection.offsets });
    const counts = selection.freshCounts;
    const perPhoto = choice.search !== null;
    begin(lr.selected);
    tagSelection(logRef(log.source), choice)
      .then((result) => {
        setRun({ state: "idle" });
        onApplied(selectionApplied(result, choice, counts));
        answered(null, perPhoto);
      })
      .catch((e: ApplyError) => {
        setRun({ state: "failed", message: applyFailure(e, "tag") });
        answered(e, perPhoto);
      });
  };

  const onCatalog = path === "catalog";
  const line = photosLine(journey, { lr, dives: log.dives, period: onCatalog ? catalog.period : null });
  const bar = reviewBar(input);

  return (
    <>
      {line && <p className="journey-line">{line}</p>}
      {!lr.connected && (
        <div className="notice notice-warn" role="status">
          <span>{t(onCatalog ? "coll.disconnected" : "tag.disconnected")}</span>
        </div>
      )}

      <ReviewOptions
        input={input}
        log={log}
        settings={settings}
        writing={writing}
        onOptions={(options) => dispatch({ type: "setOptions", options })}
        onOpenSettings={onOpenSettings}
      />

      <section className="card review-know" aria-labelledby="h-know">
        <h2 id="h-know">{t("journey.know.title")}</h2>
        {knowLines(input).map((l, n) => (
          <KnowLine key={n} text={l.text} warn={l.warn} />
        ))}
        <KnowLine text={unitsText(log.units)} warn={false} />
      </section>

      {run.state === "failed" && (
        <div className="notice notice-warn" role="alert">
          <span>{run.message}</span>
          <button type="button" className="btn btn-g" onClick={() => setRun({ state: "idle" })}>
            {t("tag.dismiss")}
          </button>
        </div>
      )}

      <div className="card selection-bar apply-bar">
        {run.state === "writing" ? (
          <ProgressBar done={run.done} total={run.total} label={t("tag.writing")} />
        ) : (
          <div className="selection-text">
            {bar.text && <span className={bar.blocked ? "muted" : "strong"}>{bar.text}</span>}
            <span className="muted small">{t("coll.undoShort")}</span>
          </div>
        )}
        <div className="selection-actions">
          {parentMissing && writeOptions(journey).collections && !writing && (
            <button type="button" className="btn btn-s" onClick={onOpenSettings}>
              {t("coll.openSettings")}
            </button>
          )}
          <button type="button" className="btn btn-s" onClick={() => dispatch({ type: "back" })} disabled={writing}>
            <Icon name="back" />
            {t("coll.back")}
          </button>
          <button type="button" className="btn btn-p" disabled={!canApply(input, writing)} onClick={apply}>
            {tn("tag.apply", applyCount(input))}
            <Icon name="arrow" />
          </button>
        </div>
      </div>
    </>
  );
}
