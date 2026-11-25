// Steps 1 and 2 of the journey, "Photos" and "Dives" (spec F5), as the screens
// show them: the two cards of step 1 and its bar, the head of the dive list
// and the bar of step 2 on either path, and the explanation under "Covers the
// selection". Pure, so that every text is tested in both languages.

import { offsetField } from "../apply/cameras";
import { formatInt, t, tn } from "../i18n";
import { formatDay } from "../ui/format";
import type { Dive, LightroomStatus, SelectionPhotos } from "../types";
import { canGoNext, chosenDive, tickedDives } from "./rules";
import type { Journey } from "./state";
import { diveLabel, selectionSummary, type SelectionShown } from "./texts";

/** The selection as step 1 and step 2 see it, live. */
export type LiveSelection = Pick<LightroomStatus, "connected" | "selected"> & SelectionShown;

/** "Entire catalog": its title, what it does, and what it is for. */
export function catalogCard(lr: Pick<LightroomStatus, "selected">): { title: string; text: string; sub: string } {
  const sub = [t("journey.photos.catalogUse"), lr.selected > 0 ? t("journey.photos.selectionIgnored") : null];
  return {
    title: t("journey.photos.catalog"),
    text: t("journey.photos.catalogText"),
    sub: sub.filter(Boolean).join(" "),
  };
}

/**
 * "The N photos selected in Lightroom": the selection's cameras and day, then what the
 * path does. Greyed while nothing is selected (Lightroom away included), lit up as soon
 * as photos are (spec F5, Review Focus 2).
 */
export function selectionCard(lr: LiveSelection): { enabled: boolean; title: string; lines: string[]; sub: string | null } {
  const n = lr.connected ? lr.selected : 0;
  if (n === 0) {
    return {
      enabled: false,
      title: t("journey.photos.selectionNone"),
      lines: [t("journey.photos.noSelection")],
      sub: t("journey.photos.noSelectionText"),
    };
  }
  return {
    enabled: true,
    title: tn("journey.photos.selection", n),
    lines: [selectionSummary(lr), tn("journey.photos.selectionText", n)].filter(Boolean),
    sub: null,
  };
}

/** The bar of step 1: the choice and the step it leads to; or what greys "Next". */
export function photosBar(
  j: Journey,
  lr: Pick<LightroomStatus, "selected">,
): { choice: string | null; next: string | null; prompt: string | null } {
  const go = canGoNext(j, lr);
  if (!go.allowed || j.path === null) return { choice: null, next: null, prompt: go.prompt };
  return j.path === "catalog"
    ? { choice: t("journey.photos.catalog"), next: t("journey.photos.nextCatalog"), prompt: null }
    : { choice: tn("journey.photos.selected", lr.selected), next: t("journey.photos.nextSelection"), prompt: null };
}

/**
 * The bar of step 2, catalog path: "N dives selected" (with those the filters hide, as
 * before) and their days; "Check one or more dives" while none is ticked.
 */
export function catalogDivesBar(
  j: Journey,
  dives: Dive[],
  shownIds: Set<string>,
): { count: string | null; hidden: string | null; span: string | null; prompt: string | null } {
  const ticked = tickedDives(j, dives);
  if (ticked.length === 0) return { count: null, hidden: null, span: null, prompt: canGoNext(j, { selected: 0 }).prompt };
  const hidden = ticked.filter((d) => !shownIds.has(d.id)).length;
  const from = ticked.reduce((a, d) => (d.start < a ? d.start : a), ticked[0].start);
  const to = ticked.reduce((a, d) => (d.end > a ? d.end : a), ticked[0].end);
  return {
    count: tn("sel.count", ticked.length),
    hidden: hidden > 0 ? tn("sel.hidden", hidden) : null,
    span: t("journey.dives.span", { from: formatDay(from), to: formatDay(to) }),
    prompt: null,
  };
}

/** Selection path, above the list: what choosing a dive does; without photos selected, what is missing. */
export function selectionHead(lr: Pick<LightroomStatus, "connected" | "selected">): string {
  if (!lr.connected) return t("tag.disconnected");
  if (lr.selected === 0) return t("tag.selectInLr");
  return tn("journey.dives.selectionHead", lr.selected);
}

/** The bar of step 2, selection path: the dive chosen and that the photos will be attached to it. */
export function selectionDivesBar(
  j: Journey,
  dives: Dive[],
  lr: Pick<LightroomStatus, "selected">,
): { choice: string | null; sub: string | null; prompt: string | null } {
  if (j.dive.kind === "noDive") return { choice: t("journey.dives.noDive"), sub: null, prompt: null };
  const dive = chosenDive(j, dives);
  if (!dive) return { choice: null, sub: null, prompt: canGoNext(j, lr).prompt };
  return { choice: diveLabel(dive), sub: lr.selected > 0 ? tn("journey.dives.willAttach", lr.selected) : null, prompt: null };
}

/**
 * Under the dive marked "Covers the selection": "Covers the selection: 4 of the 5 photos
 * fall in this dive, with the remembered offset of the Olympus TG-6 (+4:00)." The mark
 * covers every photo with a capture time, so those are the ones counted. Only where the
 * sentence holds: the selection read, one camera, and its remembered offset in use;
 * otherwise the mark alone, as before.
 */
export function coversNote(
  covering: Dive | null,
  described: SelectionPhotos | null,
  offsets: Record<string, number>,
  touched: Set<string>,
): string | null {
  if (!covering || !described || described.cameras.length !== 1) return null;
  const camera = described.cameras[0];
  const remembered = camera.remembered;
  if (remembered === null || touched.has(camera.key) || offsets[camera.key] !== remembered) return null;
  return t("journey.dives.coversWhy", {
    n: formatInt(described.photos - described.undated),
    total: formatInt(described.photos),
    camera: camera.model ?? t("coll.unknownCamera"),
    offset: offsetField(remembered),
  });
}
