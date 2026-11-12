// Search and filter chips above the dive list.

import { useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { countryCounts, type Filters } from "../filters";
import { depthToMetres, depthUnit, depthValue, parseWallClock } from "../format";
import { formatInt, localized, t } from "../i18n";
import type { Capabilities, Dive, Units } from "../types";
import { Icon } from "./Icon";

interface ChipProps {
  label: string;
  /** Filter currently active: chip highlighted, with a remove cross. */
  active: boolean;
  onClear: () => void;
  children: ReactNode;
}

/** A filter chip opening a small panel. Closes on outside click or Escape. */
function Chip({ label, active, onClear, children }: ChipProps) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const onDown = (e: MouseEvent) => {
      if (!ref.current?.contains(e.target as Node)) setOpen(false);
    };
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && setOpen(false);
    document.addEventListener("mousedown", onDown);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("mousedown", onDown);
      document.removeEventListener("keydown", onKey);
    };
  }, [open]);

  return (
    <div className="chip-wrap" ref={ref}>
      <div className={`chip ${active ? "chip-on" : ""}`}>
        <button type="button" className="chip-main" aria-expanded={open} onClick={() => setOpen(!open)}>
          {label}
          {!active && <Icon name="chevron" size={14} />}
        </button>
        {active && (
          <button type="button" className="chip-x" aria-label={t("filter.remove", { name: label })} onClick={() => {
              setOpen(false);
              onClear();
            }}>
            <Icon name="close" size={14} />
          </button>
        )}
      </div>
      {open && <div className="popover card">{children}</div>}
    </div>
  );
}

const shortDate = localized((l) => new Intl.DateTimeFormat(l, { timeZone: "UTC", day: "numeric", month: "short", year: "numeric" }));
const fmtDay = (d: string) => shortDate.format(parseWallClock(d));

function periodLabel(f: Filters): string {
  if (f.from && f.to) return `${fmtDay(f.from)} → ${fmtDay(f.to)}`;
  if (f.from) return `${t("filter.from")} ${fmtDay(f.from)}`;
  if (f.to) return `${t("filter.to")} ${fmtDay(f.to)}`;
  return t("filter.period");
}

interface Props {
  dives: Dive[];
  filters: Filters;
  onChange: (f: Filters) => void;
  capabilities: Capabilities;
  units: Units;
  shown: number;
}

export function FilterBar({ dives, filters: f, onChange, capabilities, units, shown }: Props) {
  const set = (patch: Partial<Filters>) => onChange({ ...f, ...patch });
  const countries = useMemo(() => countryCounts(dives), [dives]);
  const countryName = (c: string) => c || t("filter.noCountry");
  const unit = depthUnit(units);
  const roundDepth = (m: number | null) => (m === null ? "" : String(Math.round(depthValue(m, units))));
  const readDepth = (v: string) => (v.trim() === "" || isNaN(Number(v)) ? null : depthToMetres(Number(v), units));

  const countryLabel =
    f.countries.length === 0
      ? t("filter.country")
      : f.countries.length === 1
        ? `${t("filter.country")} : ${countryName(f.countries[0])}`
        : t("filter.countries", { n: f.countries.length });

  const depthLabel =
    f.depthMin === null && f.depthMax === null
      ? t("filter.depth")
      : f.depthMax === null
        ? `≥ ${roundDepth(f.depthMin)} ${unit}`
        : f.depthMin === null
          ? `≤ ${roundDepth(f.depthMax)} ${unit}`
          : `${roundDepth(f.depthMin)}–${roundDepth(f.depthMax)} ${unit}`;

  return (
    <div className="filter-bar">
      <div className="search">
        <Icon name="search" className="search-icon" />
        <input
          className="inp"
          type="search"
          aria-label={t("filter.searchLabel")}
          placeholder={t("filter.search")}
          value={f.query}
          onChange={(e) => set({ query: e.target.value })}
        />
      </div>

      <Chip label={periodLabel(f)} active={!!(f.from || f.to)} onClear={() => set({ from: "", to: "" })}>
        <div className="form-row">
          <label>
            {t("filter.from")}
            <input className="inp" type="date" value={f.from} onChange={(e) => set({ from: e.target.value })} />
          </label>
          <label>
            {t("filter.to")}
            <input className="inp" type="date" value={f.to} onChange={(e) => set({ to: e.target.value })} />
          </label>
        </div>
      </Chip>

      <Chip label={countryLabel} active={f.countries.length > 0} onClear={() => set({ countries: [] })}>
        <div className="option-list">
          {countries.map(([c, n]) => (
            <label key={c} className="option">
              <input
                type="checkbox"
                checked={f.countries.includes(c)}
                onChange={(e) =>
                  set({ countries: e.target.checked ? [...f.countries, c] : f.countries.filter((x) => x !== c) })
                }
              />
              <span className="option-name">{countryName(c)}</span>
              <span className="cnt">{formatInt(n)}</span>
            </label>
          ))}
        </div>
      </Chip>

      <Chip label={depthLabel} active={f.depthMin !== null || f.depthMax !== null} onClear={() => set({ depthMin: null, depthMax: null })}>
        <div className="form-row">
          <label>
            {t("filter.depthMin")} ({unit})
            <input
              className="inp inp-num"
              type="number"
              min={0}
              defaultValue={roundDepth(f.depthMin)}
              onChange={(e) => set({ depthMin: readDepth(e.target.value) })}
            />
          </label>
          <label>
            {t("filter.depthMax")} ({unit})
            <input
              className="inp inp-num"
              type="number"
              min={0}
              defaultValue={roundDepth(f.depthMax)}
              onChange={(e) => set({ depthMax: readDepth(e.target.value) })}
            />
          </label>
        </div>
      </Chip>

      {capabilities.rating && (
        <Chip
          label={f.minRating ? t("filter.ratingMin", { n: f.minRating }) : t("filter.rating")}
          active={f.minRating !== null}
          onClear={() => set({ minRating: null })}
        >
          <div className="option-list">
            {[null, 1, 2, 3, 4, 5].map((n) => (
              <label key={n ?? 0} className="option">
                <input type="radio" name="rating" checked={f.minRating === n} onChange={() => set({ minRating: n })} />
                <span className="option-name">{n ? t("filter.ratingMin", { n }) : t("filter.ratingAny")}</span>
              </label>
            ))}
          </div>
        </Chip>
      )}

      {capabilities.profile && (
        <button
          type="button"
          className={`chip chip-toggle ${f.withProfile ? "chip-on" : ""}`}
          aria-pressed={f.withProfile}
          onClick={() => set({ withProfile: !f.withProfile })}
        >
          <Icon name="curve" size={15} className="accent" />
          {t("filter.withProfile")}
        </button>
      )}

      <span className="filter-count">{t("dives.shown", { shown: formatInt(shown), total: formatInt(dives.length) })}</span>
    </div>
  );
}
