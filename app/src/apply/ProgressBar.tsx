// The bar shown while Apply writes: what is going on, how far, the filled bar and a hint.

import { t } from "../i18n";

export function ProgressBar({ done, total, label }: { done: number; total: number; label: string }) {
  return (
    <div className="apply-progress">
      <div className="apply-progress-text">
        <span className="strong">{label}</span>
        {done > 0 && <span className="muted">{t("tag.progress", { done, total })}</span>}
      </div>
      <div
        className="bar"
        role="progressbar"
        aria-label={label}
        aria-valuemin={0}
        aria-valuemax={total}
        aria-valuenow={done}
      >
        <div className="bar-fill" style={{ width: `${total > 0 ? (100 * done) / total : 0}%` }} />
      </div>
      <span className="muted small">{t("tag.writingHint")}</span>
    </div>
  );
}
