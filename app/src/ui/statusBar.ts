// What the status bar says (spec F1), as plain text so it can be tested.

import { formatInt, t, tn } from "../i18n";
import { sourceName } from "../sourceText";
import type { DiveLog, LightroomStatus } from "../types";

export interface LightroomGroup {
  state: "connected" | "disconnected" | "restart";
  title: string;
  hint: string | null;
  items: string[];
}

export function lightroomItems(lr: LightroomStatus): LightroomGroup {
  // A plug-in from before an update, which this app cannot talk to.
  if (!lr.connected && lr.restartLightroom) {
    return {
      state: "restart",
      title: t("lr.restart"),
      hint: t("lr.restartHint"),
      items: [],
    };
  }
  if (!lr.connected) {
    return {
      state: "disconnected",
      title: t("lr.disconnected"),
      hint: t("lr.disconnectedHint"),
      items: [],
    };
  }
  const items: string[] = [];
  if (lr.catalog) items.push(t("status.catalog", { name: lr.catalog }));
  if (lr.catalogPhotos !== null)
    items.push(tn("status.catalogSize", lr.catalogPhotos));
  items.push(
    lr.selected > 0
      ? t("status.selected", { n: formatInt(lr.selected) })
      : t("status.selectedNone"),
  );
  return { state: "connected", title: t("lr.connected"), hint: null, items };
}

export function logItems(log: DiveLog | null): string[] {
  if (!log) return [];
  return [
    t("status.logApp", { name: sourceName(log.source.kind) }),
    t("status.logDives", { n: formatInt(log.dives.length) }),
  ];
}
