// Mirrors the Rust common dive format (core/src/model.rs).

export interface Site {
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
