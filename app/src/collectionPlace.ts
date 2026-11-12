// Where Create Collections puts the collections (spec F2): a collection set
// of the catalog (or its root), optionally a named set inside it. Pure
// functions, no UI.

import type { CollectionSet, Settings, Units } from "./types";

export type Place = Settings["collections"];

/** One collection set as a choice: its path from the root and its depth. */
export interface SetChoice {
  path: string[];
  depth: number;
}

/** Every collection set of the tree, parents before their children. */
export function flattenSets(sets: CollectionSet[], parent: string[] = []): SetChoice[] {
  return sets.flatMap((s) => {
    const path = [...parent, s.name];
    return [{ path, depth: parent.length }, ...flattenSets(s.children, path)];
  });
}

/** True when `path` leads to a set of the tree (the root always exists). */
export function setExists(sets: CollectionSet[], path: string[]): boolean {
  let level = sets;
  for (const name of path) {
    const found = level.find((s) => s.name === name);
    if (!found) return false;
    level = found.children;
  }
  return true;
}

/** A path as one value, for a <select>. */
export const pathKey = (path: string[]) => JSON.stringify(path);

/** The group set's name as the plugin uses it: trimmed, null when blank. */
export function groupName(place: Place): string | null {
  const name = place.group.trim();
  return name === "" ? null : name;
}

/** "Trips › 2026 › Dive Lens"; `root` names the catalog's root. */
export function placeLabel(place: Place, root: string): string {
  const parts = [...place.parent];
  const group = groupName(place);
  if (group) parts.push(group);
  return parts.length === 0 ? root : parts.join(" › ");
}

/** Units in use: the dive application's unless forced in Settings. */
export function unitsInUse(choice: Settings["units"], source: Units): Units {
  return {
    depth: choice.depth === "source" ? source.depth : choice.depth,
    temp: choice.temp === "source" ? source.temp : choice.temp,
  };
}
