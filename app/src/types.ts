// Mirrors the Rust common dive format (core/src/model.rs).

export interface Site {
  /** Stable id inside the dive log; null when the dive has no site. */
  id: string | null;
  name: string | null;
  location: string | null;
  country: string | null;
  lat: number | null;
  lon: number | null;
}

export interface Dive {
  id: string;
  number: number | null;
  /** Wall-clock time without time zone: "YYYY-MM-DDTHH:MM:SS". */
  start: string;
  end: string;
  durationS: number;
  maxDepthM: number | null;
  tempHighC: number | null;
  tempLowC: number | null;
  rating: number | null;
  site: Site;
  hasProfile: boolean;
}

export interface Sample {
  time_s: number;
  depth_m: number;
  temp_c: number | null;
}

export interface Capabilities {
  temperature: boolean;
  rating: boolean;
  gps: boolean;
  profile: boolean;
}

export type SourceKind = "macDive";

export interface SourceInfo {
  kind: SourceKind;
  path: string;
  isDefaultPath: boolean;
}

export interface DiveLog {
  source: SourceInfo;
  capabilities: Capabilities;
  dives: Dive[];
  /** Every named site of the log, including sites with no dive yet. */
  sites: Site[];
}

export type ReaderError =
  | { kind: "notFound"; detail: string }
  | { kind: "notADiveLog"; detail: string }
  | { kind: "database"; detail: string }
  | { kind: "unknownDive"; detail: string }
  | { kind: "badProfile"; detail: string };

export interface LoadError {
  source: SourceInfo;
  error: ReaderError;
}

export interface Units {
  depth: "m" | "ft";
  temp: "c" | "f";
}

/** State of the link with the Lightroom plugin (app/src-tauri/src/lightroom.rs). */
export interface LightroomStatus {
  connected: boolean;
  catalog: string | null;
  lightroomVersion: string | null;
  pluginVersion: string | null;
  /** Photos selected in Lightroom; 0 when nothing is selected. */
  selected: number;
}

/** What Tag Selection writes (app/src-tauri/src/lib.rs, TagChoice). */
export interface TagChoice {
  diveId: string | null;
  location: { kind: "diveSite" } | { kind: "site"; siteId: string; lat: number | null; lon: number | null } | null;
}

export type ApplyOutcome =
  | { outcome: "applied"; count: number }
  | { outcome: "failed"; reason: "noSelection" | "busy" | "badRequest" | "error" | string; detail: string | null };

export type TagError =
  | { error: "nothingChosen" | "notConnected" | "linkLost" | "badGps" | "unknownDive" | "unknownSite" }
  | { error: "source"; detail: ReaderError };
