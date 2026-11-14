// Create Collections, step 1: browse, filter and check the dives, then Next.

import { useDeferredValue, useEffect, useMemo, useState } from "react";
import { DiveDetail } from "../components/DiveDetail";
import { DiveList } from "../components/DiveList";
import { FilterBar } from "../components/FilterBar";
import { Icon } from "../components/Icon";
import { LogNotice } from "../components/LogNotice";
import { Steps } from "../components/Steps";
import { formatRange } from "../format";
import { applyFilters, hasFilters, NO_FILTERS, searchIndex, sortDives, type Filters, type Sort } from "../filters";
import { formatInt, t, tn } from "../i18n";
import { sourceName } from "../sourceText";
import type { DiveLog } from "../types";

export function DivesScreen({
  log,
  selected,
  onSelect: setSelected,
  sort,
  onSort: setSort,
  notice,
  onDismissNotice,
  onNext,
}: {
  log: DiveLog;
  selected: Set<string>;
  onSelect: (s: Set<string>) => void;
  /** Remembered between launches (Settings). */
  sort: Sort;
  onSort: (s: Sort) => void;
  /** Why the flow started again (the dive log changed), shown once. */
  notice: string | null;
  onDismissNotice: () => void;
  onNext: () => void;
}) {
  const units = log.units;
  const { dives, capabilities } = log;
  const [filters, setFilters] = useState<Filters>(NO_FILTERS);
  const [focusedId, setFocusedId] = useState<string | null>(null);

  // Typing stays fluid: the list catches up with the filters just after.
  const deferredFilters = useDeferredValue(filters);
  const index = useMemo(() => searchIndex(dives), [dives]);
  const shown = useMemo(
    () => sortDives(applyFilters(dives, deferredFilters, index), sort),
    [dives, deferredFilters, index, sort],
  );

  // Keep the detail on a visible dive.
  useEffect(() => {
    if (!shown.some((d) => d.id === focusedId)) setFocusedId(shown[0]?.id ?? null);
  }, [shown, focusedId]);

  const focused = dives.find((d) => d.id === focusedId) ?? null;
  const withProfile = useMemo(() => dives.filter((d) => d.hasProfile).length, [dives]);

  const toggle = (id: string) => {
    const next = new Set(selected);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    setSelected(next);
  };

  const toggleAll = () => {
    const next = new Set(selected);
    const all = shown.every((d) => next.has(d.id));
    for (const d of shown) {
      if (all) next.delete(d.id);
      else next.add(d.id);
    }
    setSelected(next);
  };

  const selectedDives = useMemo(() => dives.filter((d) => selected.has(d.id)), [dives, selected]);
  const shownIds = useMemo(() => new Set(shown.map((d) => d.id)), [shown]);
  const hidden = selectedDives.filter((d) => !shownIds.has(d.id)).length;
  const span =
    selectedDives.length > 0
      ? formatRange(
          selectedDives.reduce((a, d) => (d.start < a ? d.start : a), selectedDives[0].start),
          selectedDives.reduce((a, d) => (d.end > a ? d.end : a), selectedDives[0].end),
        )
      : "";

  return (
    <main className="screen">
      <Steps current={1} />
      <header className="screen-head">
        <div>
          <p className="muted">
            {sourceName(log.source.kind)} · {tn("source.dives", dives.length)}
            {capabilities.profile && <> · {t("dives.withProfile", { n: formatInt(withProfile) })}</>}
          </p>
        </div>
        <label className="sort">
          {t("sort.label")}
          <select className="inp" value={sort} onChange={(e) => setSort(e.target.value as Sort)}>
            <option value="recent">{t("sort.recent")}</option>
            <option value="oldest">{t("sort.oldest")}</option>
            <option value="depth">{t("sort.depth")}</option>
          </select>
        </label>
      </header>
      {notice && <LogNotice text={notice} onDismiss={onDismissNotice} />}

      <FilterBar
        dives={dives}
        filters={filters}
        onChange={setFilters}
        capabilities={capabilities}
        units={units}
        shown={shown.length}
      />

      <div className="dives-body">
        {shown.length > 0 ? (
          <DiveList
            dives={shown}
            mode="multi"
            grouped={sort !== "depth"}
            focusedId={focusedId}
            onFocus={setFocusedId}
            selected={selected}
            onToggle={toggle}
            onToggleAll={toggleAll}
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
        <DiveDetail log={log.source} dive={focused} units={units} capabilities={capabilities} />
      </div>

      <div className="card selection-bar">
        {selectedDives.length > 0 ? (
          <div className="selection-text">
            <span className="strong">
              {tn("sel.count", selectedDives.length)}
              {hidden > 0 && <span className="muted"> ({tn("sel.hidden", hidden)})</span>}
            </span>
            <span className="muted small">{span}</span>
          </div>
        ) : (
          <span className="muted">{t("sel.none")}</span>
        )}
        <div className="selection-actions">
          {selectedDives.length > 0 && (
            <button type="button" className="btn btn-s" onClick={() => setSelected(new Set())}>
              {t("sel.clear")}
            </button>
          )}
          <button type="button" className="btn btn-p" onClick={onNext} disabled={selectedDives.length === 0}>
            {t("sel.next")}
            <Icon name="arrow" />
          </button>
        </div>
      </div>
    </main>
  );
}
