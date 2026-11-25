// Settings screen (spec F2, F3, F7, F10, F11, F13). Every change is saved at once;
// the app follows it live (language, units, dive log location, remembered clock offsets).

import { useEffect, useState } from "react";
import { open, save } from "@tauri-apps/plugin-dialog";
import { openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";
import {
  checkSettingsFile,
  checkUpdate,
  collectionSets,
  exportSettings,
  forgetOffset,
  importSettings,
  installUpdate,
  rememberedOffsets,
} from "../api";
import { fileName, importErrorText } from "./backup";
import { offsetLines } from "./offsets";
import { flattenSets, pathKey, placeLabel, setExists } from "../collections/collectionPlace";
import { Icon } from "../ui/Icon";
import { changeErrorText } from "../diveLog";
import { systemLang, t, tn } from "../i18n";
import {
  appOf,
  appTexts,
  availableApps,
  chooseButtons,
  chooseLog,
  errorText,
  readText,
  withLocation,
} from "../sourceText";
import { failureText, isBusy, stateLine } from "../updates/updates";
import type {
  AppInfo,
  CollectionSet,
  DiveApp,
  DiveLog,
  ImportCheck,
  LightroomStatus,
  LoadError,
  RememberedOffset,
  Settings,
  SourceInfo,
  UpdateStatus,
} from "../types";

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
  writing,
  onChange,
}: {
  settings: Settings;
  logState: LogState;
  /** Lightroom is being written to: the dive log cannot change. */
  writing: boolean;
  onChange: (p: Patch) => void;
}) {
  const source = logState.status === "ok" ? logState.log.source : logState.status === "error" ? logState.failure.source : null;
  // The application in use, as the back end resolves it (the one chosen
  // when this system has it).
  const app = appOf(source?.kind ?? settings.source.kind);
  const texts = appTexts(app);
  const setPath = (path: string | null) => onChange({ source: withLocation(settings.source, app, path) });
  const choose = async (folder: boolean) => {
    const picked = await chooseLog(app, source?.path, folder);
    if (picked) setPath(picked);
  };
  // One button next to the location, or a row of them (a file or a folder).
  const buttons = chooseButtons(app).map((b) => (
    <button key={b.label} type="button" className="btn btn-s" disabled={writing} onClick={() => choose(b.folder)}>
      {b.label}
    </button>
  ));

  let state: React.ReactNode;
  if (logState.status === "loading") {
    state = <span className="set-state muted">{texts.reading}</span>;
  } else if (logState.status === "ok") {
    const counts = {
      dives: tn("source.dives", logState.log.dives.length),
      sites: tn("settings.sites", logState.log.sites.length),
    };
    state = (
      <span className="set-state">
        <Icon name="check" size={14} className="accent" />
        {readText(app, logState.log.format, counts)}
      </span>
    );
  } else {
    state = (
      <span className="set-state set-warn">
        <Icon name="warn" size={14} />
        {errorText(app, logState.failure.error, "title")}
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
          value={app.id}
          disabled={writing}
          onChange={(e) => onChange({ source: { ...settings.source, kind: e.target.value } })}
        >
          {availableApps().map((a) => (
            <option key={a.id} value={a.id}>
              {a.name}
            </option>
          ))}
        </select>
      </Row>
      <Row title={texts.log} hint={texts.logHint} top>
        <div className="set-path">
          <div className="set-path-row">
            <input className="inp" aria-label={texts.log} value={source?.path ?? ""} readOnly />
            {buttons.length === 1 && buttons}
          </div>
          {buttons.length > 1 && <div className="set-path-row">{buttons}</div>}
          <div className="set-path-row set-path-state">
            {state}
            {source && !source.isDefaultPath && (
              <button type="button" className="btn btn-g btn-small" disabled={writing} onClick={() => setPath(null)}>
                {t("settings.standard")}
              </button>
            )}
          </div>
          {writing && <span className="set-note">{t("settings.logLocked")}</span>}
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

/**
 * Remembered clock offsets (spec F2): the cameras whose offset is remembered for the
 * dive application in use, each with "Forget", which removes it from the offsets file
 * at once. Read when Settings open, again for another application and after each
 * "Forget". Nothing in Lightroom changes.
 */
function OffsetsSection({ app, onForgotten }: { app: DiveApp; onForgotten: (key: string) => void }) {
  const [remembered, setRemembered] = useState<RememberedOffset[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let live = true;
    setRemembered(null);
    setError(null);
    rememberedOffsets()
      .then((r) => live && setRemembered(r))
      .catch((e: unknown) => live && setError(t("settings.offsetsUnread", { detail: String(e) })));
    return () => {
      live = false;
    };
  }, [app.id]);

  const forget = async (key: string) => {
    setBusy(true);
    setError(null);
    try {
      setRemembered(await forgetOffset(key));
      onForgotten(key);
    } catch (e) {
      setError(t("settings.forgetError", { detail: String(e) }));
    } finally {
      setBusy(false);
    }
  };
  const lines = remembered ? offsetLines(remembered) : [];

  return (
    <section className="card" aria-labelledby="s-offsets">
      <h2 id="s-offsets" className="sec">
        {t("settings.offsets")}
      </h2>
      <p className="sec-hint">{t("settings.offsetsHint", { app: app.name })}</p>
      {error && (
        <div className="set">
          <div className="notice notice-warn set-full" role="alert">
            <span>{error}</span>
          </div>
        </div>
      )}
      {remembered !== null && lines.length === 0 && (
        <div className="set">
          <span className="set-note">{t("settings.offsetsNone")}</span>
        </div>
      )}
      {lines.map((l) => (
        <Row key={l.key} title={l.name} hint={l.serial ?? undefined}>
          <div className="set-actions set-offset">
            <span className="set-offset-n">{l.offset}</span>
            <button
              type="button"
              className="btn btn-s btn-small"
              disabled={busy}
              aria-label={t("settings.forgetLabel", { camera: l.serial ? `${l.name} ${l.serial}` : l.name })}
              onClick={() => forget(l.key)}
            >
              {t("settings.forget")}
            </button>
          </div>
        </Row>
      ))}
    </section>
  );
}

/**
 * Settings backup (spec F11): export the settings to a file, or replace them
 * with a file's after confirmation. A dive log the file names but this
 * computer lacks is reported, with the way to choose where it is.
 */
function BackupSection({
  settings,
  logState,
  writing,
  onChange,
}: {
  settings: Settings;
  logState: LogState;
  /** Lightroom is being written to: an import, which may name another dive log, waits. */
  writing: boolean;
  onChange: (p: Patch) => void;
}) {
  const [pending, setPending] = useState<{ path: string; check: ImportCheck } | null>(null);
  const [missing, setMissing] = useState<SourceInfo | null>(null);
  const [message, setMessage] = useState<{ tone: "ok" | "warn"; text: string } | null>(null);
  const [busy, setBusy] = useState(false);
  const filters = [{ name: t("settings.backupFilter"), extensions: ["json"] }];

  const reset = () => {
    setPending(null);
    setMissing(null);
    setMessage(null);
  };
  const doExport = async () => {
    reset();
    const path = await save({ defaultPath: t("settings.exportName"), filters });
    if (!path) return;
    try {
      await exportSettings(path);
      setMessage({ tone: "ok", text: t("settings.exported", { name: fileName(path) }) });
    } catch (e) {
      setMessage({ tone: "warn", text: t("settings.exportError", { detail: String(e) }) });
    }
  };
  const doImport = async () => {
    reset();
    const path = await open({ multiple: false, directory: false, filters });
    if (typeof path !== "string") return;
    try {
      setPending({ path, check: await checkSettingsFile(path) });
    } catch (e) {
      setMessage({ tone: "warn", text: importErrorText(e) });
    }
  };
  const confirm = async () => {
    if (!pending) return;
    setBusy(true);
    try {
      const result = await importSettings(pending.path);
      if (result.found) setMessage({ tone: "ok", text: t("settings.imported") });
      else setMissing(result.source);
    } catch (e) {
      setMessage({ tone: "warn", text: importErrorText(e) });
    } finally {
      setBusy(false);
      setPending(null);
    }
  };
  const choose = async (folder: boolean) => {
    if (!missing) return;
    const app = appOf(missing.kind);
    const picked = await chooseLog(app, undefined, folder);
    if (!picked) return;
    onChange({ source: withLocation(settings.source, app, picked) });
    setMissing(null);
  };
  // Once the dive log reads (chosen here or in the Dive log section), the
  // reminder goes.
  const showMissing = missing !== null && logState.status !== "ok";

  return (
    <section className="card" aria-labelledby="s-backup">
      <h2 id="s-backup" className="sec">
        {t("settings.backup")}
      </h2>
      <Row
        title={t("settings.backupRow")}
        hint={t("settings.backupHint")}
        top={pending !== null || showMissing || message !== null || writing}
      >
        <div className="set-actions">
          <button type="button" className="btn btn-s" disabled={busy} onClick={doExport}>
            {t("settings.export")}
          </button>
          <button type="button" className="btn btn-s" disabled={busy || writing} onClick={doImport}>
            {t("settings.import")}
          </button>
        </div>
        {writing && <span className="set-note set-full">{t("settings.logLocked")}</span>}
        {pending && (
          <div className="set-explain" role="group" aria-label={t("settings.import")}>
            <b className="set-ask">{t("settings.importAsk", { name: fileName(pending.path) })}</b>
            <span>{t("settings.importAskHint")}</span>
            {!pending.check.found && (
              <span className="set-warn">{t("settings.importAskMissing", { path: pending.check.source.path })}</span>
            )}
            <div className="set-actions">
              <button type="button" className="btn btn-p" disabled={busy || writing} onClick={confirm}>
                {t("settings.importReplace")}
              </button>
              <button type="button" className="btn btn-g" disabled={busy} onClick={() => setPending(null)}>
                {t("settings.importCancel")}
              </button>
            </div>
          </div>
        )}
        {showMissing && missing && (
          <div className="set-explain" role="alert">
            <span className="set-warn">{t("settings.importedMissing", { path: missing.path })}</span>
            <div className="set-actions">
              {chooseButtons(appOf(missing.kind)).map((b) => (
                <button key={b.label} type="button" className="btn btn-s" disabled={writing} onClick={() => choose(b.folder)}>
                  {b.label}
                </button>
              ))}
            </div>
          </div>
        )}
        {message && (
          <div className={`notice notice-${message.tone} set-full`} role={message.tone === "ok" ? "status" : "alert"}>
            <span>{message.text}</span>
          </div>
        )}
      </Row>
    </section>
  );
}

function UpdatesSection({
  settings,
  update,
  info,
  writing,
  onChange,
}: {
  settings: Settings;
  update: UpdateStatus | null;
  info: AppInfo | null;
  /** Lightroom is being written to: nothing is installed meanwhile. */
  writing: boolean;
  onChange: (p: Patch) => void;
}) {
  const phase = update?.phase ?? { state: "idle" as const };
  const notes = update?.notesUrl ?? null;
  const releases = update?.releasesUrl ?? null;
  // The version found, while it can still be installed (again, after a failure).
  const found =
    phase.state === "available"
      ? phase.version
      : phase.state === "failed" && phase.reason !== "notInstallable"
        ? phase.version
        : null;
  const byHand = update !== null && (!update.installable || (phase.state === "failed" && phase.reason === "notInstallable"));

  let explain: React.ReactNode = null;
  if (phase.state === "downloading" || phase.state === "installing") {
    const text =
      phase.state === "downloading"
        ? t("update.downloading", { version: phase.version })
        : t("update.installing", { version: phase.version });
    const percent = phase.state === "downloading" ? phase.percent : 100;
    explain = (
      <div className="set-explain" role="status">
        <span>{text}</span>
        <div className="bar" aria-hidden="true">
          <div className="bar-fill" style={{ width: `${percent ?? 0}%` }} />
        </div>
      </div>
    );
  } else if (found || phase.state === "failed") {
    const version = found ?? (phase.state === "failed" ? phase.version : null);
    explain = (
      <div className="set-explain" role={phase.state === "failed" ? "alert" : "status"}>
        {phase.state === "failed" && <span className="set-warn">{failureText(phase.reason)}</span>}
        {version && <b className="set-ask">{t("update.newVersion", { version })}</b>}
        {version && byHand && <span>{t("update.downloadHint")}</span>}
        {version && !byHand && <span>{t("update.restartAfter")}</span>}
        {version && !byHand && writing && <span className="set-note">{t("update.err.writing")}</span>}
        {version && (
          <div className="set-actions">
            {byHand ? (
              <button type="button" className="btn btn-p" onClick={() => openUrl(notes ?? releases ?? SOURCE_CODE)}>
                {t("update.download")}
              </button>
            ) : (
              <button type="button" className="btn btn-p" disabled={writing} onClick={() => installUpdate()}>
                {t("update.install")}
              </button>
            )}
            {notes && (
              <button type="button" className="btn btn-g" onClick={() => openUrl(notes)}>
                {t("update.notes")}
              </button>
            )}
          </div>
        )}
      </div>
    );
  }

  return (
    <section className="card" aria-labelledby="s-upd">
      <h2 id="s-upd" className="sec">
        {t("settings.updates")}
      </h2>
      <Row title={t("settings.autoCheck")} hint={t("settings.autoCheckHint")}>
        <input
          className="sw"
          type="checkbox"
          role="switch"
          aria-label={t("settings.autoCheckLabel")}
          checked={settings.checkUpdates}
          onChange={(e) => onChange({ checkUpdates: e.target.checked })}
        />
      </Row>
      <Row
        title={t("settings.appVersion", { version: info?.version ?? "" })}
        hint={update ? stateLine(update) : undefined}
        top={explain !== null}
      >
        <button type="button" className="btn btn-s" disabled={isBusy(update)} onClick={() => checkUpdate()}>
          {t("update.checkNow")}
        </button>
        {explain}
      </Row>
    </section>
  );
}

export function SettingsScreen({
  settings,
  logState,
  lr,
  info,
  update,
  writing,
  error,
  onChange,
  onForgotten,
  onBack,
}: {
  settings: Settings;
  logState: LogState;
  lr: LightroomStatus;
  info: AppInfo | null;
  update: UpdateStatus | null;
  /** Lightroom is being written to: the dive log cannot change. */
  writing: boolean;
  /** Why the last change was not made (a ChangeError), or null. */
  error: unknown;
  onChange: (patch: Patch) => void;
  /** A camera's remembered clock offset was forgotten (its key). */
  onForgotten: (key: string) => void;
  /** Back to the screen Settings were opened from. */
  onBack: () => void;
}) {
  const app = appOf(
    logState.status === "ok" ? logState.log.source.kind : logState.status === "error" ? logState.failure.source.kind : settings.source.kind,
  );
  const appName = app.name;
  const sourceUnits = logState.status === "ok" ? logState.log.sourceUnits : null;
  const asSource = (unit: string | null) =>
    unit ? t("settings.asSource", { source: appName, unit }) : t("settings.asSourceBare", { source: appName });
  const systemName = t(`lang.${systemLang()}`);

  return (
    <main className="screen screen-scroll">
      <div className="settings">
        <button type="button" className="btn btn-s settings-back" onClick={onBack}>
          <Icon name="back" />
          {t("settings.back")}
        </button>
        <h1 className="settings-title">{t("settings.title")}</h1>
        {error !== null && (
          <p className="set-error" role="alert">
            <Icon name="warn" size={16} />
            {changeErrorText(error)}
          </p>
        )}

        <SourceSection settings={settings} logState={logState} writing={writing} onChange={onChange} />

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

        <CollectionsSection settings={settings} lr={lr} onChange={onChange} />

        <OffsetsSection app={app} onForgotten={onForgotten} />

        <BackupSection settings={settings} logState={logState} writing={writing} onChange={onChange} />

        <UpdatesSection settings={settings} update={update} info={info} writing={writing} onChange={onChange} />

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
