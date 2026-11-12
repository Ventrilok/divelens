// Settings screen (spec F2, F3, F7, F10). Every change is saved at once;
// the app follows it live (language, units, dive log location).

import { useEffect, useState } from "react";
import { openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";
import { collectionSets } from "../api";
import { flattenSets, pathKey, placeLabel, setExists } from "../collectionPlace";
import { Icon } from "../components/Icon";
import { systemLang, t, tn } from "../i18n";
import { chooseLog, errorText, sourceName } from "../sourceText";
import type { AppInfo, CollectionSet, DiveLog, LightroomStatus, LoadError, Settings, SourceKind } from "../types";

const SOURCE_CODE = "https://github.com/Ventrilok/divelens";

export type LogState = { status: "loading" } | { status: "ok"; log: DiveLog } | { status: "error"; failure: LoadError };

type Patch = Partial<Settings>;

function Row({ title, hint, children, top }: { title: string; hint?: string; children: React.ReactNode; top?: boolean }) {
  return (
    <div className={`set ${top ? "set-top" : ""}`}>
      <div className="set-t">
        <b>{title}</b>
        {hint && <span>{hint}</span>}
      </div>
      {children}
    </div>
  );
}

function Segmented<T extends string>({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: T;
  options: { value: T; label: string }[];
  onChange: (v: T) => void;
}) {
  return (
    <div className="seg" role="radiogroup" aria-label={label}>
      {options.map((o) => (
        <label key={o.value} className={o.value === value ? "on" : ""}>
          <input type="radio" checked={o.value === value} onChange={() => onChange(o.value)} />
          {o.label}
        </label>
      ))}
    </div>
  );
}

function SourceSection({
  settings,
  logState,
  info,
  onChange,
}: {
  settings: Settings;
  logState: LogState;
  info: AppInfo | null;
  onChange: (p: Patch) => void;
}) {
  if (info?.os === "windows") {
    return (
      <section className="card" aria-labelledby="s-src">
        <h2 id="s-src" className="sec">
          {t("settings.source")}
        </h2>
        <div className="set">
          <span className="muted">{t("settings.noSource")}</span>
        </div>
      </section>
    );
  }
  const source = logState.status === "ok" ? logState.log.source : logState.status === "error" ? logState.failure.source : null;
  // The application in use (on Windows, always Subsurface).
  const kind: SourceKind = source?.kind ?? settings.source.kind;
  const ss = kind === "subsurface";
  const setPath = (path: string | null) =>
    onChange({ source: { ...settings.source, [ss ? "subsurfacePath" : "macDivePath"]: path } });
  const choose = async (folder: boolean) => {
    const picked = await chooseLog(kind, source?.path, folder);
    if (picked) setPath(picked);
  };

  let state: React.ReactNode;
  if (logState.status === "loading") {
    state = <span className="set-state muted">{t(ss ? "settings.ssLoading" : "settings.baseLoading")}</span>;
  } else if (logState.status === "ok") {
    const counts = {
      dives: tn("source.dives", logState.log.dives.length),
      sites: tn("settings.sites", logState.log.sites.length),
    };
    const storage = logState.log.storage;
    const key = storage?.kind === "git" ? "settings.gitOk" : storage?.kind === "xml" ? "settings.xmlOk" : "settings.baseOk";
    state = (
      <span className="set-state">
        <Icon name="check" size={14} className="accent" />
        {t(key, counts)}
      </span>
    );
  } else {
    state = (
      <span className="set-state set-warn">
        <Icon name="warn" size={14} />
        {errorText(kind, logState.failure.error, "title")}
      </span>
    );
  }

  return (
    <section className="card" aria-labelledby="s-src">
      <h2 id="s-src" className="sec">
        {t("settings.source")}
      </h2>
      <Row title={t("settings.app")} hint={t("settings.appHint")}>
        <select
          className="inp set-select"
          aria-label={t("settings.app")}
          value={kind}
          onChange={(e) => onChange({ source: { ...settings.source, kind: e.target.value as SourceKind } })}
        >
          <option value="macDive">{sourceName("macDive")}</option>
          <option value="subsurface">{sourceName("subsurface")}</option>
        </select>
      </Row>
      <Row title={t(ss ? "settings.ssBase" : "settings.base")} hint={t(ss ? "settings.ssBaseHint" : "settings.baseHint")} top>
        <div className="set-path">
          <div className="set-path-row">
            <input className="inp" aria-label={t(ss ? "settings.ssBase" : "settings.base")} value={source?.path ?? ""} readOnly />
            {!ss && (
              <button type="button" className="btn btn-s" onClick={() => choose(false)}>
                {t("settings.choose")}
              </button>
            )}
          </div>
          {ss && (
            <div className="set-path-row">
              <button type="button" className="btn btn-s" onClick={() => choose(false)}>
                {t("errSs.chooseFile")}
              </button>
              <button type="button" className="btn btn-s" onClick={() => choose(true)}>
                {t("errSs.chooseFolder")}
              </button>
            </div>
          )}
          <div className="set-path-row set-path-state">
            {state}
            {source && !source.isDefaultPath && (
              <button type="button" className="btn btn-g btn-small" onClick={() => setPath(null)}>
                {t("settings.standard")}
              </button>
            )}
          </div>
        </div>
      </Row>
    </section>
  );
}

function CollectionsSection({ settings, lr, onChange }: { settings: Settings; lr: LightroomStatus; onChange: (p: Patch) => void }) {
  const place = settings.collections;
  const [sets, setSets] = useState<CollectionSet[] | null>(null);
  const [group, setGroup] = useState(place.group);
  useEffect(() => setGroup(place.group), [place.group]);

  useEffect(() => {
    if (!lr.connected) {
      setSets(null);
      return;
    }
    let live = true;
    collectionSets()
      .then((s) => live && setSets(s))
      .catch(() => live && setSets(null));
    return () => {
      live = false;
    };
  }, [lr.connected, lr.catalog]);

  const missing = sets !== null && !setExists(sets, place.parent);
  const choices = sets ? flattenSets(sets) : [];
  const current = pathKey(place.parent);
  const commitGroup = () => {
    if (group !== place.group) onChange({ collections: { ...place, group } });
  };
  const root = t("settings.root");
  const example = placeLabel({ ...place, group }, root);

  return (
    <section className="card" aria-labelledby="s-coll">
      <h2 id="s-coll" className="sec">
        {t("settings.collections")}
      </h2>
      <Row title={t("settings.parent")} hint={missing ? t("settings.parentMissingHint") : t("settings.parentHint")}>
        <div className="set-path set-path-narrow">
          <select
            className="inp set-select"
            aria-label={t("settings.parent")}
            aria-invalid={missing}
            value={current}
            disabled={sets === null}
            onChange={(e) => onChange({ collections: { ...place, parent: JSON.parse(e.target.value) as string[] } })}
          >
            <option value={pathKey([])}>{root}</option>
            {(sets === null || missing) && place.parent.length > 0 && (
              <option value={current}>
                {missing ? t("settings.parentMissing", { path: place.parent.join(" › ") }) : place.parent.join(" › ")}
              </option>
            )}
            {choices.map((c) => (
              <option key={pathKey(c.path)} value={pathKey(c.path)}>
                {" ".repeat(c.depth)}
                {c.path[c.path.length - 1]}
              </option>
            ))}
          </select>
          {sets === null && <span className="set-note">{t("settings.parentOffline")}</span>}
        </div>
      </Row>
      <Row title={t("settings.group")} hint={t("settings.groupHint")} top>
        <div className="set-path set-path-narrow">
          <input
            className="inp"
            aria-label={t("settings.group")}
            value={group}
            maxLength={120}
            onChange={(e) => setGroup(e.target.value)}
            onBlur={commitGroup}
            onKeyDown={(e) => e.key === "Enter" && commitGroup()}
          />
          <span className="set-note">
            {t("settings.preview", { path: `${example} › ${t("settings.exampleCollection")}` })}
          </span>
          <span className="set-note">{t("settings.moveNote")}</span>
        </div>
      </Row>
    </section>
  );
}

export function SettingsScreen({
  settings,
  logState,
  lr,
  info,
  error,
  onChange,
}: {
  settings: Settings;
  logState: LogState;
  lr: LightroomStatus;
  info: AppInfo | null;
  /** Why the last change was not saved. */
  error: string | null;
  onChange: (patch: Patch) => void;
}) {
  const appName = sourceName(
    logState.status === "ok" ? logState.log.source.kind : logState.status === "error" ? logState.failure.source.kind : settings.source.kind,
  );
  const sourceUnits = logState.status === "ok" ? logState.log.sourceUnits : null;
  const asSource = (unit: string | null) =>
    unit ? t("settings.asSource", { source: appName, unit }) : t("settings.asSourceBare", { source: appName });
  const systemName = t(`lang.${systemLang()}`);

  return (
    <main className="screen screen-scroll">
      <div className="settings">
        <h1 className="settings-title">{t("settings.title")}</h1>
        {error && (
          <p className="set-error" role="alert">
            <Icon name="warn" size={16} />
            {t("settings.saveError", { detail: error })}
          </p>
        )}

        <SourceSection settings={settings} logState={logState} info={info} onChange={onChange} />

        <section className="card" aria-labelledby="s-aff">
          <h2 id="s-aff" className="sec">
            {t("settings.display")}
          </h2>
          <Row title={t("settings.language")} hint={t("settings.languageHint")}>
            <select
              className="inp set-select"
              aria-label={t("settings.language")}
              value={settings.language}
              onChange={(e) => onChange({ language: e.target.value as Settings["language"] })}
            >
              <option value="system">{t("settings.languageSystem", { name: systemName })}</option>
              <option value="fr">{t("lang.fr")}</option>
              <option value="en">{t("lang.en")}</option>
            </select>
          </Row>
          <Row title={t("settings.depth")} hint={t("settings.unitsHint")}>
            <Segmented
              label={t("settings.depth")}
              value={settings.units.depth}
              options={[
                { value: "source", label: asSource(sourceUnits && (sourceUnits.depth === "ft" ? "ft" : "m")) },
                { value: "m", label: t("settings.metres") },
                { value: "ft", label: t("settings.feet") },
              ]}
              onChange={(depth) => onChange({ units: { ...settings.units, depth } })}
            />
          </Row>
          <Row title={t("settings.temp")} hint={t("settings.unitsHint")}>
            <Segmented
              label={t("settings.temp")}
              value={settings.units.temp}
              options={[
                { value: "source", label: asSource(sourceUnits && (sourceUnits.temp === "f" ? "°F" : "°C")) },
                { value: "c", label: "°C" },
                { value: "f", label: "°F" },
              ]}
              onChange={(temp) => onChange({ units: { ...settings.units, temp } })}
            />
          </Row>
        </section>

        <section className="card" aria-labelledby="s-write">
          <h2 id="s-write" className="sec">
            {t("settings.writing")}
          </h2>
          <Row title={t("settings.altitude")} hint={t("settings.altitudeHint")} top>
            <input
              className="sw"
              type="checkbox"
              role="switch"
              aria-label={t("settings.altitude")}
              checked={settings.depthToAltitude}
              onChange={(e) => onChange({ depthToAltitude: e.target.checked })}
            />
            <div className="set-explain">
              <span>{t("settings.altitude1")}</span>
              <span>{t("settings.altitude2")}</span>
              <span className="set-warn">{t("settings.altitude3")}</span>
            </div>
          </Row>
        </section>

        <CollectionsSection settings={settings} lr={lr} onChange={onChange} />

        <section className="card" aria-labelledby="s-diag">
          <h2 id="s-diag" className="sec">
            {t("settings.diagnostic")}
          </h2>
          <Row title={t("settings.logLevel")} hint={t("settings.logLevelHint")}>
            <div className="set-actions">
              <select
                className="inp"
                aria-label={t("settings.logLevel")}
                value={settings.logLevel}
                onChange={(e) => onChange({ logLevel: e.target.value as Settings["logLevel"] })}
              >
                <option value="normal">{t("settings.logNormal")}</option>
                <option value="detailed">{t("settings.logDetailed")}</option>
              </select>
              <button
                type="button"
                className="btn btn-s"
                disabled={!info?.logPath}
                onClick={() => info?.logPath && revealItemInDir(info.logPath)}
              >
                {t("settings.showLog")}
              </button>
            </div>
          </Row>
        </section>

        <section className="card" aria-labelledby="s-about">
          <h2 id="s-about" className="sec">
            {t("settings.about")}
          </h2>
          <Row title={t("settings.free")} hint={t("settings.freeHint")}>
            <button type="button" className="btn btn-s" onClick={() => openUrl(SOURCE_CODE)}>
              {t("settings.code")}
            </button>
          </Row>
        </section>

        {info && <p className="settings-foot">{t("settings.version", { version: info.version })}</p>}
      </div>
    </main>
  );
}
