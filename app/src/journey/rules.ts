// The journey's rules (spec F5), pure and tested: when "Next" is allowed and
// what is sent to the engine here; step 4 (what greys "Apply", the effect of
// each option) in rules/review.ts and step 5 in rules/summary.ts, re-exported
// so that the screens import every rule from this module.

import { t } from "../i18n";
import type { Dive, LightroomStatus, LocationChoice, TagChoice } from "../types";
import { divesChosen, type Journey } from "./state";
import { pickedLocation, siteDetails, siteSource, writeOptions } from "./rules/shared";
import type { SelectionReview } from "./rules/review";

export { chosenDive, siteDetails, siteGives, siteSource, tickedDives, writeOptions } from "./rules/shared";
export {
  applyCount,
  barSummary,
  canApply,
  effects,
  goodToKnow,
  gpsError,
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

/** Whether "Next" can be clicked, and the prompt shown while it cannot (null: none). */
export interface NextState {
  allowed: boolean;
  prompt: string | null;
}

/**
 * What greys "Next". Step 1 without a path chosen: greyed without a prompt (the
 * specification gives none).
 */
export function canGoNext(j: Journey, lr: Pick<LightroomStatus, "selected">): NextState {
  const greyed = (prompt: string | null): NextState => ({ allowed: false, prompt });
  if (j.step === 1) {
    if (j.path === null) return greyed(null);
    if (j.path === "selection" && lr.selected === 0) return greyed(t("tag.selectInLr"));
  }
  if (j.step === 2 && !divesChosen(j)) return greyed(t(j.path === "catalog" ? "sel.none" : "tag.chooseDive"));
  return { allowed: true, prompt: null };
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
  /** The selection's reading, the same one reviewBlocker checked. */
  selection: Pick<SelectionReview, "needsValues" | "current">;
  /** Clock offsets shown, in minutes by camera key. */
  offsets: Record<string, number>;
}

/**
 * Selection path. The dive is sent even with its details unticked (it gives the site
 * and the collection's name). The current description of the selection and its
 * offsets only with per-photo values written; otherwise Apply writes on the
 * selection of the moment. The location only with location or GPS ticked.
 */
export function tagChoice(i: TagChoiceInput): TagChoice {
  const j = i.journey;
  const options = writeOptions(j);
  const described = i.selection.current?.search ?? null;
  const perPhoto = options.dive && i.selection.needsValues && described !== null;
  const location: LocationChoice | null = !siteDetails(j.options)
    ? null
    : siteSource(j, i.dive) === "diveSite"
      ? { kind: "diveSite" }
      : pickedLocation(j.options);
  return {
    diveId: j.dive.kind === "dive" ? j.dive.id : null,
    location,
    search: perPhoto ? described : null,
    offsets: perPhoto ? i.offsets : {},
    options,
  };
}
