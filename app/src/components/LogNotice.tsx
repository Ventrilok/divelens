// Why a flow started again: the dive log changed while the diver was in it.
// Shown once, on the screen the diver lands on.

import { t } from "../i18n";

export function LogNotice({ text, onDismiss }: { text: string; onDismiss: () => void }) {
  return (
    <div className="notice notice-warn" role="alert">
      <span>{text}</span>
      <button type="button" className="btn btn-g" onClick={onDismiss}>
        {t("tag.dismiss")}
      </button>
    </div>
  );
}
