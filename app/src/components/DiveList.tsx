// Dive list. Shared by Create Collections, step 1 (several dives can be ticked) and
// Tag Selection (one dive is chosen: mode "single", radio buttons). The
// focused row (shown in the detail card, moved with the arrow keys) is not
// the chosen dive: browsing never changes what gets written.

import { Fragment, useEffect, useRef } from "react";
import { formatDepth, formatDuration, formatMonth, formatRowDate, formatTemp, monthKey } from "../format";
import { t } from "../i18n";
import type { Capabilities, Dive, Units } from "../types";
import { Icon } from "./Icon";
import { Stars } from "./Stars";

interface Props {
  dives: Dive[];
  mode: "multi" | "single";
  /** Month headers (only meaningful when sorted by date). */
  grouped: boolean;
  focusedId: string | null;
  onFocus: (id: string) => void;
  selected?: Set<string>;
  onToggle?: (id: string) => void;
  onToggleAll?: () => void;
  /** Single mode: the chosen dive. */
  chosenId?: string | null;
  /** A dive to point out with a badge (e.g. it covers the Lightroom selection). */
  mark?: { id: string; label: string } | null;
  onChoose?: (id: string) => void;
  capabilities: Capabilities;
  units: Units;
  /** Name of the dive application the dives come from. */
  source: string;
}

export function DiveList({ dives, mode, grouped, focusedId, onFocus, selected, onToggle, onToggleAll, chosenId, onChoose, mark, capabilities, units, source }: Props) {
  const multi = mode === "multi";
  const bodyRef = useRef<HTMLTableSectionElement>(null);

  useEffect(() => {
    if (!focusedId) return;
    bodyRef.current
      ?.querySelector(`[data-id="${CSS.escape(focusedId)}"]`)
      ?.scrollIntoView({ block: "nearest" });
  }, [focusedId]);

  const onKeyDown = (e: React.KeyboardEvent) => {
    if ((e.target as HTMLElement).tagName === "INPUT") return;
    const i = dives.findIndex((d) => d.id === focusedId);
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      const next = dives[Math.min(dives.length - 1, Math.max(0, i + (e.key === "ArrowDown" ? 1 : -1)))];
      if (next) onFocus(next.id);
    } else if (e.key === " " && focusedId) {
      e.preventDefault();
      if (multi) onToggle?.(focusedId);
      else onChoose?.(focusedId);
    }
  };

  const shownSelected = multi && selected ? dives.filter((d) => selected.has(d.id)).length : 0;
  const allChecked = dives.length > 0 && shownSelected === dives.length;
  const someChecked = shownSelected > 0 && !allChecked;
  const columns = 8 + (capabilities.temperature ? 1 : 0) + (capabilities.rating ? 1 : 0) + (capabilities.profile ? 1 : 0);

  let lastMonth = "";
  return (
    <div className="card list-card" tabIndex={0} onKeyDown={onKeyDown}>
      <table className="tbl">
        <thead>
          <tr>
            {!multi && (
              <th className="col-check">
                <span className="sr-only">{t("list.choice")}</span>
              </th>
            )}
            {multi && (
              <th className="col-check">
                <input
                  type="checkbox"
                  aria-label={t("list.selectAll")}
                  checked={allChecked}
                  ref={(el) => {
                    if (el) el.indeterminate = someChecked;
                  }}
                  onChange={onToggleAll}
                />
              </th>
            )}
            <th className="num">{t("col.number")}</th>
            <th>{t("col.date")}</th>
            <th>{t("col.site")}</th>
            <th>{t("col.country")}</th>
            <th className="num">{t("col.duration")}</th>
            <th className="num">{t("col.depth")}</th>
            {capabilities.temperature && <th className="num">{t("col.temp")}</th>}
            {capabilities.rating && <th>{t("col.rating")}</th>}
            {capabilities.profile && <th>{t("col.profile")}</th>}
          </tr>
        </thead>
        <tbody ref={bodyRef}>
          {dives.map((d) => {
            const month = monthKey(d.start);
            const header = grouped && month !== lastMonth;
            lastMonth = month;
            const isSelected = multi ? !!selected?.has(d.id) : d.id === chosenId;
            const cls = [d.id === focusedId ? "foc" : "", isSelected ? "sel" : ""].join(" ");
            return (
              <Fragment key={d.id}>
                {header && (
                  <tr className="grp">
                    <td colSpan={columns}>{formatMonth(d.start)}</td>
                  </tr>
                )}
                <tr
                  className={cls}
                  data-id={d.id}
                  onClick={() => {
                    onFocus(d.id);
                    if (!multi) onChoose?.(d.id);
                  }}
                  aria-selected={d.id === focusedId}
                >
                  {!multi && (
                    <td className="col-check">
                      <input
                        type="radio"
                        name="dive-choice"
                        checked={isSelected}
                        onChange={() => onChoose?.(d.id)}
                        aria-label={t("list.choose", { n: d.number ?? "" })}
                      />
                    </td>
                  )}
                  {multi && (
                    <td className="col-check" onClick={(e) => e.stopPropagation()}>
                      <input
                        type="checkbox"
                        checked={isSelected}
                        onChange={() => onToggle?.(d.id)}
                        aria-label={t("list.select", { n: d.number ?? "" })}
                      />
                    </td>
                  )}
                  <td className="num">{d.number ?? ""}</td>
                  <td>{formatRowDate(d.start)}</td>
                  <td className="site-cell">
                    <span className="site-name">{d.site.name ?? t("detail.unknownSite")}</span>
                    {d.site.location && <span className="muted"> · {d.site.location}</span>}
                    {d.invalid && (
                      <span className="badge badge-invalid" title={t("list.invalidHint", { source })}>
                        {t("list.invalid")}
                      </span>
                    )}
                    {mark?.id === d.id && <span className="badge">{mark.label}</span>}
                  </td>
                  <td>{d.site.country ?? ""}</td>
                  <td className="num">{formatDuration(d.durationS)}</td>
                  <td className="num">{formatDepth(d.maxDepthM, units)}</td>
                  {capabilities.temperature && <td className="num">{formatTemp(d.tempLowC ?? d.tempHighC, units)}</td>}
                  {capabilities.rating && (
                    <td>
                      <Stars rating={d.rating} />
                    </td>
                  )}
                  {capabilities.profile && (
                    <td>
                      {d.hasProfile ? (
                        <Icon name="curve" className="accent" label={t("list.hasProfile")} />
                      ) : (
                        <Icon name="noCurve" className="dim" label={t("list.noProfile")} />
                      )}
                    </td>
                  )}
                </tr>
              </Fragment>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
