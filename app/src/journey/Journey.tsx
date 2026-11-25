// The journey (spec F5): one screen in five steps, Scope › Dives › Clocks ›
// Review & Apply › Summary, on the path chosen at step 1. It holds the
// journey's one state and the two photo readings, mounted once, so that what
// the diver chose and set survives going back and forth between steps, and
// Settings: the app keeps the journey mounted, hidden, while Settings show or
// the dive log is read again.
//
// While Apply writes in Lightroom, nothing leaves step 4: the strip leads
// nowhere, "Open Dive Lens…" is dropped (as before), and the app keeps the
// gear and the Lightroom menus still.

import { useEffect, useReducer, useRef, useState } from "react";
import { refreshCatalogSize } from "../api";
import type { Sort } from "../dives/filters";
import { Steps } from "../ui/Steps";
import type { DiveLog, LightroomStatus, Settings } from "../types";
import { ClocksStep } from "./ClocksStep";
import { DivesStep } from "./DivesStep";
import { PhotosStep } from "./PhotosStep";
import { ReviewStep } from "./ReviewStep";
import type { Applied } from "./rules";
import { appliedFor, looksUpSetAgain, readingDive, searchedDives, selectionReads, stripLeadsBack, type Written } from "./shell";
import { reduce, start, stepStates, type Step } from "./state";
import { SummaryStep } from "./SummaryStep";
import { useCatalogPhotos } from "./useCatalogPhotos";
import { useParentMissing } from "./useCollectionSet";
import { useSelectionPhotos } from "./useSelectionPhotos";

export function Journey({
  log,
  lr,
  settings,
  sort,
  onSort,
  notice,
  onDismissNotice,
  onOpenSettings,
  onWriting,
  menu,
  logChanges,
  forgotten,
}: {
  /** The last dive log read: a new one (same log read again) drops the dives it lost. */
  log: DiveLog;
  lr: LightroomStatus;
  settings: Settings;
  /** The dive list's order, remembered between launches. */
  sort: Sort;
  onSort: (s: Sort) => void;
  /** Why the journey started again (the dive log changed): shown at step 1, cleared once it is left. */
  notice: string | null;
  onDismissNotice: () => void;
  onOpenSettings: () => void;
  /** Told when Apply starts and stops writing in Lightroom. */
  onWriting: (writing: boolean) => void;
  /** "Open Dive Lens…" chosen in Lightroom, counted: each one brings step 1. */
  menu: number;
  /** Another dive log read (application or location), counted: each one starts the journey again. */
  logChanges: number;
  /** The cameras whose remembered clock offset Settings forgot, in order: no longer shown as remembered. */
  forgotten: string[];
}) {
  const [j, dispatch] = useReducer(reduce, start);
  const [writing, setWriting] = useState(false);
  const [written, setWritten] = useState<Written | null>(null);

  // What the app tells, handled before anything of this render is shown, so that
  // the photo readings never look for the dives of a log no longer read.
  const [seen, setSeen] = useState({ menu, logChanges, log });
  if (seen.menu !== menu || seen.logChanges !== logChanges || seen.log !== log) {
    setSeen({ menu, logChanges, log });
    // One dive log at a time (spec F2): nothing of the previous one stays chosen.
    if (seen.logChanges !== logChanges) dispatch({ type: "logChanged" });
    // The same log read again: what it lost is no longer chosen (nothing changes otherwise).
    if (seen.log !== log) dispatch({ type: "logRead", ids: log.dives.map((d) => d.id) });
    // While Lightroom is written to, no menu leaves the screen: dropped, as before.
    if (seen.menu !== menu && !writing) dispatch({ type: "menu" });
  }

  // The two photo readings, each mounted once for the whole journey.
  const catalog = useCatalogPhotos(log, lr, searchedDives(j), j.run, forgotten);
  const selection = useSelectionPhotos(log, lr, readingDive(j, log.dives), selectionReads(j), j.run, forgotten);
  // From step 2 on, so that step 4 opens knowing whether the collection set is still there;
  // again on each arrival at step 3 from step 2, counted.
  const [collectionSetRound, setCollectionSetRound] = useState(0);
  const parentMissing = useParentMissing(lr, settings.collections.parent, j.step >= 2 && j.step <= 4, collectionSetRound);

  // Leaving step 1 clears the notice of a changed dive log; landing on it counts the catalog again (spec F1).
  // Step 3 reached from step 2 looks the collection set up again.
  const stepBefore = useRef(j.step);
  useEffect(() => {
    const was = stepBefore.current;
    stepBefore.current = j.step;
    if (was === j.step) return;
    if (was === 1) onDismissNotice();
    if (j.step === 1) refreshCatalogSize().catch(() => {});
    if (looksUpSetAgain(was, j.step)) setCollectionSetRound((n) => n + 1);
  }, [j.step]);

  const runNow = useRef(j.run);
  runNow.current = j.run;
  const onApplied = (applied: Applied) => {
    setWritten({ run: runNow.current, applied });
    dispatch({ type: "applied" });
  };
  const tellWriting = (on: boolean) => {
    setWriting(on);
    onWriting(on);
  };

  const path = j.path;
  const applied = appliedFor(written, j);
  const onGo = stripLeadsBack(j, writing) ? (step: Step) => dispatch({ type: "goTo", step }) : undefined;

  return (
    <main className="screen">
      <Steps states={stepStates(j)} onGo={onGo} />
      {j.step === 1 && <PhotosStep journey={j} dispatch={dispatch} lr={lr} notice={notice} onDismissNotice={onDismissNotice} />}
      {j.step === 2 && path && (
        <DivesStep
          path={path}
          journey={j}
          dispatch={dispatch}
          log={log}
          lr={lr}
          sort={sort}
          onSort={onSort}
          selection={selection}
        />
      )}
      {j.step === 3 && path && (
        <ClocksStep
          path={path}
          journey={j}
          catalog={catalog}
          selection={selection}
          log={log}
          lr={lr}
          onBack={() => dispatch({ type: "back" })}
          onNext={() => dispatch({ type: "next" })}
        />
      )}
      {j.step === 4 && path && (
        <ReviewStep
          path={path}
          journey={j}
          dispatch={dispatch}
          catalog={catalog}
          selection={selection}
          log={log}
          lr={lr}
          settings={settings}
          parentMissing={parentMissing}
          onOpenSettings={onOpenSettings}
          onApplied={onApplied}
          onWriting={tellWriting}
        />
      )}
      {j.step === 5 && applied && (
        <SummaryStep
          applied={applied}
          journey={j}
          log={log}
          settings={settings}
          onHandleSeparately={(ids) => dispatch({ type: "handleSeparately", ids })}
          onDone={() => dispatch({ type: "done" })}
        />
      )}
    </main>
  );
}
