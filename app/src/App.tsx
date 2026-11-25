import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  appInfo,
  dismissUpdate,
  getSettings,
  initialScreen,
  lightroomStatus,
  lightroomWriting,
  loadDives,
  onLightroomStatus,
  onLightroomWriting,
  onLogChanged,
  onSettingsChanged,
  onShowScreen,
  onUpdateStatus,
  updateSettings,
  updateStatus,
} from "./api";
import { revertChange } from "./settings/save";
import { unitsInUse } from "./collections/collectionPlace";
import { latestOnly, logChangedNotice, logWatch, sameLogSetting } from "./diveLog";
import { profiles } from "./dives/profiles";
import { isReadyScreen, type Screen } from "./ui/screens";
import { Header } from "./ui/Header";
import { StatusBar } from "./ui/AppStatusBar";
import { lang, setLanguage, t, type Key } from "./i18n";
import { Journey } from "./journey/Journey";
import { SettingsScreen, type LogState } from "./settings/SettingsScreen";
import { SourceError } from "./settings/SourceError";
import { UpdateBanner } from "./updates/UpdateBanner";
import { noticeVersion } from "./updates/updates";
import type { AppInfo, DiveLog, LightroomStatus, LoadError, Settings, Sort, SourceInfo, SourceKind, UpdateStatus } from "./types";
import "./App.css";

const offline: LightroomStatus = {
  connected: false,
  catalog: null,
  lightroomVersion: null,
  pluginVersion: null,
  selected: 0,
  catalogPhotos: null,
  selectionVersion: 0,
  selectionFrom: null,
  selectionTo: null,
  selectionCameras: [],
  coverFrom: null,
  coverTo: null,
  restartLightroom: false,
};

