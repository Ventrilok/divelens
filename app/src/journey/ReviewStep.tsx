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
import { applyCollections, onApplyProgress, tagSelection } from "../api";
import { applyFailure } from "../apply/failure";
import { ProgressBar } from "../apply/ProgressBar";
import { logRef } from "../diveLog";
import { t, tn } from "../i18n";
import { Icon } from "../ui/Icon";
import type { ApplyError, DiveLog, LightroomStatus, Settings } from "../types";
import {
  applyPlan,
  catalogApplied,
  knowLines,
  perPhotoSent,
  readAgain,
  reviewBar,
  selectionApplied,
  staleAnswer,
} from "./review";
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
  parentMissing,
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
  /** The collection set chosen in Settings is no longer in the catalog (spec F2). */
  parentMissing: boolean;
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

  // The collection set chosen in Settings, looked up by the journey (useParentMissing).
  const parent = settings.collections.parent;

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

  // The journey's run now, and whether the step is still shown: Lightroom's answer to
  // an earlier run, or once the step is gone, changes nothing (the latest run wins).
  const runNow = useRef(journey.run);
  runNow.current = journey.run;
  const mounted = useRef(false);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);

  const input: ReviewInput = { journey, lr, log, parentMissing, parent, catalog, selection };
  // What a click writes; null exactly when Apply is greyed.
  const plan = applyPlan(input, writing);

  const begin = (total: number) => {
    busy.current = true;
    setRun({ state: "writing", done: 0, total });
  };
  /**
   * Lightroom answered (`error` null: written). An answer no longer wanted only lets
   * Apply be clicked again; otherwise the step shows it, and what is read again after
   * it is read again, as before.
   */
  const answered = (runAtClick: number, perPhoto: boolean, error: ApplyError | null, show: () => void) => {
    busy.current = false;
    if (staleAnswer(runAtClick, runNow.current, mounted.current)) {
      // Still shown for another run: the write it waited for is over.
      if (mounted.current) setRun({ state: "idle" });
      return;
    }
    show();
    const again = readAgain(path, perPhoto, error);
    if (again.searchAgain) catalog.searchAgain();
    if (again.describeAgain) selection.describeAgain();
    if (again.clearTouched) selection.clearTouched();
  };

  // The success and failure handlers are apart: whatever the summary does once shown
  // is never taken for a refused write.
  const apply = () => {
    if (busy.current || plan === null) return;
    const runAtClick = journey.run;
    if (plan.path === "catalog") {
      // The search shown, its counts and the choices of this click: the summary tells them.
      const { period, preview } = plan;
      const choices = catalog.choices;
      begin(preview.total);
      applyCollections(logRef(log.source), period.search, choices, writeOptions(journey), otherSite(journey)).then(
        (result) =>
          answered(runAtClick, false, null, () => {
            setRun({ state: "idle" });
            onApplied(catalogApplied(result, preview, period, choices));
          }),
        (e: ApplyError) =>
          answered(runAtClick, false, e, () => setRun({ state: "failed", message: applyFailure(e, "collections") })),
      );
      return;
    }
    // The description the blocker checked, and its counts: the summary tells its photos outside the dive.
    const choice = tagChoice({ journey, dive: chosenDive(journey, log.dives), selection, offsets: selection.offsets });
    const counts = selection.freshCounts;
    const perPhoto = perPhotoSent(choice);
    begin(lr.selected);
    tagSelection(logRef(log.source), choice).then(
      (result) =>
        answered(runAtClick, perPhoto, null, () => {
          setRun({ state: "idle" });
          onApplied(selectionApplied(result, choice, counts));
        }),
      (e: ApplyError) => answered(runAtClick, perPhoto, e, () => setRun({ state: "failed", message: applyFailure(e, "tag") })),
    );
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
