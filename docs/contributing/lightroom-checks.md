# Lightroom checks

The automatic tests cannot prove that Lightroom accepts what Dive Lens writes to its catalog, or that one **Undo** takes it all back. These eleven checks are played by hand in Lightroom Classic, on a test catalog, with fictional photos.

Play them before a release that touches the plug-in or the catalog writes, and after a large refactoring. Compare the result with the previous run: a difference is a finding, even if it looks harmless.

## Set-up

1. In Lightroom Classic: **File › New Catalog…**, in a temporary folder. Never use your usual catalog.
2. **File › Import Photos and Video…**: choose the folder `docs/contributing/lightroom-checks/`, and import it with **Copy** into a folder next to the test catalog. Never use **Add**: Lightroom could write metadata into the files of the repository.
3. In Dive Lens settings: source Subsurface, dive log `core/fixtures/subsurface/fictional.xml`, metric units, altitude option off (it is turned on only in check 10).

### Playing on a development build (macOS)

The installed plug-in and the one in the repository share the identifier `com.querton.divelens`, and we do not rely on what Lightroom does with two of them.

1. Quit Lightroom.
2. Move `~/Library/Application Support/Adobe/Lightroom/Modules/DiveLens.lrplugin` out of the Modules folder (for example onto the Desktop).
3. From `app/`, build the app: `npm run tauri build -- --bundles app`.
4. Open Lightroom: **File › Plug-in Manager › Add**, choose `plugin/DiveLens.lrdevplugin`.
5. Play the checks.
6. Afterwards: remove the repository plug-in in the Plug-in Manager, quit Lightroom, and put `DiveLens.lrplugin` back in Modules.

### Windows

The installed version is enough (the reference, or a trial installer from a manual Release workflow run on a branch other than `main`).

## The photos

| Photo | Taken | Dive | Photo's own depth |
|---|---|---|---|
| A1 | 2025-06-14 09:40:00 | 101 | 18.0 m |
| A2 | 2025-06-14 09:50:00 | 101 | 15.0 m |
| A3 | 2025-06-14 10:00:00 | 101 | 10.0 m |
| B1 | 2025-06-14 14:10:00 | 102 | 12.0 m |
| B2 | 2025-06-14 14:20:00 | 102 | (no value to check) |
| X1 | 2025-06-14 12:00:00 | none (between the two dives) | none |

The fictional dive log holds two dives that day. Dive 101: 09:30 to 10:12:30, site Atlantis / North Reef / Coral Garden, GPS 12.5, -45.25. Dive 102: 14:00 to 14:35, site Glacier Wall, GPS -64.125, -60.5.

Expected for dive 101: `MaxDepth` 18.4 m, country "Atlantis", city "North Reef", location "Coral Garden", GPS 12°30'N 45°15'W.

## The checks

Each check starts from the state left by the previous one unless it says otherwise. Reset (remove the Dive Lens values, or use a fresh import) whenever a check says "clean".

| # | Start | Action | Expected |
|---|---|---|---|
| 1 | Clean. Select A1, A2, A3. | Tag the selection with dive 101, **Apply**. | Dive Lens fields filled. Dive values as above. Each photo has its own depth: A1 18.0 m, A2 15.0 m, A3 10.0 m. GPS and altitude written. |
| 2 | After 1. | **Edit › Undo** once. | Everything is gone, GPS and altitude included. |
| 3 | After 1 (redo it if needed). Select A1. | Tag it with dive 102, **Apply**. | The dive 101 values are gone from A1; it shows dive 102 values. |
| 4 | After 3. Select A1. | Tag it again with dive 102, **Apply**. | Nothing else changes. |
| 5 | Select no photo. | Try to apply. | A clear message; nothing written. |
| 6 | Select A1, open the app on that selection. | Change the selection in Lightroom (select B1), then click **Apply** in the app. | Never the values of the old selection on other photos: either a refusal with nothing written, or the app describes the new selection again and writes on the new one. |
| 7 | Clean. Select A1, A2, A3, B1, B2. | **Library › Plug-in Extras › Create Collections…** for dives 101 and 102. | Collections in the "Dive Lens" set, photos in the right collection, values written. |
| 8 | After 7. | **Edit › Undo** once. | Note exactly what one Undo takes back. If a collection set was just created, it forms its own Undo step (separate write in the current code). |
| 9 | After 7 (redo it if needed). | Run **Create Collections…** again. | Nothing duplicated. |
| 10 | Clean. Settings: units in feet, altitude option on. Tag A1, A2, A3 with dive 101. | **Apply**. | Depths shown in ft, altitude correct. Set the settings back afterwards. |
| 11 | Windows only. | Play checks 2 and 8. | One Undo is enough. |

If the reference shows on Windows that one Undo does not take back both writes, stop and report it: it is not fixed by a refactoring.

## Recording the result

In the pull request, give:

- the system (macOS or Windows) and the Lightroom Classic version;
- the Dive Lens version;
- one line per check, 1 to 11: **OK**, or the difference described in words.

Describe differences in words only. No real dive data, no screenshot of a real catalog.
