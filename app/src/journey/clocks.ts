// Step 3 of the journey, "Clocks" (spec F5): the preview by dive, the photos
// outside the dives, and the bottom bar, on either path. Pure, so that every
// text is tested in both languages; the columns and texts of the preview are
// those of Create Collections.

import { formatInt, t, tn } from "../i18n";
import type { Camera, CameraCount, CollectionsPreview, Dive, LightroomStatus, TagPreview } from "../types";
import type { Described } from "./readings";
import { diveLabel, outsideLines } from "./texts";

/** A camera as the screens name it: its model. */
export function cameraName(c: Camera): string {
  return c.model ?? t("coll.unknownCamera");
}

/** One row of the preview by dive. */
export interface PreviewRow {
  dive: Dive;
  /** Photos going to the dive; null until counted. */
  photos: number | null;
  /** Cameras with photos going to it; 0 also until counted. */
  cameras: number;
  /** The "Per-photo data" column. */
  perPhoto: string;
  /** No photo goes to the dive: the row is in orange. */
  warn: boolean;
}

function row(dive: Dive, c: { photos: number; inDive: number; cameras: number } | undefined, unreadable: boolean): PreviewRow {
  const none = c !== undefined && c.photos === 0;
  const perPhoto = none
    ? t("coll.noPhotoHint")
    : !dive.hasProfile
      ? t("coll.noProfile")
      : unreadable
        ? t("tag.perPhoto.unreadable")
        : c && c.inDive === 0
          ? t("coll.surfaceOnly")
          : t("coll.perPhoto");
  return { dive, photos: c?.photos ?? null, cameras: c?.cameras ?? 0, perPhoto, warn: none };
}

/** Catalog path: each dive ticked, in the order given, with its live counts. */
export function catalogRows(dives: Dive[], preview: CollectionsPreview | null): PreviewRow[] {
  const counts = new Map(preview?.dives.map((c) => [c.id, c]) ?? []);
  return dives.map((d) => row(d, counts.get(d.id), false));
}

/** Selection path: the dive chosen, which every photo selected is attached to. */
export function selectionRow(dive: Dive, counts: TagPreview | null, unreadable: boolean): PreviewRow {
  const c = counts && { photos: counts.total, inDive: counts.inDive, cameras: counts.cameras.length };
  return row(dive, c ?? undefined, unreadable);
}

/** "58 Olympus TG-6 · 7 Sony A7": the photos outside of each camera that has some, in the cameras' order. */
export function outsideByCamera(cameras: Camera[], counts: CameraCount[]): string {
  const outside = new Map(counts.map((c) => [c.key, c.outside]));
  return cameras
    .filter((c) => (outside.get(c.key) ?? 0) > 0)
    .map((c) => t("journey.clocks.cameraCount", { n: formatInt(outside.get(c.key) ?? 0), camera: cameraName(c) }))
    .join(" · ");
}

/** The block under the preview about the photos outside the dives. */
export interface OutsideBlock {
  title: string;
  lines: string[];
  /** Their number by camera. */
  cameras: string;
  /** What to do with them, when there is something to say. */
  how: string | null;
}

/** Catalog path, "Outside the dives · N photos"; null before the counts, or with none outside. */
export function catalogOutside(cameras: Camera[], preview: CollectionsPreview | null): OutsideBlock | null {
  if (!preview || preview.outside === 0) return null;
  return {
    title: tn("journey.clocks.outside", preview.outside),
    lines: [t("journey.clocks.outsideText")],
    cameras: outsideByCamera(cameras, preview.cameras),
    how: t("journey.clocks.outsideHow"),
  };
}

/** Selection path, "Outside the dive's times · N photos"; null before the counts, or with every photo in the dive. */
export function selectionOutside(cameras: Camera[], counts: TagPreview | null): OutsideBlock | null {
  const n = counts ? counts.outside.length + counts.outsideMore : 0;
  if (!counts || n === 0) return null;
  return {
    title: tn("journey.clocks.outsideDive", n),
    lines: outsideLines(counts.outside, counts.outsideMore),
    cameras: outsideByCamera(cameras, counts.cameras),
    how: null,
  };
}

/** Catalog path, the bar: "N photos in N dives · N outside the dives", then each dive without photo. */
export function catalogBar(dives: Dive[], preview: CollectionsPreview | null): { total: string | null; emptyDives: string[] } {
  if (!preview) return { total: null, emptyDives: [] };
  const counts = new Map(preview.dives.map((c) => [c.id, c]));
  const withPhotos = preview.dives.filter((d) => d.photos > 0).length;
  return {
    total: t("coll.total", {
      photos: tn("coll.photos", preview.total),
      dives: tn("coll.dives", withPhotos),
      outside: formatInt(preview.outside),
    }),
    emptyDives: dives
      .filter((d) => counts.get(d.id)?.photos === 0)
      .map((d) => t("journey.clocks.emptyDive", { dive: diveLabel(d) })),
  };
}

/** What the selection's reading shows, as the step reads it (useSelectionPhotos). */
export interface SelectionShown {
  described: Described | null;
  counts: TagPreview | null;
  profile: { status: "loading" | "ok" | "error" };
  /** The dive has a profile that can be read (or is being read). */
  needsValues: boolean;
}

type Lr = Pick<LightroomStatus, "connected" | "selected">;

/**
 * What the camera cards wait for on the selection path, said in their block and in the
 * bar; null once they show, and while Lightroom is not connected (the warning above says
 * it). A failed reading is not waiting: the block shows it with "Try again".
 */
export function selectionWaiting(r: Pick<SelectionShown, "described" | "counts" | "profile">, lr: Lr): string | null {
  if (!lr.connected) return null;
  if (lr.selected === 0) return t("tag.selectInLr");
  if (r.described && r.counts) return null;
  return r.profile.status === "loading" ? t("detail.profileLoading") : t("tag.perPhoto.reading");
}

/**
 * Selection path, the bar: "N photos attached to <no. site> · N with depth", counting
 * every photo selected, those without a capture time too, and "Nothing else to set.".
 * While the cards of a dive with a profile wait, what they wait for instead.
 */
export function selectionBar(
  dive: Dive,
  r: SelectionShown,
  lr: Lr,
): { total: string | null; waiting: string | null; hint: string | null } {
  if (r.needsValues && !(r.described && r.counts)) return { total: null, waiting: selectionWaiting(r, lr), hint: null };
  const n = r.counts ? r.counts.total : lr.selected;
  const depth = r.needsValues && r.counts ? r.counts.inDive : 0;
  return {
    total: tn("journey.clocks.attached", n, { dive: diveLabel(dive), depth: formatInt(depth) }),
    waiting: null,
    hint: t("journey.clocks.nothingElse"),
  };
}
