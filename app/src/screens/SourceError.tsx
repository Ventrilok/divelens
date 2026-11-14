// Shown instead of a screen when the dive log cannot be opened.

import { useState } from "react";
import { setSourcePath } from "../api";
import { Icon } from "../components/Icon";
import { changeErrorText } from "../diveLog";
import { t } from "../i18n";
import { chooseLog, errorKind, errorText } from "../sourceText";
import type { LoadError } from "../types";

export function SourceError({ failure, onRetry }: { failure: LoadError; onRetry: () => void }) {
  const source = failure.source.kind;
  const kind = errorKind(source, failure.error);
  const canRetry = kind === "database" || kind === "oldFormat";
  // Why the location chosen was not saved (a ChangeError), or null.
  const [refused, setRefused] = useState<unknown>(null);

  const locate = async (folder: boolean) => {
    const picked = await chooseLog(source, failure.source.path, folder);
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
        <h1>{errorText(source, failure.error, "title")}</h1>
        <p className="muted">{errorText(source, failure.error, "text")}</p>
        <p className="path">{failure.source.path}</p>
        {kind === "database" && <p className="path">{failure.error.detail}</p>}
        {refused !== null && (
          <p className="set-error" role="alert">
            <Icon name="warn" size={16} />
            {changeErrorText(refused)}
          </p>
        )}
        <div className="actions">
          {source === "subsurface" ? (
            <>
              <button type="button" className="btn btn-p" onClick={() => locate(false)}>
                {t("errSs.chooseFile")}
              </button>
              <button type="button" className="btn btn-s" onClick={() => locate(true)}>
                {t("errSs.chooseFolder")}
              </button>
            </>
          ) : (
            <button type="button" className="btn btn-p" onClick={() => locate(false)}>
              {t("err.locate")}
            </button>
          )}
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
