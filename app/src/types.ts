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
  /** Marked invalid in the dive log (Subsurface): listed with a badge. */
  invalid: boolean;
  /** The diver's own notes, line breaks kept. */
  notes: string | null;
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

/** A dive application, by its id ("macDive", "subsurface"…): see DiveApp. */
export type SourceKind = string;

/** A reader error as the screens tell it. */
export interface Problem {
  title: string;
  text: string;
}

/** What the screens say of a dive application, in one language (core/src/apps.rs, Texts). */
export interface AppTexts {
  /** What the log is called ("MacDive database"), the title of its location in Settings. */
  log: string;
  logHint: string;
  /** While the log is read. */
  reading: string;
  /** Once the log is read, by the format the reader reports (DiveLog.format); {dives} and {sites} for the counts. */
  read: Record<string, string>;
  /** File chooser: the kind of file shown. */
  fileType: string;
  /** Error screen: the button choosing the log elsewhere when it is a file or a folder, not either; null: "Choose…". */
  locate: string | null;
  /** By reader error kind: notFound, notADiveLog and database at least; any other kind reads as database. */
  problems: Record<string, Problem>;
}

/** A dive application available on this system, as the back end describes it (core/src/apps.rs). */
export interface DiveApp {
  id: SourceKind;
  /** Its name, as shown and written on photos. */
  name: string;
  /** The log is a file with one of these extensions (null: never a file), and/or a folder. */
  choose: { file: string[] | null; folder: boolean };
  texts: { en: AppTexts; fr: AppTexts };
}

export interface SourceInfo {
  kind: SourceKind;
  path: string;
  isDefaultPath: boolean;
}

/** The dive log a screen shows, named by every action on it (application and location). */
export interface LogRef {
  kind: SourceKind;
  path: string;
}

export interface DiveLog {
  source: SourceInfo;
  /** How the log is stored, as its reader tells (a key of AppTexts.read). */
  format: string;
  capabilities: Capabilities;
  dives: Dive[];
  /** Every named site of the log, including sites with no dive yet. */
  sites: Site[];
  /** Units in use: the dive application's unless forced in Settings (set by the app). */
  units: Units;
  /** Units of the dive application's preferences. */
  sourceUnits: Units;
}

export type ReaderError =
  | { kind: "notFound"; detail: string }
  | { kind: "notADiveLog"; detail: string }
  | { kind: "database"; detail: string }
  | { kind: "oldFormat"; detail: string }
  | { kind: "severalLogs"; detail: string }
  | { kind: "unknownDive"; detail: string }
  | { kind: "badProfile"; detail: string };

export interface LoadError {
  source: SourceInfo;
  error: ReaderError;
}

/** Why a dive profile was not read; logChanged: the screen shows another log than the configured one. */
export type ProfileError = { error: "logChanged" } | { error: "source"; detail: ReaderError };

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
  | { outcome: "applied"; count: number; altitudesCleared: number }
  | { outcome: "failed"; reason: "noSelection" | "busy" | "badRequest" | "error" | string; detail: string | null };

export type TagError =
  | { error: "nothingChosen" | "notConnected" | "linkLost" | "badGps" | "unknownDive" | "unknownSite" | "logChanged" }
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
  /** Number of the search: the counts, Suggest and Apply name it (a newer search replaces it). */
  search: number;
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
  | { error: "nothingChosen" | "notConnected" | "linkLost" | "noPhotos" | "nothingToWrite" | "logChanged" }
  | { error: "source"; detail: ReaderError }
  | { error: "lightroom"; reason: string; detail: string | null };

// --- Settings (app/src-tauri/src/settings.rs) -------------------------------

export type Sort = "recent" | "oldest" | "depth";

export interface Settings {
  version: number;
  /** Application chosen, and the location chosen for each, by id (none: its standard location). */
  source: { kind: SourceKind; locations: Record<SourceKind, string> };
  units: { depth: "source" | "m" | "ft"; temp: "source" | "c" | "f" };
  language: "system" | "en" | "fr";
  depthToAltitude: boolean;
  /** Where Create Collections puts the collections. */
  collections: { parent: string[]; group: string };
  logLevel: "normal" | "detailed";
  diveSort: Sort | null;
  legacyImported: boolean;
}

/** A settings file's dive log on this computer (spec F11). */
export interface ImportCheck {
  source: SourceInfo;
  /** The dive log is at that location. */
  found: boolean;
}

/** Why a settings file was not imported. */
export type ImportError =
  | { kind: "unreadable"; detail: string }
  | { kind: "notSettings" }
  | { kind: "notSaved"; detail: string }
  | { kind: "writeInProgress" };

/** Why a setting was not changed; writeInProgress: it would switch the dive log during a write to Lightroom. */
export type ChangeError = { kind: "notSaved"; detail: string } | { kind: "writeInProgress" };

/** A collection set of the Lightroom catalog. */
export interface CollectionSet {
  name: string;
  children: CollectionSet[];
}

export interface AppInfo {
  version: string;
  os: string;
  /** The app's log file, next to the plugin's. */
  logPath: string | null;
}
