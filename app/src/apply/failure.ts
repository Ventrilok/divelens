// Why a search or a write of Tag Selection or Create Collections failed, in words
// for the diver: one translator for both screens. `feature` chooses the texts that
// belong to each screen ("selection" or "photos found"); both name the same Undo
// step, "Dive Lens" (the plug-in's CatalogWrite.UNDO_NAME).

import { t } from "../i18n";
import type { ApplyError } from "../types";

export type ApplyFeature = "tag" | "collections";

/** A reason this version does not know: say what it can, never the code. */
function unknownReason(detail: string | null, feature: ApplyFeature): string {
  if (feature === "tag") return detail ? t("tag.err.other", { detail }) : t("tag.err.lightroom");
  return detail ? t("coll.err.other", { detail }) : t("coll.err.lightroom");
}

/** Why Apply did not (or not entirely) write. */
export function applyFailure(e: ApplyError, feature: ApplyFeature): string {
  const tag = feature === "tag";
  switch (e.error) {
    case "source":
      return tag ? t("tag.err.source", { detail: e.detail.detail }) : t("coll.err.source", { detail: e.detail.detail });
    case "notConnected":
      return tag ? t("tag.err.notConnected") : t("coll.err.notConnected");
    case "linkLost":
      return tag ? t("tag.err.linkLost") : t("coll.err.linkLost");
    case "noPhotos":
      // Tag: the selection must be described again. Collections: the search is gone.
      return tag ? t("tag.err.selectionChanged") : t("coll.err.noPhotos");
    case "nothingChosen":
      // Tag: a dive or "no dive" and a site are chosen before Apply, so the options ticked
      // are what leaves nothing to write. Collections: only a search refuses so.
      return tag ? t("journey.review.nothingWithOptions") : t("coll.err.nothingChosen");
    case "logChanged":
      // Not seen: the app reads the log again and says why.
      return t("log.changedNothingWritten");
    case "nothingToWrite":
      // Collections: no photo in the dives, or options that leave nothing on those found;
      // the refusal does not tell which, and this text holds for both.
      return tag ? t("tag.err.lightroom") : t("journey.review.nothingWithOptions");
    case "badGps":
      // Another site's position (both paths): the same refusal.
      return t("tag.err.badGps");
    case "unknownDive":
      return tag ? t("tag.err.unknownDive") : t("coll.err.lightroom");
    case "unknownSite":
      return t("tag.err.unknownSite");
    case "lightroom":
      break;
  }
  // The plug-in's reasons ("applyFailed"). A reason sent for the other feature only is an unknown one.
  switch (e.reason) {
    case "selectionChanged": // Tag: nothing written, the selection is no longer the one described
    case "staleSelection":
      if (tag) return t("tag.err.selectionChanged");
      break;
    case "noSelection": // Tag only
      if (tag) return t("tag.err.noSelection");
      break;
    case "busy":
      return tag ? t("tag.err.busy") : t("coll.err.busy");
    case "gpsNotWritten":
      // Written but for the GPS and altitude, on either path: Undo, then apply again.
      return e.detail ? t("tag.err.gpsNotWrittenDetail", { detail: e.detail }) : t("tag.err.gpsNotWritten");
    case "staleSearch": // Collections only
      if (!tag) return t("coll.err.noPhotos");
      break;
    case "parentNotFound": // Collections, on either path
      return t("coll.parentMissing", { path: e.detail ?? "" });
    case "badRequest":
      // Its detail is internal ("bad group"): never shown.
      return tag ? t("tag.err.badRequest") : t("coll.err.badRequest");
    case "error":
      break; // Lightroom's own message is the detail: worded as an unknown reason
  }
  return unknownReason(e.detail, feature);
}

/** Catalog path: Apply was refused because the photos found are gone, so they are looked for again. */
export function searchAgain(e: ApplyError): boolean {
  return e.error === "noPhotos" || (e.error === "lightroom" && e.reason === "staleSearch");
}

/**
 * Why the photos could not be looked for or described (nothing is written by a search).
 * Tag: null when a newer description replaced this one. Collections: always a text.
 */
export function searchFailure(e: ApplyError, feature: "collections"): string;
export function searchFailure(e: ApplyError, feature: ApplyFeature): string | null;
export function searchFailure(e: ApplyError, feature: ApplyFeature): string | null {
  if (feature === "tag") {
    switch (e.error) {
      case "noPhotos":
      case "logChanged": // the app reads the log again and starts the screen again
        return null;
      case "notConnected":
      case "linkLost":
        return t("tag.disconnected");
      case "lightroom":
        return e.detail ? t("tag.perPhoto.failed", { detail: e.detail }) : t("tag.perPhoto.failedNoDetail");
      case "source":
        return t("tag.perPhoto.failed", { detail: e.detail.detail });
      default:
        return t("tag.perPhoto.failedNoDetail");
    }
  }
  switch (e.error) {
    case "notConnected":
      return t("coll.search.err.notConnected");
    case "linkLost":
      return t("coll.search.err.linkLost");
    case "nothingToWrite":
      return t("coll.err.nothingToWrite");
    case "nothingChosen":
      return t("coll.search.err.divesGone");
    case "logChanged":
      return t("log.changed");
    case "lightroom":
      if (e.reason === "busy") return t("coll.search.err.busy");
      return e.detail ? t("coll.search.err.lightroom", { detail: e.detail }) : t("coll.search.err.generic");
    case "source":
      return t("coll.search.err.source", { detail: e.detail.detail });
    default: // noPhotos
      return t("coll.search.err.generic");
  }
}
