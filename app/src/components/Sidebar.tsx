import { tn, t } from "../i18n";
import type { DiveLog } from "../types";
import { Icon, type IconName } from "./Icon";

export type Screen = "dives" | "collections" | "tag" | "settings";

const items: { id: Screen; icon: IconName; label: () => string; ready: boolean }[] = [
  { id: "dives", icon: "list", label: () => t("nav.dives"), ready: true },
  { id: "collections", icon: "folder", label: () => t("nav.collections"), ready: false },
  { id: "tag", icon: "tag", label: () => t("nav.tag"), ready: false },
];

function NavItem({ id, icon, label, ready, current, onGo }: (typeof items)[number] & { current: Screen; onGo: (s: Screen) => void }) {
  return (
    <button
      type="button"
      className={`nav ${current === id ? "nav-on" : ""}`}
      aria-current={current === id ? "page" : undefined}
      disabled={!ready}
      title={ready ? undefined : t("nav.soon")}
      onClick={() => onGo(id)}
    >
      <Icon name={icon} />
      {label()}
    </button>
  );
}

export function Sidebar({ current, onGo, log }: { current: Screen; onGo: (s: Screen) => void; log: DiveLog | null }) {
  return (
    <nav className="sidebar" aria-label="DiveLens">
      <div className="brand">
        <Icon name="logo" size={26} className="accent" />
        <span>DiveLens</span>
      </div>
      {items.map((it) => (
        <NavItem key={it.id} {...it} current={current} onGo={onGo} />
      ))}
      <div className="sidebar-bottom">
        <NavItem id="settings" icon="settings" label={() => t("nav.settings")} ready={false} current={current} onGo={onGo} />
        <div className="status">
          <div className="status-line">
            <span className="dot dot-warn" />
            <span className="status-title">{t("lr.disconnected")}</span>
          </div>
          {log && (
            <div>
              {t("source.macDive")} · {tn("source.dives", log.dives.length)}
            </div>
          )}
        </div>
      </div>
    </nav>
  );
}