function App({ initialSettings }: { initialSettings: Settings }) {
  // The journey, or Settings in its place (the journey stays mounted, hidden).
  const [screen, setScreen] = useState<Screen>("journey");
  // Why the journey started again (an action was refused: the dive log
  // changed). Shown once at step 1, until Close, leaving step 1, a menu or the gear.
  const [logNotice, setLogNotice] = useState<Key | null>(null);
  // "Open Dive Lens…", counted: each one brings the journey back to step 1.
  const [menu, setMenu] = useState(0);
  // Apply is writing in Lightroom (the journey says so at once, the back end
  // once the write reached it): no menu and no Settings leave the screen
  // meanwhile, so that the summary shows. Read by the menus' listener.
  const [journeyWriting, setJourneyWriting] = useState(false);
  const writingNow = useRef(false);

  // A Lightroom menu (or the Plug-in Manager, or an update): the journey at
  // step 1 with its choices kept, or Settings. Dropped while writing, as before.
  const open = useCallback((s: Screen) => {
    if (writingNow.current) return;
    setLogNotice(null);
    if (s === "journey") setMenu((n) => n + 1);
    setScreen(s);
  }, []);
  // The gear (or the update notice): Settings in place of the journey.
  const openSettings = useCallback(() => {
    if (writingNow.current) return;
    setLogNotice(null);
    setScreen("settings");
  }, []);
  // "Back": the journey, at the step it was on, with its choices.
  const closeSettings = useCallback(() => setScreen("journey"), []);
  // The cameras whose remembered clock offset Settings forgot, in order: the
  // journey no longer shows them as remembered.
  const [forgotten, setForgotten] = useState<string[]>([]);
  const onForgotten = useCallback((key: string) => setForgotten((keys) => [...keys, key]), []);
  const [state, setState] = useState<LogState>({ status: "loading" });
  const [settings, setSettings] = useState<Settings>(initialSettings);
  // What the settings file holds: the screen may show a change not saved yet.
  const savedSettings = useRef(initialSettings);
  const showSaved = useCallback((s: Settings) => {
    savedSettings.current = s;
    setSettings(s);
  }, []);
  // Why the last change of settings was not made (a ChangeError), or null.
  const [saveError, setSaveError] = useState<unknown>(null);
  const [info, setInfo] = useState<AppInfo | null>(null);

  // Each read of the dive log forgets the profiles shown so far, and only the
  // latest read counts: a slow read of the previous log never replaces it.
  // A read showing another log than the previous one (application in use and
  // location, as the back end checks them), even one that fails, starts the
  // journey again at step 1 with nothing chosen. The journey keeps the last
  // log that loaded while another read is under way.
  const reads = useMemo(latestOnly, []);
  const [otherLog] = useState(logWatch);
  const [logChanges, setLogChanges] = useState(0);
  const [lastLog, setLastLog] = useState<DiveLog | null>(null);
  const load = useCallback(() => {
    profiles.clear();
    setState({ status: "loading" });
    const show = (next: LogState, source: SourceInfo | undefined) => {
      if (source && otherLog(source)) setLogChanges((n) => n + 1);
      if (next.status === "ok") setLastLog(next.log);
      setState(next);
    };
    reads(
      loadDives(),
      (log) => show({ status: "ok", log }, log.source),
      (error) => {
        const failure = error as LoadError;
        show({ status: "error", failure }, failure?.source);
      },
    );
  }, [reads, otherLog]);

  // The dive log is read again whenever the application or its location
  // changes, whichever way (Settings, the dive-log error screen, an import,
  // the previous plugin's settings taken over). A change of the other
  // application's location only reads nothing: the log in use stays on
  // screen, and so do the flows.
  const source = settings.source;
  const sourceKey = JSON.stringify(source);
  const readFor = useRef(source);
  const kindInUse = useRef<SourceKind | null>(null);
  kindInUse.current =
    state.status === "ok" ? state.log.source.kind : state.status === "error" ? (state.failure.source?.kind ?? null) : null;
  useEffect(() => {
    const before = readFor.current;
    readFor.current = source;
    if (before !== source && kindInUse.current && sameLogSetting(before, source, kindInUse.current)) return;
    load();
  }, [load, sourceKey]);

  // The back end refused an action on a dive log that is no longer the
  // configured one: read the log and the settings again, and say why at step 1,
  // where the journey starts again.
  useEffect(() => {
    onLogChanged((applied) => {
      setLogNotice((current) => logChangedNotice(current, applied));
      load();
      getSettings().then(showSaved);
    });
  }, [load]);
  const notice = logNotice && t(logNotice);
  const dismissNotice = useCallback(() => setLogNotice(null), []);

  // While Lightroom is being written to, the dive log cannot change. Events
  // win over the first answer, which may arrive after one of them.
  const [writing, setWriting] = useState(false);
  useEffect(() => {
    let heard = false;
    const stop = onLightroomWriting((on) => {
      heard = true;
      setWriting(on);
    });
    lightroomWriting().then((on) => {
      if (!heard) setWriting(on);
    });
    return () => {
      stop.then((f) => f());
    };
  }, []);
  writingNow.current = writing || journeyWriting;

  // Settings changed elsewhere (back end) reach this window too.
  useEffect(() => {
    appInfo().then(setInfo);
    const stop = onSettingsChanged(showSaved);
    return () => {
      stop.then((f) => f());
    };
  }, []);

  // The language changes at once: every string and format follows.
  setLanguage(settings.language);

  const change = useCallback((patch: Partial<Settings>) => {
    // Shown at once (a new dive log is read once its location is saved, not
    // before); if saving fails, the last saved value comes back.
    if (!patch.source) setSettings((s) => ({ ...s, ...patch }));
    updateSettings(patch)
      .then((s) => {
        showSaved(s);
        setSaveError(null);
      })
      .catch((e: unknown) => {
        setSettings((s) => revertChange(s, patch, savedSettings.current));
        setSaveError(e);
      });
  }, [showSaved]);

  const sort: Sort = settings.diveSort ?? "recent";
  const onSort = useCallback((s: Sort) => change({ diveSort: s }), [change]);

  // Lightroom link: current state, then live updates; menus choose the screen.
  const [lr, setLr] = useState<LightroomStatus>(offline);
  useEffect(() => {
    const go = (s: string | null) => s && isReadyScreen(s) && open(s);
    initialScreen().then(go);
    lightroomStatus().then(setLr);
    const stops = [onLightroomStatus(setLr), onShowScreen(go)];
    return () => stops.forEach((p) => p.then((stop) => stop()));
  }, [open]);

  // Updates (spec F13): where they stand, then live changes.
  const [update, setUpdate] = useState<UpdateStatus | null>(null);
  useEffect(() => {
    let heard = false;
    const stop = onUpdateStatus((u) => {
      heard = true;
      setUpdate(u);
    });
    updateStatus().then((u) => {
      if (!heard) setUpdate(u);
    });
    return () => {
      stop.then((f) => f());
    };
  }, []);
  // The notice of a new version shows above the other screens; Settings
  // tells it in its Updates section.
  const newVersion = screen === "settings" ? null : noticeVersion(update);

  // Units in use: the dive application's unless forced in Settings.
  const journeyLog = useMemo(
    () => lastLog && { ...lastLog, units: unitsInUse(settings.units, lastLog.sourceUnits) },
    [lastLog, settings.units],
  );
  // The log the status bar names: the one read, once it is (the journey's then).
  const log = state.status === "ok" ? journeyLog : null;

  // A change of language shows every text again in the new one; the journey
  // is not mounted again, so that it keeps its step and its choices (spec F2).
  return (
    <div className="app">
      <Header
        key={`header-${lang}`}
        onSettings={() => (screen === "settings" ? closeSettings() : openSettings())}
        settingsDisabled={writing || journeyWriting}
        inSettings={screen === "settings"}
      />
      <div className="content">
        <UpdateBanner
          key={`update-${lang}`}
          version={newVersion}
          restartLightroom={lr.restartLightroom}
          onSee={openSettings}
          onClose={(version) => dismissUpdate(version)}
        />
        {screen === "settings" && (
          <SettingsScreen
            key={`settings-${lang}`}
            settings={settings}
            logState={state}
            lr={lr}
            info={info}
            update={update}
            writing={writing}
            error={saveError}
            onChange={change}
            onForgotten={onForgotten}
            onBack={closeSettings}
          />
        )}
        {screen === "journey" && state.status === "loading" && (
          <main key={`loading-${lang}`} className="screen screen-center">
            <p className="muted">{t("dives.loading")}</p>
          </main>
        )}
        {screen === "journey" && state.status === "error" && (
          <SourceError key={`error-${lang}`} failure={state.failure} onRetry={load} />
        )}
        {/* Hidden, not unmounted, while Settings show or the log is read again: choices survive. */}
        {journeyLog && (
          <div className="journey" hidden={screen === "settings" || state.status !== "ok"}>
            <Journey
              log={journeyLog}
              lr={lr}
              settings={settings}
              sort={sort}
              onSort={onSort}
              notice={notice}
              onDismissNotice={dismissNotice}
              onOpenSettings={openSettings}
              onWriting={setJourneyWriting}
              menu={menu}
              logChanges={logChanges}
              forgotten={forgotten}
            />
          </div>
        )}
      </div>
      <StatusBar key={`status-${lang}`} lr={lr} log={log} />
    </div>
  );
}

export default App;
