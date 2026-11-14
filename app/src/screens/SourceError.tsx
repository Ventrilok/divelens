// Shown instead of a screen when the dive log cannot be opened.

import { useState } from "react";
import { setSourcePath } from "../api";
import { Icon } from "../components/Icon";
import { changeErrorText } from "../diveLog";
import { t } from "../i18n";
import { appOf, appTexts, chooseButtons, chooseLog, errorKind, errorText } from "../sourceText";
import type { LoadError } from "../types";

/** Errors where the log is not at the location: trying again changes nothing, another location is needed. */
const ELSEWHERE = ["notFound", "notADiveLog", "severalLogs"];

export function SourceError({ failure, onRetry }: { failure: LoadError; onRetry: () => void }) {
  const app = appOf(failure.source.kind);
  const kind = errorKind(app, failure.error);
  const canRetry = !ELSEWHERE.includes(kind);
  // Why the location chosen was not saved (a ChangeError), or null.
  const [refused, setRefused] = useState<unknown>(null);

  const locate = async (folder: boolean) => {
    const picked = await chooseLog(app, failure.source.path, folder);
    if (!picked) return;
    try {
      await setSourcePath(picked);
    } catch (e) {
      setRefused(e);
      return;
    }
    setRefused(null);
    onRetry();
  };

  return (
    <main className="screen screen-center">
      <div className="card empty-state">
        <div className="empty-icon">
          <Icon name="database" size={24} />
        </div>
        <h1>{errorText(app, failure.error, "title")}</h1>
        <p className="muted">{errorText(app, failure.error, "text")}</p>
        <p className="path">{failure.source.path}</p>
        {kind === "database" && <p className="path">{failure.error.detail}</p>}
        {refused !== null && (
          <p className="set-error" role="alert">
            <Icon name="warn" size={16} />
            {changeErrorText(refused)}
          </p>
        )}
        <div className="actions">
          {chooseButtons(app, appTexts(app).locate ?? t("settings.choose")).map((b, i) => (
            <button key={b.label} type="button" className={i === 0 ? "btn btn-p" : "btn btn-s"} onClick={() => locate(b.folder)}>
              {b.label}
            </button>
          ))}
          {canRetry && (
            <button type="button" className="btn btn-s" onClick={onRetry}>
              {t("err.retry")}
            </button>
          )}
        </div>
      </div>
    </main>
  );
}
