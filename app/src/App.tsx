import { useCallback, useEffect, useState } from "react";
import { loadDives } from "./api";
import { Sidebar, type Screen } from "./components/Sidebar";
import { t } from "./i18n";
import { DivesScreen } from "./screens/DivesScreen";
import { SourceError } from "./screens/SourceError";
import type { DiveLog, LoadError } from "./types";
import "./App.css";

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

  return (
    <div className="app">
      <Sidebar current={screen} onGo={setScreen} log={state.status === "ok" ? state.log : null} />
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
