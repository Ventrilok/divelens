// Step 2 of the journey, "Dives" (spec F4, F5), one screen for both paths:
// the dive list with its search, filters, sort and detail card. Catalog path:
// several dives are ticked, each photo going to the dive of its time.
// Selection path: one dive is chosen, every photo selected being attached to
// it, or "No dive — location only"; the dive whose times cover the selection
// is pointed out, never chosen for the diver.
//
// Rendered inside the journey's screen, under the step strip: it brings the
// line "Photos: …", the list and the bottom bar ("Back" returns to step 1).

import { useEffect, useMemo } from "react";
import { DiveDetail } from "../dives/DiveDetail";
import { DiveList } from "../dives/DiveList";
import { EmptyList, EmptyMessage } from "../dives/EmptyList";
import { FilterBar } from "../dives/FilterBar";
import { useDiveBrowser } from "../dives/useDiveBrowser";
import type { Sort } from "../dives/filters";
import { t, tn } from "../i18n";
import { sourceName } from "../sourceText";
import { Icon } from "../ui/Icon";
import type { DiveLog, LightroomStatus } from "../types";
import { catalogDivesBar, coversNote, selectionDivesBar, selectionHead } from "./choose";
import { canGoNext } from "./rules";
import type { Action, Journey, Path } from "./state";
import { photosLine } from "./texts";
import type { SelectionReading } from "./useSelectionPhotos";

export function DivesStep({
  path,
  journey,
  dispatch,
  log,
  lr,
  sort,
  onSort,
  selection,
}: {
  path: Path;
  journey: Journey;
  dispatch: (a: Action) => void;
  log: DiveLog;
  lr: LightroomStatus;
  /** Remembered between launches (Settings), the same on both paths. */
  sort: Sort;
  onSort: (s: Sort) => void;
  /** The selection path's reading: the dive covering the selection, and what explains it. */
  selection: SelectionReading;
}) {
  const { dives, capabilities, units } = log;
  const { filters, setFilters, shown, focusedId, setFocusedId, focused } = useDiveBrowser(dives, sort);
  const shownIds = useMemo(() => new Set(shown.map((d) => d.id)), [shown]);
  const multi = path === "catalog";
  const pick = journey.dive;
  const chosenId = pick.kind === "dive" ? pick.id : null;
  const covering = multi ? null : selection.covering;

  // Back from a later step, the detail card shows the dive chosen.
  useEffect(() => {
    if (chosenId !== null) setFocusedId(chosenId);
  }, []);
  // While no dive is chosen, the detail card follows the dive covering the
  // selection; once one is chosen, it stays where it is.
  useEffect(() => {
    if (covering && pick.kind === "none") setFocusedId(covering.id);
  }, [covering]);

  const toggleAll = () => {
    const next = new Set(journey.dives);
    const all = shown.every((d) => next.has(d.id));
    for (const d of shown) {
      if (all) next.delete(d.id);
      else next.add(d.id);
    }
    dispatch({ type: "setDives", ids: [...next] });
  };

  const line = photosLine(journey, { lr, dives, period: null });
  const next = canGoNext(journey, lr);
  const source = sourceName(log.source.kind);
  const note = covering && coversNote(covering, selection.described?.photos ?? null, selection.offsets, selection.touched);
  const catalogBar = multi ? catalogDivesBar(journey, dives, shownIds) : null;
  const selectionBar = multi ? null : selectionDivesBar(journey, dives, lr);
  const prompt = (catalogBar ?? selectionBar)?.prompt ?? null;

  return (
    <>
      <header className="screen-head">
        <div className="dives-head">
          {line && <p className="journey-line">{line}</p>}
          {!multi && <p className="muted small">{selectionHead(lr)}</p>}
        </div>
        <label className="sort">
          {t("sort.label")}
          <select className="inp" value={sort} onChange={(e) => onSort(e.target.value as Sort)}>
            <option value="recent">{t("sort.recent")}</option>
            <option value="oldest">{t("sort.oldest")}</option>
            <option value="depth">{t("sort.depth")}</option>
          </select>
        </label>
      </header>

      <FilterBar
        dives={dives}
        filters={filters}
        onChange={setFilters}
        capabilities={capabilities}
        units={units}
        shown={shown.length}
        count={multi ? undefined : tn("journey.dives.oneToChoose", shown.length)}
      />

      <div className="dives-body">
        {/* Selection path: "No dive — location only" stays, above the message, when no dive is listed. */}
        {shown.length > 0 || !multi ? (
          <DiveList
            dives={shown}
            mode={multi ? "multi" : "single"}
            grouped={sort !== "depth"}
            focusedId={focusedId}
            onFocus={setFocusedId}
            selected={journey.dives}
            onToggle={(id) => dispatch({ type: "toggleDive", id })}
            onToggleAll={toggleAll}
            chosenId={chosenId}
            onChoose={(id) => dispatch({ type: "pickDive", pick: { kind: "dive", id } })}
            mark={covering ? { id: covering.id, label: t("tag.covers"), note } : null}
            noDive={
              multi
                ? null
                : {
                    chosen: pick.kind === "noDive",
                    label: t("journey.dives.noDive"),
                    hint: t("journey.dives.noDiveHint"),
                    onChoose: () => dispatch({ type: "pickDive", pick: { kind: "noDive" } }),
                  }
            }
            empty={<EmptyMessage filters={filters} onChange={setFilters} />}
            capabilities={capabilities}
            units={units}
            source={source}
          />
        ) : (
          <EmptyList filters={filters} onChange={setFilters} />
        )}
        <DiveDetail log={log.source} dive={focused} units={units} capabilities={capabilities} />
      </div>

      <div className="card selection-bar">
        <div className="selection-text">
          {catalogBar?.count && (
            <span className="strong">
              {catalogBar.count}
              {catalogBar.hidden && <span className="muted"> ({catalogBar.hidden})</span>}
            </span>
          )}
          {catalogBar?.span && <span className="muted small">{catalogBar.span}</span>}
          {selectionBar?.choice && <span className="strong">{selectionBar.choice}</span>}
          {selectionBar?.sub && <span className="muted small">{selectionBar.sub}</span>}
          {prompt && <span className="muted">{prompt}</span>}
        </div>
        <div className="selection-actions">
          {multi && journey.dives.size > 0 && (
            <button type="button" className="btn btn-s" onClick={() => dispatch({ type: "setDives", ids: [] })}>
              {t("sel.clear")}
            </button>
          )}
          <button type="button" className="btn btn-s" onClick={() => dispatch({ type: "back" })}>
            <Icon name="back" />
            {t("coll.back")}
          </button>
          <button type="button" className="btn btn-p" disabled={!next.allowed} onClick={() => dispatch({ type: "next" })}>
            {t("sel.next")}
            <Icon name="arrow" />
          </button>
        </div>
      </div>
    </>
  );
}
