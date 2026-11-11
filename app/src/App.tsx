import { useCallback, useEffect, useState } from "react";
import { initialScreen, lightroomStatus, loadDives, onLightroomStatus, onShowScreen } from "./api";
import { isReadyScreen, Sidebar, type Screen } from "./components/Sidebar";
import { t } from "./i18n";
import { DivesScreen } from "./screens/DivesScreen";
import { SourceError } from "./screens/SourceError";
import type { DiveLog, LightroomStatus, LoadError } from "./types";
import "./App.css";

const offline: LightroomStatus = { connected: false, catalog: null, lightroomVersion: null, pluginVersion: null, selected: 0 };

type State = { status: "loading" } | { status: "ok"; log: DiveLog } | { status: "error"; failure: LoadError };

function App() {
  const [screen, setScreen] = useState<Screen>("dives");
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
      {state.status === "ok" && screen === "dives" && <DivesScreen log={state.log} />}
    </div>
  );
}

export default App;
