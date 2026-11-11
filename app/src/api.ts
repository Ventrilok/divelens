import { invoke } from "@tauri-apps/api/core";
import type { DiveLog, Sample } from "./types";

export const loadDives = () => invoke<DiveLog>("load_dives");
export const diveProfile = (id: string) => invoke<Sample[]>("dive_profile", { id });
export const setSourcePath = (path: string) => invoke<void>("set_source_path", { path });
export const versions = () => invoke<[string, string]>("versions");
