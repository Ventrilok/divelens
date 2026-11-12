import React from "react";
import ReactDOM from "react-dom/client";
import "@fontsource-variable/inter";
import { getSettings } from "./api";
import App from "./App";
import { setLanguage } from "./i18n";
import type { Settings } from "./types";

// Settings first: the very first screen is already in the chosen language.
getSettings().then((settings: Settings) => {
  setLanguage(settings.language);
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <App initialSettings={settings} />
    </React.StrictMode>,
  );
});
