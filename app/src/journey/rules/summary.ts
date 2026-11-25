// Step 5 of the journey, "Summary" (spec F5): the run recalled, what was
// written, and the warnings, on both paths.

import { t, tn } from "../../i18n";
import { formatDay } from "../../ui/format";
import type { Camera, CollectionsResult, Dive, OutsidePhoto, TagApplied } from "../../types";
import type { Journey } from "../state";
import { diveLabel } from "../texts";
import { bySide, chosenDive, siteDetails, tickedDives, writeOptions } from "./shared";

/** What Apply wrote, as the summary shows it. */
export type Applied =
  | { path: "catalog"; result: CollectionsResult; outside: number; cameras: Camera[]; ignored: string[] }
  | {
      path: "selection";
      result: TagApplied;
      total: number;
      outside: OutsidePhoto[];
      outsideMore: number;
      /** Per-photo values were written (dive details ticked, a profile that could be read). */
      perPhoto: boolean;
    };

export interface SummaryInput {
  journey: Journey;
  applied: Applied;
  /** The log's dives: the dives ticked or chosen are among them. */
  dives: Dive[];
}

export interface Summary {
  head: string;
  stats: { label: string; n: number }[];
  warnings: string[];
}

/** The options ticked, as the summary's head names them. */
function optionNames(j: Journey): string {
  const o = writeOptions(j);
  return [
    o.dive ? t("journey.summary.optDive") : null,
    siteDetails(o) ? t("journey.summary.optSite") : null,
    o.collections ? t(j.path === "selection" ? "journey.summary.optCollection" : "journey.summary.optCollections") : null,
  ]
    .filter(Boolean)
    .join(", ");
}

const stats = (photos: number, withDepth: number, collections: number) => [
  { label: t("coll.result.photos"), n: photos },
  { label: t("coll.result.withDepth"), n: withDepth },
  { label: t(collections === 1 ? "journey.summary.collection" : "coll.result.collections"), n: collections },
];

function catalogSummary(j: Journey, a: Extract<Applied, { path: "catalog" }>, all: Dive[]): Summary {
  const dives = tickedDives(j, all);
  const period =
    dives.length > 0
      ? t("journey.summary.period", {
          from: formatDay(dives[0].start),
          to: formatDay(dives.reduce((end, d) => (d.end > end ? d.end : end), dives[0].end)),
        })
      : null;
  const cameras = tn("coll.cameras", a.cameras.filter((c) => !a.ignored.includes(c.key)).length);
  const byId = new Map(all.map((d) => [d.id, d]));
  const warnings = a.result.emptyDives
    .map((id) => byId.get(id))
    .filter((d): d is Dive => d !== undefined)
    .map((d) => t("coll.result.empty", { dive: diveLabel(d) }));
  if (a.outside > 0) warnings.push(tn("journey.summary.outside", a.outside));
  return {
    head: [period, cameras, optionNames(j)].filter(Boolean).join(" · "),
    stats: stats(a.result.photos, a.result.withDepth, a.result.collections.length),
    warnings,
  };
}

function selectionSummary(j: Journey, a: Extract<Applied, { path: "selection" }>, all: Dive[]): Summary {
  const dive = chosenDive(j, all);
  const warnings: string[] = [];
  const noDepth = a.result.count - a.result.withDepth;
  if (a.perPhoto && noDepth > 0) {
    const sides = bySide(noDepth, a.outside, a.outsideMore);
    if (!sides) warnings.push(tn("journey.summary.noDepth", noDepth));
    else {
      if (sides.before.length > 0) warnings.push(tn("journey.summary.noDepthBefore", sides.before.length));
      if (sides.after.length > 0) warnings.push(tn("journey.summary.noDepthAfter", sides.after.length));
    }
  }
  return {
    head: [
      tn("journey.photos.selected", a.result.count),
      dive ? t("journey.line.dive", { dive: diveLabel(dive) }) : null,
      optionNames(j),
    ]
      .filter(Boolean)
      .join(" · "),
    stats: stats(a.result.count, a.result.withDepth, a.result.collection ? 1 : 0),
    warnings,
  };
}

export function summary(i: SummaryInput): Summary {
  const a = i.applied;
  return a.path === "catalog" ? catalogSummary(i.journey, a, i.dives) : selectionSummary(i.journey, a, i.dives);
}
