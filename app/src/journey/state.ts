// The journey's state (spec F5): the path chosen at step 1, the step shown,
// the dives ticked or chosen and the step-4 options, in one state for both
// paths. Pure, so that every rule of navigation is tested; the screens send
// actions and show the state.

import { NO_PICK, type SitePick } from "../tag/SitePicker";

export type Path = "catalog" | "selection";
export type Step = 1 | 2 | 3 | 4 | 5;
/** Selection path, step 2: a dive, "no dive — location only", or nothing chosen yet. */
export type DivePick = { kind: "none" } | { kind: "noDive" } | { kind: "dive"; id: string };

/**
 * Step 4. "Site details" has no box of its own: it is ticked while "Location"
 * or "GPS" is, and unticking it unticks both (spec F5: both unticked equal the
 * option unticked).
 */
export interface Options {
  dive: boolean;
  altitude: boolean;
  place: boolean;
  gps: boolean;
  collections: boolean;
  site: "diveSite" | "other";
  /** The site picked under "Other site", its GPS as typed. */
  pick: SitePick;
}

export interface Journey {
  step: Step;
  path: Path | null;
  /** Catalog path: the dives ticked. */
  dives: Set<string>;
  /** Selection path: the dive chosen. */
  dive: DivePick;
  options: Options;
  /** Changes when the run starts again: the photo hooks reset on it. */
  run: number;
}

/** On every run: everything ticked but the altitude, on each dive's site. */
export const defaultOptions: Options = {
  dive: true,
  altitude: false,
  place: true,
  gps: true,
  collections: true,
  site: "diveSite",
  pick: NO_PICK,
};

const NONE: DivePick = { kind: "none" };

export const start: Journey = { step: 1, path: null, dives: new Set(), dive: NONE, options: defaultOptions, run: 0 };

export type Action =
  | { type: "choosePath"; path: Path }
  | { type: "next" }
  | { type: "back" }
  | { type: "goTo"; step: Step }
  | { type: "toggleDive"; id: string }
  | { type: "setDives"; ids: string[] }
  | { type: "pickDive"; pick: DivePick }
  | { type: "setOptions"; options: Partial<Options> }
  | { type: "applied" }
  | { type: "backToDives" }
  | { type: "handleSeparately"; ids: string[] }
  | { type: "done" }
  | { type: "menu" }
  | { type: "logChanged" }
  /** The dive log was read again (same log): `ids`, every dive it holds now. */
  | { type: "logRead"; ids: string[] };

export type StepState = "done" | "on" | "next" | "skipped";

/** Selection path with "no dive — location only": the clocks are skipped. */
export function noDive(j: Journey): boolean {
  return j.path === "selection" && j.dive.kind === "noDive";
}

/** Step 2's choice is made: dives ticked (catalog), a dive or "no dive" (selection). */
export function divesChosen(j: Journey): boolean {
  return j.path === "catalog" ? j.dives.size > 0 : j.path === "selection" && j.dive.kind !== "none";
}

function nextStep(j: Journey): Step | null {
  switch (j.step) {
    case 1:
      return j.path === null ? null : 2;
    case 2:
      if (!divesChosen(j)) return null;
      return noDive(j) ? 4 : 3;
    case 3:
      return 4;
    default:
      // Step 4 moves on by Apply ("applied"); the summary is the end.
      return null;
  }
}

function previousStep(j: Journey): Step | null {
  switch (j.step) {
    case 2:
      return 1;
    case 3:
      return 2;
    case 4:
      return noDive(j) ? 2 : 3;
    default:
      return null;
  }
}

const samePick = (a: DivePick, b: DivePick) =>
  a.kind === b.kind && (a.kind !== "dive" || (b.kind === "dive" && a.id === b.id));

/** A new run of the catalog path from step 2: no dive ticked, the options back to their defaults. */
const backToDives = (j: Journey): Journey => ({
  ...j,
  step: 2,
  dives: new Set(),
  dive: NONE,
  options: defaultOptions,
  run: j.run + 1,
});

/** A new run of the same path from step 1. */
const done = (j: Journey): Journey => ({ ...start, path: j.path, run: j.run + 1 });

/** The dive log holds only `ids` now: what it lost is no longer chosen. */
function logRead(j: Journey, ids: Set<string>): Journey {
  const dives = [...j.dives].filter((id) => ids.has(id));
  const diveGone = j.dive.kind === "dive" && !ids.has(j.dive.id);
  if (dives.length === j.dives.size && !diveGone) return j;
  const next: Journey = { ...j, dives: new Set(dives), dive: diveGone ? NONE : j.dive };
  // Steps 3 and 4 need a dive of this log: back to step 2 to choose again.
  const lost = j.path === "catalog" ? dives.length === 0 : diveGone;
  return lost && (j.step === 3 || j.step === 4) ? { ...next, step: 2 } : next;
}

export function reduce(j: Journey, a: Action): Journey {
  switch (a.type) {
    case "choosePath":
      // One state for both paths: another path starts the next steps again.
      if (a.path === j.path) return j;
      return { ...j, path: a.path, dives: new Set(), dive: NONE, options: defaultOptions, run: j.run + 1 };
    case "next": {
      const to = nextStep(j);
      return to === null ? j : { ...j, step: to };
    }
    case "back": {
      const to = previousStep(j);
      return to === null ? j : { ...j, step: to };
    }
    case "goTo":
      // Only to a step already done, never from the summary, never to skipped clocks.
      if (j.step === 5 || a.step >= j.step || (a.step === 3 && noDive(j))) return j;
      return { ...j, step: a.step };
    case "toggleDive": {
      const dives = new Set(j.dives);
      if (dives.has(a.id)) dives.delete(a.id);
      else dives.add(a.id);
      return { ...j, dives };
    }
    case "setDives":
      return { ...j, dives: new Set(a.ids) };
    case "pickDive": {
      if (samePick(a.pick, j.dive)) return j;
      // The site follows the choice: another site for no dive, the dive's site for a dive.
      const site = a.pick.kind === "noDive" ? "other" : a.pick.kind === "dive" ? "diveSite" : j.options.site;
      return { ...j, dive: a.pick, options: { ...j.options, site } };
    }
    case "setOptions":
      return { ...j, options: { ...j.options, ...a.options } };
    case "applied":
      // A write ending after the journey started again shows no summary.
      return j.step === 4 ? { ...j, step: 5 } : j;
    case "backToDives":
      return backToDives(j);
    case "handleSeparately":
      return { ...backToDives(j), step: 3, dives: new Set(a.ids) };
    case "done":
      return done(j);
    case "menu":
      // "Open Dive Lens…": step 1, the choices kept; leaving the summary ends its run.
      if (j.step === 5) return j.path === "selection" ? done(j) : { ...backToDives(j), step: 1 };
      return j.step === 1 ? j : { ...j, step: 1 };
    case "logChanged":
      return { ...start, run: j.run + 1 };
    case "logRead":
      return logRead(j, new Set(a.ids));
  }
}

/** The five steps as the strip shows them; the clocks are skipped for "no dive". */
export function stepStates(j: Journey): StepState[] {
  return ([1, 2, 3, 4, 5] as const).map((n) => {
    if (n === 3 && noDive(j)) return "skipped";
    return n < j.step ? "done" : n === j.step ? "on" : "next";
  });
}
