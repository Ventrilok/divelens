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
  /** Display units, from the dive application's preferences. */
  units: Units;
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
  /** Changes with each new selection; thumbnails carry it. */
  selectionVersion: number;
  /** Earliest and latest capture times, wall-clock "YYYY-MM-DDTHH:MM:SS". */
  selectionFrom: string | null;
  selectionTo: string | null;
  /** The selection's range moved by each camera's remembered clock offset. */
  coverFrom: string | null;
  coverTo: string | null;
}

/** Thumbnails of the selection received so far (data URLs, by position). */
export interface SelectionPreview {
  version: number;
  thumbnails: (string | null)[];
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

// --- Create Collections (app/src-tauri/src/collections.rs) ------------------

export interface Camera {
  key: string;
  model: string | null;
  serial: string | null;
  photos: number;
  /** Offset last applied to this camera, in minutes. */
  remembered: number | null;
}

export interface Period {
  /** First and last day searched, "YYYY-MM-DD". */
  from: string;
  to: string;
  photos: number;
  undated: number;
  cameras: Camera[];
}

export interface CollectionChoices {
  offsets: Record<string, number>;
  ignored: string[];
  marginMin: number;
}

export interface DiveCount {
  id: string;
  photos: number;
  inDive: number;
  cameras: number;
}

export interface CameraCount {
  key: string;
  inDives: number;
  outside: number;
}

export interface CollectionsPreview {
  dives: DiveCount[];
  cameras: CameraCount[];
  total: number;
  outside: number;
  ignored: number;
}

export interface WrittenCollection {
  collection: string;
  count: number;
  created: boolean;
}

export interface CollectionsResult {
  collections: WrittenCollection[];
  photos: number;
  withDepth: number;
  emptyDives: string[];
}

export type CollectionsError =
  | { error: "nothingChosen" | "notConnected" | "linkLost" | "noPhotos" | "nothingToWrite" }
  | { error: "source"; detail: ReaderError }
  | { error: "lightroom"; reason: string; detail: string | null };
