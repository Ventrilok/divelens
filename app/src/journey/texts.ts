// The journey's composed texts (spec F5): how a dive is named, the line
// "Photos: …" under the step strip, and the photos taken outside the dive.
// Pure, so that every text is tested in both languages.

import { localized, t, tn } from "../i18n";
import { DASH, formatDay, parseWallClock } from "../ui/format";
import type { Dive, LightroomStatus, OutsidePhoto, Period } from "../types";
import { noDive, type Journey } from "./state";

/** At most this many photos outside the dive are detailed (choice no. 4). */
export const OUTSIDE_SHOWN = 5;

const longDate = localized(
  (l) => new Intl.DateTimeFormat(l, { timeZone: "UTC", weekday: "long", day: "numeric", month: "long", year: "numeric" }),
);

/** A dive as the screens name it, "<no. site>": "707 Coral Steps"; a dash when it has neither. */
export function diveLabel(d: Dive): string {
  return [d.number !== null ? String(d.number) : null, d.site.name].filter(Boolean).join(" ") || DASH;
}

/** "15:45": the hour and minute of a wall-clock time, read from the text, never shifted by a time zone. */
export function photoTime(wallClock: string): string {
  return wallClock.slice(11, 16);
}

/** The selection as Lightroom describes it live. */
export type SelectionShown = Pick<LightroomStatus, "selectionCameras" | "selectionFrom" | "selectionTo">;

/** "Olympus TG-6 · vendredi 6 mars 2026": each camera model once, then the day (first and last day over several). */
export function selectionSummary(lr: SelectionShown): string {
  const cameras = [...new Set(lr.selectionCameras.map((c) => c.model ?? t("coll.unknownCamera")))].join(", ");
  let day = "";
  if (lr.selectionFrom && lr.selectionTo) {
    day =
      lr.selectionFrom.slice(0, 10) === lr.selectionTo.slice(0, 10)
        ? longDate.format(parseWallClock(lr.selectionFrom.slice(0, 10)))
        : `${formatDay(lr.selectionFrom)} → ${formatDay(lr.selectionTo)}`;
  }
  return [cameras, day].filter(Boolean).join(" · ");
}

/** What the line "Photos: …" is made of. */
export interface PhotosLineInput {
  lr: Pick<LightroomStatus, "selected"> & SelectionShown;
  /** The log's dives: the dives ticked or chosen are among them. */
  dives: Dive[];
  /** Catalog path: the photos found for the dives ticked; null while they are looked for. */
  period: Period | null;
}

/**
 * The line under the step strip that recalls the choices made (spec F5), plain text
 * ("Back" returns to step 1). null where there is none: step 1 and the summary. Step 4
 * repeats step 3's line, and step 2's for "no dive".
 */
export function photosLine(j: Journey, i: PhotosLineInput): string | null {
  if (j.step === 1 || j.step === 5 || j.path === null) return null;
  const lr = i.lr;
  if (j.path === "catalog") {
    if (j.step === 2) return t("journey.line.catalogDives");
    const dives = i.dives.filter((d) => j.dives.has(d.id));
    const ticked = tn("journey.line.divesTicked", dives.length);
    const p = i.period;
    if (!p || dives.length === 0) return [t("journey.line.catalog"), ticked].join(" · ");
    // The days of the dives ticked, not the period searched, which is wider.
    const from = dives.reduce((a, d) => (d.start < a ? d.start : a), dives[0].start);
    const to = dives.reduce((a, d) => (d.end > a ? d.end : a), dives[0].end);
    return [
      t("journey.line.catalogPeriod", { from: formatDay(from), to: formatDay(to) }),
      t("journey.line.photosFrom", { photos: tn("coll.photos", p.photos), cameras: tn("coll.cameras", p.cameras.length) }),
      ticked,
    ].join(" · ");
  }
  const selected = tn("journey.line.selection", lr.selected);
  if (j.step === 2 || noDive(j)) return [selected, selectionSummary(lr)].filter(Boolean).join(" · ");
  const pick = j.dive;
  const dive = pick.kind === "dive" ? i.dives.find((d) => d.id === pick.id) : undefined;
  return [
    selected,
    tn("coll.cameras", lr.selectionCameras.length),
    dive ? t("journey.line.dive", { dive: diveLabel(dive) }) : null,
  ]
    .filter(Boolean)
    .join(" · ");
}

/** "Taken at 15:45, 7 minutes after getting out of the water. Attached to the dive anyway…" */
export function outsideSentence(p: OutsidePhoto): string {
  const key = p.minutes < 0 ? "journey.outside.before" : "journey.outside.after";
  return tn(key, Math.abs(p.minutes), { time: photoTime(p.time) });
}

/**
 * The block "Outside the dive's times": at most five photos detailed, then "and N more"
 * (choice no. 4). `more`: the photos outside the dive not in `outside`.
 */
export function outsideLines(outside: OutsidePhoto[], more: number): string[] {
  const shown = outside.slice(0, OUTSIDE_SHOWN);
  const rest = more + outside.length - shown.length;
  const lines = shown.map(outsideSentence);
  if (rest > 0) lines.push(tn("journey.outside.more", rest));
  return lines;
}
