import type { Settings } from "../types";

// A setting change is shown at once. If saving it fails, the screen goes back
// to what was saved, but only for what this change touched and nobody changed since.

const same = (a: unknown, b: unknown) => JSON.stringify(a) === JSON.stringify(b);

/** The screen with the change shown, and the values it replaces. A dive log
 *  change is shown only once saved: nothing to show, nothing to undo. */
export function applyChange(
  settings: Settings,
  patch: Partial<Settings>,
): { next: Settings; previous: Partial<Settings> } {
  if (patch.source) return { next: settings, previous: {} };
  const previous: Record<string, unknown> = {};
  for (const key of Object.keys(patch)) previous[key] = settings[key as keyof Settings];
  return { next: { ...settings, ...patch }, previous: previous as Partial<Settings> };
}

/** The screen after a failed save: each setting of the change goes back to its
 *  previous value, unless it has been changed again since. */
export function revertChange(settings: Settings, patch: Partial<Settings>, previous: Partial<Settings>): Settings {
  const back: Record<string, unknown> = {};
  for (const key of Object.keys(previous) as (keyof Settings)[]) {
    if (same(settings[key], patch[key])) back[key] = previous[key];
  }
  return Object.keys(back).length ? { ...settings, ...back } : settings;
}
