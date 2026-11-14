import React from "react";
import ReactDOM from "react-dom/client";
import "@fontsource-variable/inter";
import { diveApps, getSettings } from "./api";
import App from "./App";
import { setLanguage } from "./i18n";
import { setApps } from "./sourceText";

// Settings and dive applications first: the very first screen is already in
// the chosen language, with the texts of the application in use.
Promise.all([getSettings(), diveApps()]).then(([settings, apps]) => {
  setLanguage(settings.language);
  setApps(apps);
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <App initialSettings={settings} />
    </React.StrictMode>,
  );
});
