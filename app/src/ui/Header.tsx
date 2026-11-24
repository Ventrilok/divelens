import { t } from "../i18n";
import logo from "../../icon-source/logo-color.svg";
import { Icon } from "./Icon";

/** Logo and name on the left, the Settings gear on the right (spec F1). */
export function Header({
  onSettings,
  settingsDisabled,
  inSettings,
}: {
  onSettings: () => void;
  /** Lightroom is being written to: Settings stay closed. */
  settingsDisabled: boolean;
  inSettings: boolean;
}) {
  return (
    <header className="app-header">
      <div className="brand">
        <img src={logo} alt="" className="brand-logo" />
        <span>Dive Lens</span>
      </div>
      <button
        type="button"
        className="gear"
        aria-label={t("nav.settings")}
        title={t("nav.settings")}
        aria-pressed={inSettings}
        disabled={settingsDisabled}
        onClick={onSettings}
      >
        <Icon name="settings" size={20} />
      </button>
    </header>
  );
}
