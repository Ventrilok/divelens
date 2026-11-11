// Shown instead of a screen when the dive log cannot be opened.

import { open } from "@tauri-apps/plugin-dialog";
import { setSourcePath } from "../api";
import { Icon } from "../components/Icon";
import { t } from "../i18n";
import type { LoadError } from "../types";

export function SourceError({ failure, onRetry }: { failure: LoadError; onRetry: () => void }) {
  const kind = failure.error.kind === "notFound" || failure.error.kind === "notADiveLog" ? failure.error.kind : "database";

  const locate = async () => {
    const dir = failure.source.path.replace(/[/\\][^/\\]*$/, "");
    const picked = await open({
      multiple: false,
      directory: false,
      defaultPath: dir || undefined,
      filters: [{ name: t("err.dialogFilter"), extensions: ["sqlite"] }],
    });
    if (typeof picked === "string") {
      await setSourcePath(picked);
      onRetry();
    }
  };

  return (
    <main className="screen screen-center">
      <div className="card empty-state">
        <div className="empty-icon">
          <Icon name="database" size={24} />
        </div>
        <h1>{t(`err.${kind}.title`)}</h1>
        <p className="muted">{t(`err.${kind}.text`)}</p>
        <p className="path">{failure.source.path}</p>
        {kind === "database" && <p className="path">{failure.error.detail}</p>}
        <div className="actions">
          <button type="button" className="btn btn-p" onClick={locate}>
            {t("err.locate")}
          </button>
          {kind === "database" && (
            <button type="button" className="btn btn-s" onClick={onRetry}>
              {t("err.retry")}
            </button>
          )}
        </div>
      </div>
    </main>
  );
}
