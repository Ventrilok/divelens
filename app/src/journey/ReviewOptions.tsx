// Step 4 of the journey: the card "What Dive Lens writes in Lightroom" (spec
// F5), the same options on either path. Each option with its text and, on its
// right, its effect on this run; "Copy depth to GPS altitude" under the dive
// details, the site written and "Location" and "GPS" under the site details;
// "Create collections" with the place chosen in Settings. Everything it shows
// is decided in review.ts and rules/review.ts; nothing can be changed while
// Apply writes.

import { useId, type ReactNode } from "react";
import { t } from "../i18n";
import { SitePicker } from "./SitePicker";
import type { DiveLog, Settings } from "../types";
import {
  chooseOtherSite,
  collectionsText,
  reviewBoxes,
  reviewEffects,
  siteDetailsTicked,
  siteRadios,
  type Box,
  type Effect,
} from "./review";
import { gpsError, type ReviewInput } from "./rules";
import type { Options } from "./state";

/** One option: its box, its name and text, then its effect on the right (none for "Location" and "GPS"). */
function Row({
  box,
  title,
  text,
  effect,
  sub = false,
  extra,
  onChange,
}: {
  box: Box;
  title: string;
  text: string;
  effect?: Effect;
  /** Under another option: indented. */
  sub?: boolean;
  /** Under the text, outside the box's label (a link). */
  extra?: ReactNode;
  onChange: (on: boolean) => void;
}) {
  const id = useId();
  return (
    <div className={`${sub ? "review-sub" : "review-opt"} ${box.enabled ? "" : "review-off"}`}>
      <div className="review-t">
        <input
          id={id}
          type="checkbox"
          checked={box.checked}
          disabled={!box.enabled}
          onChange={(e) => onChange(e.target.checked)}
        />
        <div className="review-body">
          <label htmlFor={id}>
            <b>{title}</b>
            <span className="review-text">{text}</span>
          </label>
          {extra}
        </div>
      </div>
      {effect && <div className={`review-eff ${effect.on ? "" : "muted"}`}>{effect.text}</div>}
    </div>
  );
}

export function ReviewOptions({
  input,
  log,
  settings,
  writing,
  onOptions,
  onOpenSettings,
}: {
  /** What step 4 knows: the journey and both photo readings. */
  input: ReviewInput;
  log: DiveLog;
  settings: Settings;
  writing: boolean;
  onOptions: (options: Partial<Options>) => void;
  onOpenSettings: () => void;
}) {
  const j = input.journey;
  const b = reviewBoxes(j);
  const e = reviewEffects(input);
  const r = siteRadios(j, log.dives);
  const radios = useId();
  return (
    <section className="card review" aria-labelledby="h-write">
      <fieldset className="review-fields" disabled={writing}>
        <div className="review-head">
          <h2 id="h-write">{t("journey.review.title")}</h2>
          <p className="muted small">{t("journey.review.hint")}</p>
        </div>

        <div className="review-grp lbl">{t("journey.review.metadata")}</div>
        <Row
          box={b.dive}
          title={t("journey.opt.dive")}
          text={t("journey.opt.diveText")}
          effect={e.dive}
          onChange={(on) => onOptions({ dive: on })}
        />
        <Row
          sub
          box={b.altitude}
          title={t("journey.opt.altitude")}
          text={t("journey.opt.altitudeText")}
          effect={e.altitude}
          onChange={(on) => onOptions({ altitude: on })}
        />
        <Row
          box={b.site}
          title={t("journey.opt.site")}
          text={t("journey.opt.siteText")}
          effect={e.site}
          onChange={(on) => onOptions(siteDetailsTicked(on))}
        />
        <div className={`review-sub ${r.other.enabled ? "" : "review-off"}`}>
          <div className="review-site">
            <div className="review-src" role="radiogroup" aria-label={t("journey.opt.site")}>
              <label className="review-radio">
                <input
                  type="radio"
                  name={radios}
                  checked={r.source === "diveSite"}
                  disabled={!r.diveSite.enabled}
                  onChange={() => onOptions({ site: "diveSite" })}
                />
                {r.diveSite.label}
              </label>
              <label className="review-radio">
                <input
                  type="radio"
                  name={radios}
                  checked={r.source === "other"}
                  disabled={!r.other.enabled}
                  onChange={() => onOptions(chooseOtherSite(j, log.dives))}
                />
                {r.other.label}
              </label>
            </div>
            {r.picker && (
              <SitePicker
                sites={log.sites}
                pick={j.options.pick}
                onChange={(pick) => onOptions({ pick })}
                gpsError={gpsError(input)}
              />
            )}
          </div>
        </div>
        <Row
          sub
          box={b.place}
          title={t("tag.location")}
          text={t("journey.opt.placeText")}
          onChange={(on) => onOptions({ place: on })}
        />
        <Row
          sub
          box={b.gps}
          title={t("journey.opt.gps")}
          text={t("journey.opt.gpsText")}
          onChange={(on) => onOptions({ gps: on })}
        />

        <div className="review-grp lbl">{t("settings.collections")}</div>
        <Row
          box={b.collections}
          title={t("journey.opt.collections")}
          text={collectionsText(j, settings.collections, log.dives, input.catalog.preview)}
          effect={e.collections}
          extra={
            <button type="button" className="link" onClick={onOpenSettings}>
              {t("journey.opt.placeLink")}
            </button>
          }
          onChange={(on) => onOptions({ collections: on })}
        />
      </fieldset>
    </section>
  );
}
