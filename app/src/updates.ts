// What the update notice and Settings › Updates say (spec F13).

import { localized, t } from "./i18n";
import type { UpdateFailure, UpdateStatus } from "./types";

// A real moment (when updates were last looked for), shown in the time zone
// of this computer: unlike dive times, which are wall-clock.
const day = localized((l) => new Intl.DateTimeFormat(l, { day: "numeric", month: "long", year: "numeric" }));
const time = localized((l) => new Intl.DateTimeFormat(l, { hour: "2-digit", minute: "2-digit" }));

const sameDay = (a: Date, b: Date) =>
  a.getFullYear() === b.getFullYear() && a.getMonth() === b.getMonth() && a.getDate() === b.getDate();

/** When updates were last looked for: "today at 09:12", or "on 14 November 2026 at 09:12". */
export function checkedWhen(seconds: number, now: Date = new Date()): string {
  const at = new Date(seconds * 1000);
  const params = { date: day.format(at), time: time.format(at) };
  return sameDay(at, now) ? t("update.today", params) : t("update.onDay", params);
}

/** The version the notice above the screens announces, or null: none found, or its notice closed. */
export function noticeVersion(status: UpdateStatus | null): string | null {
  if (status?.phase.state !== "available") return null;
  return status.phase.version === status.dismissed ? null : status.phase.version;
}

/** The line under "Dive Lens <version>" in Settings, for states without an action of their own. */
export function stateLine(status: UpdateStatus, now: Date = new Date()): string {
  const when = status.lastCheck === null ? null : checkedWhen(status.lastCheck, now);
  switch (status.phase.state) {
    case "checking":
      return t("update.checking");
    case "upToDate":
      return when ? t("update.upToDate", { when }) : t("update.never");
    default:
      return when ? t("update.lastCheck", { when }) : t("update.never");
  }
}

export function failureText(reason: UpdateFailure): string {
  return t(`update.err.${reason}`);
}

/** Looking for or installing an update is under way: no other check meanwhile. */
export function isBusy(status: UpdateStatus | null): boolean {
  const state = status?.phase.state;
  return state === "checking" || state === "downloading" || state === "installing";
}
