import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { isLogChanged } from "./diveLog";
import type {
  AppInfo,
  ApplyOutcome,
  CollectionSet,
  CollectionChoices,
  CollectionsPreview,
  CollectionsResult,
  DiveApp,
  DiveLog,
  DiveSpan,
  ImportCheck,
  LightroomStatus,
  LogRef,
  Period,
  Sample,
  SelectionPhotos,
  SelectionPreview,
  Settings,
  TagChoice,
  TagPreview,
} from "./types";

let logChanged = (_applied: boolean) => {};
/**
 * Calls `handler` when the back end refuses an action because the dive log is
 * no longer the one shown; `applied`: the action was an Apply.
 */
export const onLogChanged = (handler: (applied: boolean) => void) => {
  logChanged = handler;
};
/** An action on the dive log `log`: refused (rejects with logChanged) when it is no longer the configured one. */
function onLog<T>(call: Promise<T>, applied = false): Promise<T> {
  return call.catch((e: unknown) => {
    if (isLogChanged(e)) logChanged(applied);
    throw e;
  });
}

export const loadDives = () => invoke<DiveLog>("load_dives");
/** Profile of a dive of `log`; rejects with a ProfileError. */
export const diveProfile = (log: LogRef, id: string) => onLog(invoke<Sample[]>("dive_profile", { log, id }));
/** Points DiveLens at another dive log; null: the application's standard location. Rejects with a ChangeError. */
export const setSourcePath = (path: string | null) => invoke<Settings>("set_source_path", { path });
export const appInfo = () => invoke<AppInfo>("app_info");
/** The dive applications available on this system, in the order Settings lists them. */
export const diveApps = () => invoke<DiveApp[]>("dive_apps");

export const getSettings = () => invoke<Settings>("get_settings");
/** Changes the settings named in `patch`; resolves with all of them, rejects with a ChangeError. */
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
/** Whether Lightroom is being written to (the dive log cannot change meanwhile). */
export const lightroomWriting = () => invoke<boolean>("lightroom_writing");
export const onLightroomWriting = (handler: (writing: boolean) => void): Promise<UnlistenFn> =>
  listen<boolean>("lightroom-writing", (e) => handler(e.payload));
export const initialScreen = () => invoke<string | null>("initial_screen");

/** Calls `handler` on every change of the Lightroom link. */
export const onLightroomStatus = (handler: (s: LightroomStatus) => void): Promise<UnlistenFn> =>
  listen<LightroomStatus>("lightroom-status", (e) => handler(e.payload));

/** Calls `handler` when a Lightroom menu asks for a screen. */
export const onShowScreen = (handler: (screen: string) => void): Promise<UnlistenFn> =>
  listen<string>("show-screen", (e) => handler(e.payload));

/** Tags the photos selected in Lightroom with data of `log`; rejects with a TagError. */
export const tagSelection = (log: LogRef, choice: TagChoice) =>
  onLog(invoke<ApplyOutcome>("tag_selection", { log, choice }), true);

/**
 * Describes the photos selected in Lightroom now, with the offsets
 * remembered for the application of `log`; `search` numbers it, higher than
 * any asked for before. Rejects with a CollectionsError.
 */
export const findSelectionPhotos = (log: LogRef, search: number) =>
  onLog(invoke<SelectionPhotos>("find_selection_photos", { log, search }));
/** Tag Selection's live counts for description `search`; rejects with noPhotos once a newer one replaced it. */
export const tagPreview = (search: number, dive: DiveSpan | null, offsets: Record<string, number>) =>
  invoke<TagPreview>("tag_preview", { search, dive, offsets });
/** Suggested offset (minutes) per camera key of description `search`, for `dive`; null when no offset fits. */
export const suggestTagOffsets = (search: number, dive: DiveSpan, cameras: string[]) =>
  invoke<Record<string, number | null>>("suggest_tag_offsets", { search, dive, cameras });

/** Progress of a catalog write: (photos done, total). */
export const onApplyProgress = (handler: (done: number, total: number) => void): Promise<UnlistenFn> =>
  listen<[number, number]>("apply-progress", (e) => handler(e.payload[0], e.payload[1]));

export const selectionPreview = () => invoke<SelectionPreview>("selection_preview");

/** Calls `handler` each time a thumbnail of the selection arrives. */
export const onSelectionPreview = (handler: (p: SelectionPreview) => void): Promise<UnlistenFn> =>
  listen<SelectionPreview>("selection-preview", (e) => handler(e.payload));

/**
 * Finds in Lightroom the photos of the period of dives of `log`; `search`
 * numbers it, higher than any search asked for before. Rejects with a CollectionsError.
 */
export const findCollectionPhotos = (log: LogRef, search: number, diveIds: string[]) =>
  onLog(invoke<Period>("find_collection_photos", { log, search, diveIds }));
/** Live counts of photo search `search`; rejects with noPhotos once a newer search replaced it. */
export const collectionsPreview = (search: number, choices: CollectionChoices) =>
  invoke<CollectionsPreview>("collections_preview", { search, choices });
/** Suggested offset (minutes) per camera key of photo search `search`; null when no offset fits. */
export const suggestOffsets = (search: number, cameras: string[]) =>
  invoke<Record<string, number | null>>("suggest_offsets", { search, cameras });
/** Writes the dives found with photo search `search`, which must come from `log`; rejects with a CollectionsError. */
export const applyCollections = (log: LogRef, search: number, choices: CollectionChoices) =>
  onLog(invoke<CollectionsResult>("apply_collections", { log, search, choices }), true);
