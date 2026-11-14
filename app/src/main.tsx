import React from "react";
import ReactDOM from "react-dom/client";
import "@fontsource-variable/inter";
import { diveApps, getSettings } from "./api";
import App from "./App";
import { setLanguage } from "./i18n";
import { setApps } from "./sourceText";

// Settings and dive applications first: the very first screen is already in
// the chosen language, with the texts of the application in use. Without
// the applications (never expected), the screens still show.
const apps = diveApps().catch((e: unknown) => {
  console.error("dive applications not listed", e);
  return [];
});
Promise.all([getSettings(), apps]).then(([settings, list]) => {
  setLanguage(settings.language);
  setApps(list);
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <App initialSettings={settings} />
    </React.StrictMode>,
  );
});
