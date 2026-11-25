// The journey's rules (spec F5), pure and tested: when "Next" is allowed and
// what is sent to the engine here; step 4 (what greys "Apply", the effect of
// each option) in rules/review.ts and step 5 in rules/summary.ts, re-exported
// so that the screens import every rule from this module.

import { t } from "../i18n";
import type { Dive, LightroomStatus, LocationChoice, TagChoice } from "../types";
import { divesChosen, type Journey } from "./state";
import { pickedLocation, siteDetails, siteSource, writeOptions } from "./rules/shared";

export { siteDetails, siteGives, siteSource, writeOptions } from "./rules/shared";
export {
  barSummary,
  effects,
  goodToKnow,
  reviewBlocker,
  reviewCounts,
  unitsText,
  type CatalogReview,
  type Effects,
  type ReviewCounts,
  type ReviewInput,
  type SelectionReview,
} from "./rules/review";
export { summary, type Applied, type Summary, type SummaryInput } from "./rules/summary";

/**
 * What greys "Next": the prompt shown, "" for greyed without a prompt (step 1, no
 * path chosen: the specification gives none), null when the diver may go on.
 */
export function canGoNext(j: Journey, lr: Pick<LightroomStatus, "selected">): string | null {
  if (j.step === 1) {
    if (j.path === null) return "";
    return j.path === "selection" && lr.selected === 0 ? t("tag.selectInLr") : null;
  }
  if (j.step === 2 && !divesChosen(j)) return t(j.path === "catalog" ? "sel.none" : "tag.chooseDive");
  return null;
}

/** Catalog path: one site of the log for every photo, sent only with "Other site" and location or GPS ticked. */
export function otherSite(j: Journey): LocationChoice | undefined {
  if (j.options.site !== "other" || !siteDetails(j.options)) return undefined;
  return pickedLocation(j.options) ?? undefined;
}

/** What the selection path sends, besides the journey. */
export interface TagChoiceInput {
  journey: Journey;
  /** The chosen dive, as the log has it; null for "no dive". */
  dive: Dive | null;
  /** The chosen dive has a profile that can be read (or is being read): per-photo values are computed. */
  needsValues: boolean;
  /** The number of the selection's current description; null when there is none. */
  described: number | null;
  /** Clock offsets shown, in minutes by camera key. */
  offsets: Record<string, number>;
}

/**
 * Selection path. The dive is sent even with its details unticked (it gives the site
 * and the collection's name). The described selection and its offsets only with
 * per-photo values written; otherwise Apply writes on the selection of the moment.
 * The location only with location or GPS ticked.
 */
export function tagChoice(i: TagChoiceInput): TagChoice {
  const j = i.journey;
  const options = writeOptions(j);
  const perPhoto = options.dive && i.needsValues && i.described !== null;
  const location: LocationChoice | null = !siteDetails(j.options)
    ? null
    : siteSource(j, i.dive) === "diveSite"
      ? { kind: "diveSite" }
      : pickedLocation(j.options);
  return {
    diveId: j.dive.kind === "dive" ? j.dive.id : null,
    location,
    search: perPhoto ? i.described : null,
    offsets: perPhoto ? i.offsets : {},
    options,
  };
}
