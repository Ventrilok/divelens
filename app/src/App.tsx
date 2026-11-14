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
import { latestOnly, logChangedNotice, logWatch, sameLogSetting } from "./diveLog";
import { profiles } from "./profiles";
import { isReadyScreen, Sidebar, type Screen } from "./components/Sidebar";
import { lang, setLanguage, t, type Key } from "./i18n";
import { CollectionsScreen, type Phase } from "./screens/CollectionsScreen";
import { DivesScreen } from "./screens/DivesScreen";
import { TagScreen } from "./screens/TagScreen";
import { SettingsScreen, type LogState } from "./screens/SettingsScreen";
import { SourceError } from "./screens/SourceError";
import type { AppInfo, LightroomStatus, LoadError, Settings, Sort, SourceInfo, SourceKind } from "./types";
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
  // Why a flow started again (an action was refused: the dive log changed).
  // Shown once on the screen the diver lands on, until the next step,
  // Apply, Cancel, another screen, or Close.
  const [logNotice, setLogNotice] = useState<Key | null>(null);

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
    setLogNotice(null);
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
  // A read showing another log than the previous one (application in use and
  // location, as the back end checks them) starts every flow again: no dive
  // checked, Create Collections at step 1, Tag Selection from scratch.
  const reads = useMemo(latestOnly, []);
  const [otherLog] = useState(logWatch);
  const load = useCallback(() => {
    profiles.clear();
    setState({ status: "loading" });
    const show = (next: LogState, source: SourceInfo | undefined) => {
      if (source && otherLog(source)) {
        setChosen(new Set());
        setStep("dives");
        setTagRun((n) => n + 1);
      }
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
  // configured one: read the log and the settings again, and say why on the
  // screen the diver lands on.
  useEffect(() => {
    onLogChanged((applied) => {
      setLogNotice((current) => logChangedNotice(current, applied));
      load();
      getSettings().then(setSettings);
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
          notice={notice}
          onDismissNotice={dismissNotice}
          onNext={() => {
            setLogNotice(null);
            setStep("adjust");
          }}
        />
      )}
      {log && screen === "collections" && adjusting && (
        <CollectionsScreen
          log={log}
          lr={lr}
          settings={settings}
          chosen={chosen}
          onChooseDives={() => {
            setLogNotice(null);
            setStep("dives");
          }}
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
        <TagScreen
          key={tagRun}
          log={log}
          lr={lr}
          sort={sort}
          notice={notice}
          onDismissNotice={dismissNotice}
          onCancel={() => {
            setLogNotice(null);
            setTagRun((n) => n + 1);
          }}
        />
      )}
    </div>
  );
}

export default App;
