import { useCallback, useEffect, useState } from "react";
import { initialScreen, lightroomStatus, loadDives, onLightroomStatus, onShowScreen } from "./api";
import { isReadyScreen, Sidebar, type Screen } from "./components/Sidebar";
import { t } from "./i18n";
import { CollectionsScreen } from "./screens/CollectionsScreen";
import { DivesScreen } from "./screens/DivesScreen";
import { TagScreen } from "./screens/TagScreen";
import { SourceError } from "./screens/SourceError";
import type { DiveLog, LightroomStatus, LoadError } from "./types";
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

type State = { status: "loading" } | { status: "ok"; log: DiveLog } | { status: "error"; failure: LoadError };

function App() {
  const [screen, setScreen] = useState<Screen>("dives");
  // Dives chosen on the Dives screen, kept when going to Create Collections and back.
  const [chosen, setChosen] = useState<Set<string>>(new Set());
  const [state, setState] = useState<State>({ status: "loading" });

  const load = useCallback(() => {
    setState({ status: "loading" });
    loadDives()
      .then((log) => setState({ status: "ok", log }))
      .catch((failure: LoadError) => setState({ status: "error", failure }));
  }, []);

  useEffect(load, [load]);

  // Lightroom link: current state, then live updates; menus choose the screen.
  const [lr, setLr] = useState<LightroomStatus>(offline);
  useEffect(() => {
    const go = (s: string | null) => s && isReadyScreen(s) && setScreen(s);
    initialScreen().then(go);
    lightroomStatus().then(setLr);
    const stops = [onLightroomStatus(setLr), onShowScreen(go)];
    return () => stops.forEach((p) => p.then((stop) => stop()));
  }, []);

  return (
    <div className="app">
      <Sidebar current={screen} onGo={setScreen} log={state.status === "ok" ? state.log : null} lr={lr} />
      {state.status === "loading" && (
        <main className="screen screen-center">
          <p className="muted">{t("dives.loading")}</p>
        </main>
      )}
      {state.status === "error" && <SourceError failure={state.failure} onRetry={load} />}
      {state.status === "ok" && screen === "dives" && (
        <DivesScreen log={state.log} selected={chosen} onSelect={setChosen} onCreate={() => setScreen("collections")} />
      )}
      {state.status === "ok" && screen === "collections" && (
        <CollectionsScreen
          log={state.log}
          lr={lr}
          chosen={chosen}
          onChooseDives={() => setScreen("dives")}
          onRetryDives={(ids) => setChosen(new Set(ids))}
        />
      )}
      {state.status === "ok" && screen === "tag" && <TagScreen log={state.log} lr={lr} onCancel={() => setScreen("dives")} />}
    </div>
  );
}

export default App;
