import { t } from "../i18n";
import type { DiveLog, LightroomStatus } from "../types";
import { lightroomItems, logItems } from "./statusBar";

/** Bottom bar: the Lightroom link and catalog on the left, the dive log on the right (spec F1). */
export function StatusBar({
  lr,
  log,
}: {
  lr: LightroomStatus;
  log: DiveLog | null;
}) {
  const group = lightroomItems(lr);
  const logLine = logItems(log);
  return (
    <footer className="status-bar" aria-label={t("status.label")}>
      <div className="status-group" role="group" aria-label="Lightroom">
        <span className="status-title">
          <span
            className={`dot ${group.state === "connected" ? "" : "dot-warn"}`}
            aria-hidden="true"
          />
          {group.title}
        </span>
        {group.hint && (
          <span className="status-item status-hint">{group.hint}</span>
        )}
        {group.items.map((item) => (
          <span key={item} className="status-item">
            {item}
          </span>
        ))}
      </div>
      {logLine.length > 0 && (
        <div
          className="status-group"
          role="group"
          aria-label={t("status.logGroup")}
        >
          {logLine.map((item) => (
            <span key={item} className="status-item">
              {item}
            </span>
          ))}
        </div>
      )}
    </footer>
  );
}
