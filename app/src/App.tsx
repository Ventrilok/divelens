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
import { Icon } from "./ui/Icon";
import { lang, setLanguage, t, type Key } from "./i18n";
import { CollectionsScreen, type Phase } from "./collections/CollectionsScreen";
import { DivesStep } from "./collections/DivesStep";
import { TagScreen } from "./tag/TagScreen";
import { SettingsScreen, type LogState } from "./settings/SettingsScreen";
import { SourceError } from "./settings/SourceError";
import { UpdateBanner } from "./updates/UpdateBanner";
import { noticeVersion } from "./updates/updates";
import type { AppInfo, LightroomStatus, LoadError, Settings, Sort, SourceInfo, SourceKind, UpdateStatus } from "./types";
import "./App.css";

// Temporary navigation between the two screens, until the journey (phase 3).
const tabs = [
  { id: "collections", icon: "folder", label: "nav.collections" },
  { id: "tag", icon: "tag", label: "nav.tag" },
] as const;

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
  coverFrom: null,
  coverTo: null,
  restartLightroom: false,
};

function App({ initialSettings }: { initialSettings: Settings }) {
  const [screen, setScreen] = useState<Screen>("collections");
  // The screen Settings were opened from: "Back" returns there.
  const [before, setBefore] = useState<Screen>("collections");
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

  // Entering Create Collections, from its tab or the Lightroom menu,
  // always opens step 1, with the checked dives kept (none after a summary).
  const open = useCallback((s: Screen) => {
    if (phase.current === "writing") return;
    setLogNotice(null);
    if (s === "settings") {
      // From a Lightroom menu: Back leads to the default screen. The screens
      // stay as they are meanwhile (a summary included).
      setBefore("collections");
    } else {
      if (phase.current === "summary") setChosen(new Set());
      if (s === "collections") setStep("dives");
    }
    setScreen(s);
  }, []);
  // The gear (or the update notice): Settings in place of the screen in use.
  const openSettings = useCallback(() => {
    setLogNotice(null);
    setBefore(screen === "settings" ? before : screen);
    setScreen("settings");
  }, [screen, before]);
  const closeSettings = useCallback(() => setScreen(before), [before]);
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
      <Header
        onSettings={() => (screen === "settings" ? closeSettings() : openSettings())}
        settingsDisabled={writing}
        inSettings={screen === "settings"}
      />
      <div className="content">
        <UpdateBanner
          version={newVersion}
          restartLightroom={lr.restartLightroom}
          onSee={openSettings}
          onClose={(version) => dismissUpdate(version)}
        />
        {screen === "settings" && (
          <SettingsScreen
            settings={settings}
            logState={state}
            lr={lr}
            info={info}
            update={update}
            writing={writing}
            error={saveError}
            onChange={change}
            onBack={closeSettings}
          />
        )}
        {/* Hidden, not unmounted, while Settings are shown: choices survive. */}
        <div className="journey" hidden={screen === "settings"}>
          {/* Temporary: the two tabs disappear with the journey (phase 3). */}
          <div className="tabs" role="tablist">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                type="button"
                role="tab"
                aria-selected={screen === tab.id}
                className={`tab ${screen === tab.id ? "tab-on" : ""}`}
                onClick={() => screen !== tab.id && open(tab.id)}
              >
                <Icon name={tab.icon} />
                {t(tab.label)}
              </button>
            ))}
          </div>
        {state.status === "loading" && (
          <main className="screen screen-center">
            <p className="muted">{t("dives.loading")}</p>
          </main>
        )}
        {state.status === "error" && <SourceError failure={state.failure} onRetry={load} />}
        {log && screen === "collections" && !adjusting && (
          <DivesStep
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
            onOpenSettings={openSettings}
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
      </div>
      <StatusBar lr={lr} log={log} />
    </div>
  );
}

export default App;
