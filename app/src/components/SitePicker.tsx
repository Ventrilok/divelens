// "Other site" picker of Tag Selection: country › location › site, then the
// site's GPS, which the user may edit (as in the previous plugin).

import { useMemo } from "react";
import { t } from "../i18n";
import { siteTree } from "../sites";
import type { Site } from "../types";

export interface SitePick {
  country: string | null | undefined;
  location: string | null | undefined;
  siteId: string | null;
  lat: string;
  lon: string;
}

export const NO_PICK: SitePick = { country: undefined, location: undefined, siteId: null, lat: "", lon: "" };

/** Picker state opened on a site's country and location (or empty). */
export function pickAround(site: Site | null): SitePick {
  if (!site || site.id === null) return NO_PICK;
  return { ...NO_PICK, country: site.country, location: site.location };
}

function coord(v: number | null): string {
  return v === null ? "" : String(v);
}

function Column({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="pick-col" role="group" aria-label={label}>
      <div className="lbl pick-head">{label}</div>
      <div className="pick-items">{children}</div>
    </div>
  );
}

function Item({ on, onClick, name, count }: { on: boolean; onClick: () => void; name: string; count?: number }) {
  return (
    <button type="button" className={`pick-it ${on ? "pick-on" : ""}`} aria-pressed={on} onClick={onClick}>
      <span className="pick-name">{name}</span>
      {count !== undefined && <span className="cnt">{count}</span>}
    </button>
  );
}

export function SitePicker({
  sites,
  pick,
  onChange,
  gpsError,
}: {
  sites: Site[];
  pick: SitePick;
  onChange: (p: SitePick) => void;
  gpsError: boolean;
}) {
  const tree = useMemo(() => siteTree(sites), [sites]);
  const country = tree.find((c) => c.name === pick.country);
  const location = country?.locations.find((l) => l.name === pick.location);

  return (
    <div className="pick">
      <div className="pick-cols">
        <Column label={t("tag.country")}>
          {tree.map((c) => (
            <Item
              key={c.name ?? ""}
              on={c === country}
              name={c.name ?? t("tag.noCountry")}
              count={c.count}
              onClick={() => onChange({ ...NO_PICK, country: c.name })}
            />
          ))}
        </Column>
        <Column label={t("tag.region")}>
          {country?.locations.map((l) => (
            <Item
              key={l.name ?? ""}
              on={l === location}
              name={l.name ?? t("tag.noRegion")}
              count={l.sites.length}
              onClick={() => onChange({ ...NO_PICK, country: pick.country, location: l.name })}
            />
          ))}
        </Column>
        <Column label={t("tag.site")}>
          {location?.sites.map((s) => (
            <Item
              key={s.id}
              on={s.id === pick.siteId}
              name={s.name ?? ""}
              onClick={() => onChange({ ...pick, siteId: s.id, lat: coord(s.lat), lon: coord(s.lon) })}
            />
          ))}
        </Column>
      </div>
      <div className="pick-gps">
        <label>
          {t("tag.lat")}
          <input
            className="inp"
            inputMode="decimal"
            value={pick.lat}
            disabled={!pick.siteId}
            aria-invalid={gpsError}
            onChange={(e) => onChange({ ...pick, lat: e.target.value })}
          />
        </label>
        <label>
          {t("tag.lon")}
          <input
            className="inp"
            inputMode="decimal"
            value={pick.lon}
            disabled={!pick.siteId}
            aria-invalid={gpsError}
            onChange={(e) => onChange({ ...pick, lon: e.target.value })}
          />
        </label>
        {gpsError && (
          <p className="field-error" role="alert">
            {t("tag.badGps")}
          </p>
        )}
      </div>
    </div>
  );
}
