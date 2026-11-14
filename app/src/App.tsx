import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  appInfo,
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
  updateSettings,
} from "./api";
import { unitsInUse } from "./collectionPlace";
import { latestOnly, sourceWatch } from "./diveLog";
import { profiles } from "./profiles";
import { isReadyScreen, Sidebar, type Screen } from "./components/Sidebar";
import { lang, setLanguage, t } from "./i18n";
import { CollectionsScreen, type Phase } from "./screens/CollectionsScreen";
import { DivesScreen } from "./screens/DivesScreen";
import { TagScreen } from "./screens/TagScreen";
import { SettingsScreen, type LogState } from "./screens/SettingsScreen";
import { SourceError } from "./screens/SourceError";
import type { AppInfo, LightroomStatus, LoadError, Settings, Sort } from "./types";
import "./App.css";

const offline: LightroomStatus = {
  connected: false,
  catalog: null,
  lightroomVersion: null,
  pluginVersion: null,
  selected: 0,
  selectionVersion: 0,
  selectionFrom: null,
  selectionTo: null,
  coverFrom: null,
  coverTo: null,
};

function App({ initialSettings }: { initialSettings: Settings }) {
  const [screen, setScreen] = useState<Screen>("collections");
  // Create Collections: step 1 checks the dives, step 2 adjusts and applies
  // (its summary is step 3). The checked dives are kept between steps.
  const [step, setStep] = useState<"dives" | "adjust">("dives");
  const [chosen, setChosen] = useState<Set<string>>(new Set());
  // Tag Selection's Cancel starts it again from scratch.
  const [tagRun, setTagRun] = useState(0);

  // While collections are being written, the screen stays put so that the
  // summary is seen. Leaving the summary, whichever way, unchecks the dives.
  const phase = useRef<Phase>("adjust");
  const onPhase = useCallback((p: Phase) => {
    phase.current = p;
  }, []);

  // Entering Create Collections, from the sidebar or the Lightroom menu,
  // always opens step 1, with the checked dives kept (none after a summary).
  const open = useCallback((s: Screen) => {
    if (phase.current === "writing") return;
    if (phase.current === "summary") setChosen(new Set());
    setScreen(s);
    if (s === "collections") setStep("dives");
  }, []);
  const [state, setState] = useState<LogState>({ status: "loading" });
  const [settings, setSettings] = useState<Settings>(initialSettings);
  // Why the last change of settings was not made (a ChangeError), or null.
  const [saveError, setSaveError] = useState<unknown>(null);
  const [info, setInfo] = useState<AppInfo | null>(null);

  // Each read of the dive log forgets the profiles shown so far, and only the
  // latest read counts: a slow read of the previous log never replaces it.
  const reads = useMemo(latestOnly, []);
  const load = useCallback(() => {
    profiles.clear();
    setState({ status: "loading" });
    reads(
      loadDives(),
      (log) => setState({ status: "ok", log }),
      (failure) => setState({ status: "error", failure: failure as LoadError }),
    );
  }, [reads]);

  // The dive log is read again whenever the application or its location
  // changes, whichever way (Settings, the dive-log error screen, an import,
  // the previous plugin's settings taken over). Another log starts every flow
  // again: no dive checked, Create Collections at step 1, Tag Selection from
  // scratch.
  const source = settings.source;
  const sourceKey = JSON.stringify(source);
  const [sourceChanged] = useState(() => sourceWatch(initialSettings.source));
  useEffect(() => {
    if (sourceChanged(source)) {
      setChosen(new Set());
      setStep("dives");
      setTagRun((n) => n + 1);
    }
    load();
  }, [load, sourceKey, sourceChanged]);

  // The back end refused an action on a dive log that is no longer the
  // configured one: catch up with the settings, which reads the log again.
  useEffect(() => {
    onLogChanged(() => {
      getSettings().then(setSettings);
    });
  }, []);

  // While Lightroom is being written to, the dive log cannot change.
  const [writing, setWriting] = useState(false);
  useEffect(() => {
    lightroomWriting().then(setWriting);
    const stop = onLightroomWriting(setWriting);
    return () => {
      stop.then((f) => f());
    };
  }, []);

  // Settings changed elsewhere (back end) reach this window too.
  useEffect(() => {
    appInfo().then(setInfo);
    const stop = onSettingsChanged(setSettings);
    return () => {
      stop.then((f) => f());
    };
  }, []);

  // The language changes at once: every string and format follows.
  setLanguage(settings.language);

  const change = useCallback((patch: Partial<Settings>) => {
    // A new dive log is read once its location is saved, not before (the
    // back end reads the saved settings).
    if (!patch.source) setSettings((s) => ({ ...s, ...patch }));
    updateSettings(patch)
      .then((s) => {
        setSettings(s);
        setSaveError(null);
      })
      .catch((e: unknown) => setSaveError(e));
  }, []);

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

  // Units in use: the dive application's unless forced in Settings.
  const okLog = state.status === "ok" ? state.log : null;
  const log = useMemo(
    () => okLog && { ...okLog, units: unitsInUse(settings.units, okLog.sourceUnits) },
    [okLog, settings.units],
  );

  // Step 2 needs checked dives of this log (another dive log may have been chosen meanwhile).
  const hasChosen = useMemo(() => !!log && log.dives.some((d) => chosen.has(d.id)), [log, chosen]);
  const adjusting = step === "adjust" && hasChosen;
  useEffect(() => {
    if (log && !hasChosen) setStep("dives");
  }, [log, hasChosen]);

  return (
    <div className="app" key={lang}>
      <Sidebar current={screen} onGo={(s) => s !== screen && open(s)} log={log} lr={lr} />
      {screen === "settings" && (
        <SettingsScreen
          settings={settings}
          logState={state}
          lr={lr}
          info={info}
          writing={writing}
          error={saveError}
          onChange={change}
        />
      )}
      {screen !== "settings" && state.status === "loading" && (
        <main className="screen screen-center">
          <p className="muted">{t("dives.loading")}</p>
        </main>
      )}
      {screen !== "settings" && state.status === "error" && <SourceError failure={state.failure} onRetry={load} />}
      {log && screen === "collections" && !adjusting && (
        <DivesScreen
          log={log}
          selected={chosen}
          onSelect={setChosen}
          sort={sort}
          onSort={onSort}
          onNext={() => setStep("adjust")}
        />
      )}
      {log && screen === "collections" && adjusting && (
        <CollectionsScreen
          log={log}
          lr={lr}
          settings={settings}
          chosen={chosen}
          onChooseDives={() => setStep("dives")}
          onDone={() => {
            setChosen(new Set());
            setStep("dives");
          }}
          onRetryDives={(ids) => setChosen(new Set(ids))}
          onOpenSettings={() => setScreen("settings")}
          onPhase={onPhase}
        />
      )}
      {log && screen === "tag" && (
        <TagScreen key={tagRun} log={log} lr={lr} sort={sort} onCancel={() => setTagRun((n) => n + 1)} />
      )}
    </div>
  );
}

export default App;
