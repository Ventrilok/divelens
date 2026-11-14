// Dive detail card. Shared by Create Collections (step 1) and Tag Selection.

import { lazy, Suspense, useEffect, useState } from "react";
import { formatDepth, formatDiveSpan, formatDuration, formatGps, formatTemps } from "../format";
import { t } from "../i18n";
import { profiles } from "../profiles";
import type { Capabilities, Dive, LogRef, Sample, Units } from "../types";
import { Icon } from "./Icon";
import { ProfileChart } from "./ProfileChart";
import { Stars } from "./Stars";

// The map library is large: loaded the first time a dive with GPS shows.
const DiveMap = lazy(() => import("./DiveMap"));

type ProfileState = { status: "loading" } | { status: "ok"; samples: Sample[] } | { status: "error" };

/** Profile of a dive of `log`, the dive log the screen shows. */
function useProfile(log: LogRef, dive: Dive): ProfileState {
  const [state, setState] = useState<ProfileState>({ status: "loading" });
  useEffect(() => {
    if (!dive.hasProfile) return;
    const source = { kind: log.kind, path: log.path };
    const cached = profiles.get(source, dive.id);
    if (cached) {
      setState({ status: "ok", samples: cached });
      return;
    }
    let live = true;
    setState({ status: "loading" });
    profiles
      .load(source, dive.id)
      .then((samples) => live && setState({ status: "ok", samples }))
      .catch(() => live && setState({ status: "error" }));
    return () => {
      live = false;
    };
  }, [log.kind, log.path, dive.id, dive.hasProfile]);
  return state;
}

export function siteSubtitle(dive: Dive): string {
  return [dive.site.location, dive.site.country].filter(Boolean).join(" · ");
}

function Profile({ log, dive, units }: { log: LogRef; dive: Dive; units: Units }) {
  const state = useProfile(log, dive);
  if (!dive.hasProfile || (state.status === "ok" && state.samples.length < 2)) {
    return (
      <>
        <div className="profile-empty">
          <Icon name="noCurve" size={16} />
          {t("detail.noProfile")}
        </div>
        <p className="detail-hint">{t("detail.noProfileHint")}</p>
      </>
    );
  }
  return (
    <figure className="profile">
      {state.status === "ok" ? (
        <ProfileChart samples={state.samples} units={units} />
      ) : (
        <div className="profile-placeholder">
          {state.status === "loading" ? t("detail.profileLoading") : t("detail.profileError")}
        </div>
      )}
      <figcaption>
        <Icon name="check" size={14} className="accent" />
        {t("detail.profileOk")}
      </figcaption>
    </figure>
  );
}

/** Coldest in blue, warmest in orange, each with its unit and a small
 *  "min." / "max." above it. */
function Temps({ low, high, units }: { low: number | null; high: number | null; units: Units }) {
  const temps = formatTemps(low, high, units);
  if (temps.same !== null) return <>{temps.same}</>;
  return (
    <span className="temps">
      {temps.low && (
        <span className="temp temp-low">
          <small>{t("detail.tempLow")}</small>
          {temps.low}
        </span>
      )}
      {temps.high && (
        <span className="temp temp-high">
          <small>{t("detail.tempHigh")}</small>
          {temps.high}
        </span>
      )}
    </span>
  );
}

export function DiveDetail({
  log,
  dive,
  units,
  capabilities,
}: {
  /** The dive log shown, whose profiles are read. */
  log: LogRef;
  dive: Dive | null;
  units: Units;
  capabilities: Capabilities;
}) {
  if (!dive) {
    return (
      <aside className="card detail detail-empty">
        <p className="muted">{t("detail.empty")}</p>
      </aside>
    );
  }
  const gps = formatGps(dive.site.lat, dive.site.lon);
  const label = dive.number !== null ? t("detail.label", { n: dive.number }) : t("detail.labelNoNumber");
  return (
    <aside className="card detail" aria-label={label}>
      <div className="detail-head">
        <span className="lbl">
          {label}
          {dive.invalid && (
            <span className="badge badge-invalid" title={t("list.invalidHint")}>
              {t("list.invalid")}
            </span>
          )}
        </span>
        <h2>{dive.site.name ?? t("detail.unknownSite")}</h2>
        {siteSubtitle(dive) && <span className="detail-place">{siteSubtitle(dive)}</span>}
        <span className="muted small">{formatDiveSpan(dive.start, dive.end)}</span>
      </div>
      {capabilities.profile && <Profile log={log} dive={dive} units={units} />}
      <dl className="facts">
        <div>
          <dt className="lbl">{t("detail.duration")}</dt>
          <dd>{formatDuration(dive.durationS)}</dd>
        </div>
        <div>
          <dt className="lbl">{t("detail.depth")}</dt>
          <dd>{formatDepth(dive.maxDepthM, units)}</dd>
        </div>
        {capabilities.temperature && (
          <div>
            <dt className="lbl">{t("detail.temp")}</dt>
            <dd>
              <Temps low={dive.tempLowC} high={dive.tempHighC} units={units} />
            </dd>
          </div>
        )}
        {capabilities.rating && (
          <div>
            <dt className="lbl">{t("detail.rating")}</dt>
            <dd>
              <Stars rating={dive.rating} />
            </dd>
          </div>
        )}
        {capabilities.gps && gps && (
          <div className="span2">
            <dt className="lbl">{t("detail.gps")}</dt>
            <dd className="gps">{gps}</dd>
          </div>
        )}
      </dl>
      {capabilities.gps && dive.site.lat !== null && dive.site.lon !== null && (
        <Suspense fallback={<div className="map" />}>
          <DiveMap lat={dive.site.lat} lon={dive.site.lon} />
        </Suspense>
      )}
      {dive.notes && (
        <section className="notes" aria-label={t("detail.notes")}>
          <h3 className="lbl">{t("detail.notes")}</h3>
          <p>{dive.notes}</p>
        </section>
      )}
    </aside>
  );
}
