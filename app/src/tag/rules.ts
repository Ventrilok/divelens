// Tag Selection: when Apply is allowed and what the bar says. Pure rules,
// kept apart from the screen so that they can be tested.

import { t } from "../i18n";
import type { ApplyOutcome, Dive, TagError } from "../types";

/** What the screen knows when it decides whether Apply is allowed. */
export interface TagState {
  connected: boolean;
  /** Photos selected in Lightroom. */
  selected: number;
  withDive: boolean;
  withPlace: boolean;
  /** A dive is chosen. */
  chosen: boolean;
  /** The chosen dive has a profile that can be read: per-photo values are written. */
  needsValues: boolean;
  readFailure: string | null;
  profileLoading: boolean;
  /** The selection is described, and the description is current. */
  current: boolean;
  /** The counts shown are those of the current description. */
  freshCounts: boolean;
  usesDiveSite: boolean;
  pickedSite: boolean;
  gpsError: boolean;
}

/** What blocks Apply, in the order the user meets it; null when nothing does. */
export function tagBlocker(s: TagState): string | null {
  if (!s.connected) return t("tag.disconnected");
  if (s.selected === 0) return t("tag.selectInLr");
  if (!s.withDive && !s.withPlace) return t("tag.chooseSomething");
  if (s.withDive && !s.chosen) return t("tag.chooseDive");
  if (s.needsValues && s.readFailure) return s.readFailure;
  if (s.profileLoading) return t("detail.profileLoading");
  if (s.needsValues && (!s.current || !s.freshCounts)) return t("tag.perPhoto.reading");
  if (s.withPlace && !s.usesDiveSite && !s.pickedSite) return t("tag.chooseSite");
  if (s.gpsError) return t("tag.badGps");
  return null;
}

/** The bar's text: the blocker, else what will be written. */
export function tagSummary(blocker: string | null, chosen: Dive | null, placeName: string | null | undefined): string {
  if (blocker) return blocker;
  const diveLabel = chosen ? String(chosen.number ?? chosen.site.name ?? "—") : "";
  if (chosen && placeName) return t("tag.sumDiveAndPlace", { n: diveLabel, site: placeName });
  if (chosen) return t("tag.sumDive", { n: diveLabel });
  return t("tag.sumPlace", { site: placeName ?? "" });
}

export function canApplyTag(blocker: string | null, writing: boolean): boolean {
  return blocker === null && !writing;
}

/** Nothing was written because the selection is no longer the one described: it is described again. */
export function selectionChanged(e: ApplyOutcome | TagError): boolean {
  if ("outcome" in e) return e.outcome === "failed" && (e.reason === "selectionChanged" || e.reason === "staleSelection");
  return e.error === "noPhotos";
}
