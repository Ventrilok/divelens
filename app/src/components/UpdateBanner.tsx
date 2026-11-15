// Above every screen (spec F13): Lightroom must restart to finish an update,
// or a new version was found (closed until a newer one comes).

import { t } from "../i18n";
import { Icon } from "./Icon";

export function UpdateBanner({
  version,
  restartLightroom,
  onSee,
  onClose,
}: {
  /** The new version found, or null. */
  version: string | null;
  restartLightroom: boolean;
  onSee: () => void;
  onClose: (version: string) => void;
}) {
  if (restartLightroom) {
    return (
      <div className="update-banner notice notice-warn" role="status">
        <span>{t("update.restartLightroom")}</span>
      </div>
    );
  }
  if (!version) return null;
  return (
    <div className="update-banner notice notice-ok" role="status">
      <span>{t("update.available", { version })}</span>
      <span className="update-banner-actions">
        <button type="button" className="btn btn-s btn-small" onClick={onSee}>
          {t("update.see")}
        </button>
        <button type="button" className="btn btn-g btn-small" aria-label={t("update.close")} onClick={() => onClose(version)}>
          <Icon name="close" size={14} />
        </button>
      </span>
    </div>
  );
}
