// Dive profiles already read, for the dive detail card. Emptied each time
// the dive log is read: another log, or the same one saved again, may give
// other profiles for the same dive id.

import { diveProfile } from "./api";
import type { LogRef, Sample } from "./types";

type Fetch = (log: LogRef, id: string) => Promise<Sample[]>;

export interface ProfileCache {
  /** The profile of dive `id` of `log`, if already read since the last clear. */
  get(log: LogRef, id: string): Sample[] | undefined;
  /** Reads it from the back end and keeps it. */
  load(log: LogRef, id: string): Promise<Sample[]>;
  /** Forgets every profile (the dive log is being read again). */
  clear(): void;
}

const keyOf = (log: LogRef, id: string) => JSON.stringify([log.kind, log.path, id]);

export function profileCache(fetch: Fetch): ProfileCache {
  let cache = new Map<string, Sample[]>();
  return {
    get: (log, id) => cache.get(keyOf(log, id)),
    load: (log, id) => {
      // A read started before a clear lands in the cache it started with,
      // never in the new one.
      const into = cache;
      return fetch(log, id).then((samples) => {
        into.set(keyOf(log, id), samples);
        return samples;
      });
    },
    clear: () => {
      cache = new Map();
    },
  };
}

export const profiles = profileCache(diveProfile);
