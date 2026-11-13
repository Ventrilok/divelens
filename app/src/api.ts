import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AppInfo,
  ApplyOutcome,
  CollectionSet,
  CollectionChoices,
  CollectionsPreview,
  CollectionsResult,
  DiveLog,
  ImportCheck,
  LightroomStatus,
  Period,
  Sample,
  SelectionPreview,
  Settings,
  TagChoice,
} from "./types";

export const loadDives = () => invoke<DiveLog>("load_dives");
export const diveProfile = (id: string) => invoke<Sample[]>("dive_profile", { id });
/** Points DiveLens at another dive log; null: the application's standard location. */
export const setSourcePath = (path: string | null) => invoke<Settings>("set_source_path", { path });
export const appInfo = () => invoke<AppInfo>("app_info");

export const getSettings = () => invoke<Settings>("get_settings");
/** Changes the settings named in `patch`; resolves with all of them. */
export const updateSettings = (patch: Partial<Settings>) => invoke<Settings>("update_settings", { patch });
/** Writes the current settings to `path`; rejects with the reason. */
export const exportSettings = (path: string) => invoke<void>("export_settings", { path });
/** Reads a settings file without importing it; rejects with an ImportError. */
export const checkSettingsFile = (path: string) => invoke<ImportCheck>("check_settings_file", { path });
/** Replaces every setting with those of the file; rejects with an ImportError. */
export const importSettings = (path: string) => invoke<ImportCheck>("import_settings", { path });
/** Calls `handler` whenever the settings change (here or in the app's back end). */
export const onSettingsChanged = (handler: (s: Settings) => void): Promise<UnlistenFn> =>
  listen<Settings>("settings-changed", (e) => handler(e.payload));
/** The catalog's collection sets; rejects with a CollectionsError. */
export const collectionSets = () => invoke<CollectionSet[]>("collection_sets");
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

export const selectionPreview = () => invoke<SelectionPreview>("selection_preview");

/** Calls `handler` each time a thumbnail of the selection arrives. */
export const onSelectionPreview = (handler: (p: SelectionPreview) => void): Promise<UnlistenFn> =>
  listen<SelectionPreview>("selection-preview", (e) => handler(e.payload));

/** Finds in Lightroom the photos of the chosen dives' period; rejects with a CollectionsError. */
export const findCollectionPhotos = (diveIds: string[]) => invoke<Period>("find_collection_photos", { diveIds });
export const collectionsPreview = (choices: CollectionChoices) =>
  invoke<CollectionsPreview>("collections_preview", { choices });
/** Suggested offset (minutes) per camera key; null when no offset fits. */
export const suggestOffsets = (cameras: string[]) =>
  invoke<Record<string, number | null>>("suggest_offsets", { cameras });
export const applyCollections = (choices: CollectionChoices) =>
  invoke<CollectionsResult>("apply_collections", { choices });
