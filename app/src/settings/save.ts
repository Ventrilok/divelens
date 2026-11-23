import type { Settings } from "../types";

// A setting change is shown at once, before it is saved. If saving fails, the
// screen shows the last saved value again, for each setting this change set
// and nothing has changed since.

const same = (a: unknown, b: unknown) => JSON.stringify(a) === JSON.stringify(b);

/** The screen after a failed save of `patch`: each setting the patch set that
 *  still holds the patch's value goes back to its last saved value. Any other
 *  setting is left alone. A dive log change is never shown before it is saved,
 *  so there is nothing to undo. */
export function revertChange(current: Settings, patch: Partial<Settings>, saved: Settings): Settings {
  if (patch.source) return current;
  const back: Record<string, unknown> = {};
  for (const key of Object.keys(patch) as (keyof Settings)[]) {
    if (same(current[key], patch[key])) back[key] = saved[key];
  }
  return Object.keys(back).length ? { ...current, ...back } : current;
}
