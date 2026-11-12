import { useCallback, useEffect, useMemo, useState } from "react";
import {
  appInfo,
  initialScreen,
  lightroomStatus,
  loadDives,
  onLightroomStatus,
  onSettingsChanged,
  onShowScreen,
  updateSettings,
} from "./api";
import { unitsInUse } from "./collectionPlace";
import { isReadyScreen, Sidebar, type Screen } from "./components/Sidebar";
import { lang, setLanguage, t } from "./i18n";
import { CollectionsScreen } from "./screens/CollectionsScreen";
import { DivesScreen } from "./screens/DivesScreen";
import { TagScreen } from "./screens/TagScreen";
import { SettingsScreen, type LogState } from "./screens/SettingsScreen";
import { SourceError } from "./screens/SourceError";
import type { AppInfo, LightroomStatus, Settings, Sort } from "./types";
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
  const [screen, setScreen] = useState<Screen>("dives");
  // Dives chosen on the Dives screen, kept when going to Create Collections and back.
  const [chosen, setChosen] = useState<Set<string>>(new Set());
  const [state, setState] = useState<LogState>({ status: "loading" });
  const [settings, setSettings] = useState<Settings>(initialSettings);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [info, setInfo] = useState<AppInfo | null>(null);

  const load = useCallback(() => {
    setState({ status: "loading" });
    loadDives()
      .then((log) => setState({ status: "ok", log }))
      .catch((failure) => setState({ status: "error", failure }));
  }, []);

  // The dive log is read again whenever its location changes (here, or when
  // the previous plugin's settings were taken over).
  const sourceKey = JSON.stringify(settings.source);
  useEffect(load, [load, sourceKey]);

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
      .catch((e: unknown) => setSaveError(String(e)));
  }, []);

  const sort: Sort = settings.diveSort ?? "recent";
  const onSort = useCallback((s: Sort) => change({ diveSort: s }), [change]);

  // Lightroom link: current state, then live updates; menus choose the screen.
  const [lr, setLr] = useState<LightroomStatus>(offline);
  useEffect(() => {
    const go = (s: string | null) => s && isReadyScreen(s) && setScreen(s);
    initialScreen().then(go);
    lightroomStatus().then(setLr);
    const stops = [onLightroomStatus(setLr), onShowScreen(go)];
    return () => stops.forEach((p) => p.then((stop) => stop()));
  }, []);

  // Units in use: the dive application's unless forced in Settings.
  const okLog = state.status === "ok" ? state.log : null;
  const log = useMemo(
    () => okLog && { ...okLog, units: unitsInUse(settings.units, okLog.sourceUnits) },
    [okLog, settings.units],
  );

  return (
    <div className="app" key={lang}>
      <Sidebar current={screen} onGo={setScreen} log={log} lr={lr} />
      {screen === "settings" && (
        <SettingsScreen
          settings={settings}
          logState={state}
          lr={lr}
          info={info}
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
      {log && screen === "dives" && (
        <DivesScreen
          log={log}
          selected={chosen}
          onSelect={setChosen}
          sort={sort}
          onSort={onSort}
          onCreate={() => setScreen("collections")}
        />
      )}
      {log && screen === "collections" && (
        <CollectionsScreen
          log={log}
          lr={lr}
          settings={settings}
          chosen={chosen}
          onChooseDives={() => setScreen("dives")}
          onRetryDives={(ids) => setChosen(new Set(ids))}
          onOpenSettings={() => setScreen("settings")}
        />
      )}
      {log && screen === "tag" && <TagScreen log={log} lr={lr} sort={sort} onCancel={() => setScreen("dives")} />}
    </div>
  );
}

export default App;
