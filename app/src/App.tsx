import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

// Placeholder window until the first real screen (Dives) is built.
function App() {
  const [versions, setVersions] = useState<[string, string] | null>(null);

  useEffect(() => {
    invoke<[string, string]>("versions").then(setVersions).catch(() => setVersions(null));
  }, []);

  return (
    <main className="placeholder">
      <h1>DiveLens</h1>
      <p>Development build.</p>
      {versions && (
        <p className="muted">
          App {versions[0]} · Core {versions[1]}
        </p>
      )}
    </main>
  );
}

export default App;
