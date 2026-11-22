// Tag Selection: when Apply is allowed and what the bar says. Pure rules,
// kept apart from the screen so that they can be tested.

import { t } from "../i18n";
import type { CollectionsError, Dive, TagError } from "../types";

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
  if (s.needsValues && (!s.current || !s.freshCounts))
    return t("tag.perPhoto.reading");
  if (s.withPlace && !s.usesDiveSite && !s.pickedSite)
    return t("tag.chooseSite");
  if (s.gpsError) return t("tag.badGps");
  return null;
}

/** The bar's text: the blocker, else what will be written. */
export function tagSummary(
  blocker: string | null,
  chosen: Dive | null,
  placeName: string | null | undefined,
): string {
  if (blocker) return blocker;
  const diveLabel = chosen
    ? String(chosen.number ?? chosen.site.name ?? "—")
    : "";
  if (chosen && placeName)
    return t("tag.sumDiveAndPlace", { n: diveLabel, site: placeName });
  if (chosen) return t("tag.sumDive", { n: diveLabel });
  return t("tag.sumPlace", { site: placeName ?? "" });
}

export function canApplyTag(blocker: string | null, writing: boolean): boolean {
  return blocker === null && !writing;
}

/** Nothing was written because the selection is no longer the one described: it is described again. */
export function selectionChanged(e: TagError): boolean {
  if (e.error === "lightroom")
    return e.reason === "selectionChanged" || e.reason === "staleSelection";
  return e.error === "noPhotos";
}

/**
 * The selection is described again after a failed Apply: when the selection moved on, or when Lightroom
 * answered while per-photo values were asked (the offsets applied may now be the remembered ones).
 */
export function describeAgain(e: TagError, needsValues: boolean): boolean {
  return selectionChanged(e) || (e.error === "lightroom" && needsValues);
}

/** Why the selection could not be described; null: nothing to say (a newer description replaced this one, or the log changed). */
export function describeFailure(e: CollectionsError): string | null {
  switch (e.error) {
    case "noPhotos":
    case "logChanged": // the app reads the log again and starts the screen again
      return null;
    case "notConnected":
    case "linkLost":
      return t("tag.disconnected");
    case "lightroom":
      return e.detail
        ? t("tag.perPhoto.failed", { detail: e.detail })
        : t("tag.perPhoto.failedNoDetail");
    case "source":
      return t("tag.perPhoto.failed", { detail: e.detail.detail });
    default:
      return t("tag.perPhoto.failedNoDetail");
  }
}

/** What the screen does when describing the selection failed. */
export interface AfterFailure {
  reading: boolean;
  message: string | null;
  /** Describe the selection once more (the answer was lost, not the selection). */
  describeAgain: boolean;
}

/**
 * The state after a failed description of the selection. null: a newer description replaced this one
 * and owns the reading, nothing changes. Otherwise the reading always ends. A lost description
 * (noPhotos) is asked again once per selection (`retried`: already done); the second time it says why.
 * A changed log needs nothing: the app reads it again and starts the screen again.
 */
export function readingAfterFailure(
  e: CollectionsError,
  live: boolean,
  retried = false,
): AfterFailure | null {
  if (!live) return null;
  if (e.error === "noPhotos") {
    return retried
      ? {
          reading: false,
          message: t("tag.perPhoto.failedNoDetail"),
          describeAgain: false,
        }
      : { reading: false, message: null, describeAgain: true };
  }
  return { reading: false, message: describeFailure(e), describeAgain: false };
}

/** The place written is another site, one is picked, and the position typed is not valid: Apply is refused. */
export function gpsRefused(s: {
  withPlace: boolean;
  usesDiveSite: boolean;
  siteId: string | null;
  gpsValid: boolean;
}): boolean {
  return s.withPlace && !s.usesDiveSite && s.siteId !== null && !s.gpsValid;
}
