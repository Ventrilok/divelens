import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ApplyOutcome, DiveLog, LightroomStatus, Sample, TagChoice } from "./types";

export const loadDives = () => invoke<DiveLog>("load_dives");
export const diveProfile = (id: string) => invoke<Sample[]>("dive_profile", { id });
export const setSourcePath = (path: string) => invoke<void>("set_source_path", { path });
export const versions = () => invoke<[string, string]>("versions");
export const lightroomStatus = () => invoke<LightroomStatus>("lightroom_status");
export const initialScreen = () => invoke<string | null>("initial_screen");

/** Calls `handler` on every change of the Lightroom link. */
export const onLightroomStatus = (handler: (s: LightroomStatus) => void): Promise<UnlistenFn> =>
  listen<LightroomStatus>("lightroom-status", (e) => handler(e.payload));

/** Calls `handler` when a Lightroom menu asks for a screen. */
export const onShowScreen = (handler: (screen: string) => void): Promise<UnlistenFn> =>
  listen<string>("show-screen", (e) => handler(e.payload));

/** Tags the photos selected in Lightroom; rejects with a TagError. */
export const tagSelection = (choice: TagChoice) => invoke<ApplyOutcome>("tag_selection", { choice });

/** Progress of a catalog write: (photos done, total). */
export const onApplyProgress = (handler: (done: number, total: number) => void): Promise<UnlistenFn> =>
  listen<[number, number]>("apply-progress", (e) => handler(e.payload[0], e.payload[1]));
