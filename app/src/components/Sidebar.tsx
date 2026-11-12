import { tn, t } from "../i18n";
import { sourceName } from "../sourceText";
import type { DiveLog, LightroomStatus } from "../types";
import logo from "../../icon-source/logo-color.svg";
import { Icon, type IconName } from "./Icon";

export type Screen = "dives" | "collections" | "tag" | "settings";

const items: { id: Screen; icon: IconName; label: () => string; ready: boolean }[] = [
  { id: "dives", icon: "list", label: () => t("nav.dives"), ready: true },
  { id: "collections", icon: "folder", label: () => t("nav.collections"), ready: true },
  { id: "tag", icon: "tag", label: () => t("nav.tag"), ready: true },
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

/** Screens that exist so far; a request for another one is ignored. */
export function isReadyScreen(id: string): id is Screen {
  return id === "settings" || items.some((it) => it.id === id && it.ready);
}

function LightroomLine({ lr }: { lr: LightroomStatus }) {
  if (!lr.connected) {
    return (
      <>
        <div className="status-line">
          <span className="dot dot-warn" />
          <span className="status-title">{t("lr.disconnected")}</span>
        </div>
        <div>{t("lr.disconnectedHint")}</div>
      </>
    );
  }
  return (
    <>
      <div className="status-line">
        <span className="dot" />
        <span className="status-title">{t("lr.connected")}</span>
      </div>
      {lr.catalog && <div className="status-catalog">{lr.catalog}</div>}
      <div>{lr.selected > 0 ? tn("lr.selected", lr.selected) : t("lr.noSelection")}</div>
    </>
  );
}

export function Sidebar({
  current,
  onGo,
  log,
  lr,
}: {
  current: Screen;
  onGo: (s: Screen) => void;
  log: DiveLog | null;
  lr: LightroomStatus;
}) {
  return (
    <nav className="sidebar" aria-label="DiveLens">
      <div className="brand">
        <img src={logo} alt="" className="brand-logo" />
        <span>DiveLens</span>
      </div>
      {items.map((it) => (
        <NavItem key={it.id} {...it} current={current} onGo={onGo} />
      ))}
      <div className="sidebar-bottom">
        <NavItem id="settings" icon="settings" label={() => t("nav.settings")} ready current={current} onGo={onGo} />
        <div className="status">
          <LightroomLine lr={lr} />
          {log && (
            <div>
              {sourceName(log.source.kind)} · {tn("source.dives", log.dives.length)}
            </div>
          )}
        </div>
      </div>
    </nav>
  );
}
