// The journey's shell (spec F5): what it hands the two photo readings, which
// are mounted once for the whole journey, and what the step strip allows.
// Pure, so that each binding is tested at each step of each path.

import type { Dive } from "../types";
import { chosenDive, type Applied } from "./rules";
import type { Journey, Step } from "./state";

/**
 * The dives the catalog path's photo search looks for: those ticked, at steps 3 and 4
 * only. None before, or every tick at step 2 would start a search; none on the summary,
 * so that "handle separately" starts from an idle search and shows no card of the run
 * before (and Apply never takes the counts of an idle search: it is at step 4).
 */
export function searchedDives(j: Journey): string[] {
  return j.path === "catalog" && (j.step === 3 || j.step === 4) ? [...j.dives] : [];
}

/** The selection is read photo by photo on its path, from step 2 on (choice no. 3). */
export function selectionReads(j: Journey): boolean {
  return j.path === "selection" && j.step >= 2;
}

/** The dive the selection is read for: the one chosen, whatever the options say; none on the catalog path. */
export function readingDive(j: Journey, dives: Dive[]): Dive | null {
  return j.path === "selection" ? chosenDive(j, dives) : null;
}

/** The steps already done lead back, as "Back" does: never from the summary, never while Lightroom is written to. */
export function stripLeadsBack(j: Journey, writing: boolean): boolean {
  return j.step !== 5 && !writing;
}

/**
 * The collection set chosen in Settings is looked up again on each arrival at step 3
 * from step 2, as 2.1.1 did on each entry to its adjust step: a set deleted in Lightroom
 * meanwhile is noticed before Apply. Entering steps 2 to 4 from elsewhere starts a
 * lookup anyway.
 */
export function looksUpSetAgain(was: Step, now: Step): boolean {
  return was === 2 && now === 3;
}

/** What Apply wrote, kept with the run it belongs to. */
export interface Written {
  run: number;
  applied: Applied;
}

/** What the summary shows: what this run wrote; nothing once a run starts again. */
export function appliedFor(written: Written | null, j: Journey): Applied | null {
  return written && written.run === j.run ? written.applied : null;
}

/** The dive list fills the screen at step 2; the other steps scroll. */
export function screenClass(j: Journey): string {
  return j.step === 2 ? "screen" : "screen screen-scroll";
}
