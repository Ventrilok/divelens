# Lightroom checks

The automatic tests cannot prove that Lightroom accepts what Dive Lens writes to its catalog, or that one **Undo** takes it all back. These eleven checks are played by hand in Lightroom Classic, on a test catalog, with fictional photos.

Play them before a release that touches the plug-in or the catalog writes, and after a large refactoring. Compare the result with the previous run: a difference is a finding, even if it looks harmless.

## Set-up

1. In Lightroom Classic: **File › New Catalog…**, in a temporary folder. Never use your usual catalog.
2. **File › Import Photos and Video…**: choose the folder `docs/contributing/lightroom-checks/`, and import it with **Copy** into a folder next to the test catalog. Never use **Add**: Lightroom could write metadata into the files of the repository.
3. In Dive Lens settings (**Help › Plug-in Extras › Dive Lens Settings…**), set every setting to its value in the "Base" column of the table below.

### Settings

Each check says in its "Settings" column which settings it is played with ([the checks](#the-checks)). Every check uses the base settings except check 10, which changes two of them; set them back after it.

| Settings section › setting | Base (checks 1 to 9 and 11) | Check 10 |
|---|---|---|
| Dive log › Application | Subsurface | Subsurface |
| Dive log › log file | `core/fixtures/subsurface/fictional.xml` | the same |
| Display › Depth | Metres | **Feet** |
| Display › Temperature | °C | °C |
| Writing on photos › Copy depth to GPS altitude | **Off** | **On** |
| Collections › Place in | Catalog root | Catalog root |
| Collections › Group in the set | Dive Lens | Dive Lens |

The language, the log level and the automatic update check have no effect on the checks.

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
| B1 | 2025-06-14 14:10:00 | 102 | 12.0 m (first dive computer, Mares Genius; the second computer of dive 102 is not used for photo depth) |
| B2 | 2025-06-14 14:20:00 | 102 | (no value to check) |
| X1 | 2025-06-14 12:00:00 | none (between the two dives) | none |

The fictional dive log holds two dives that day. Dive 101: 09:30 to 10:12:30, site Atlantis / North Reef / Coral Garden, GPS 12.5, -45.25. Dive 102: 14:00 to 14:35, site Glacier Wall, GPS -64.125, -60.5.

Dive Lens fields are read in Lightroom's Metadata panel (Dive Lens section); the location fields in its IPTC / Location fields.

Expected for dive 101: `MaxDepth` 18.4 m, country "Atlantis", city "North Reef", location "Coral Garden", GPS 12°30'N 45°15'W.

## The checks

Each check starts from the state left by the previous one unless it says otherwise. Reset (remove the Dive Lens values, or use a fresh import) whenever a check says "clean".

| # | Settings | Start | Action | Expected |
|---|---|---|---|---|
| 1 | Base | Clean. Select A1, A2, A3. | **Library › Plug-in Extras › Tag Selection…**, choose dive 101, **Apply**. | Dive Lens fields filled. Dive values as above. Each photo has its own depth: A1 18.0 m, A2 15.0 m, A3 10.0 m. GPS written. No GPS altitude: Copy depth to GPS altitude is Off. |
| 2 | Base | After 1. | **Edit › Undo** once. | Everything is gone, GPS included. |
| 3 | Base | Start from the state after check 1 (run it again first if you undid it). Select A1. | **Library › Plug-in Extras › Tag Selection…**, choose dive 102, **Apply**. | The dive fields show dive 102 (number 102, Glacier Wall, GPS -64.125, -60.5 as Lightroom displays it). A1's own values from dive 101 (photo depth, photo temperature, elapsed time, clock offset) are empty. No photo depth is written, since A1 (09:40) is outside dive 102 (14:00-14:35). A refusal is not expected. |
| 4 | Base | After 3. Select A1. | **Tag Selection…**, choose dive 102 again, **Apply**. | Nothing else changes. |
| 5 | Base | Select one photo, then none. | Select one photo and open **Tag Selection…** (the menu item is disabled with no photo selected). Then, in Lightroom, deselect everything (**Edit › Select None**). Note what the app shows (Apply disabled with a message saying to select photos, or a refusal if you apply). | Nothing is written. |
| 6 | Base | Select A1, open the app on that selection. | Change the selection in Lightroom (select B1), then click **Apply** in the app. | Never the values of the old selection on other photos: either a refusal with nothing written, or the app describes the new selection again and writes on the new one. |
| 7 | Base | Clean. Select A1, A2, A3, B1, B2. | **Library › Plug-in Extras › Create Collections…** for dives 101 and 102. | Collections in the "Dive Lens" set, photos in the right collection, values written. |
| 8 | Base | After 7. | **Edit › Undo** once. | Note exactly what one Undo takes back. If a collection set was just created, it forms its own Undo step (separate write in the current code). |
| 9 | Base | After 7 (run check 7 again first if you undid it). | Run **Create Collections…** again. | Nothing duplicated. |
| 10 | Depth **Feet**, Copy depth to GPS altitude **On**, everything else as Base | Clean. Select A1, A2, A3. | **Tag Selection…**, dive 101, **Apply**. Afterwards, set Depth back to Metres and the altitude copy back to Off. | Depths shown in ft: A1 photo depth about 59 ft (18.0 m), max depth about 60 ft (18.4 m); GPS altitude about -18 m (Lightroom shows it in its own unit). The exact texts are those recorded in the reference run. |
| 11 | Base | Windows only. | Play checks 2 and 8. | One Undo is enough. |

If the reference shows on Windows that one Undo does not take back both writes, stop and report it: it is not fixed by a refactoring.

## Recording the result

In the pull request, give:

- the system (macOS or Windows) and the Lightroom Classic version;
- the Dive Lens version;
- one line per check, 1 to 11: **OK**, or the difference described in words.

Describe differences in words only. No real dive data, no screenshot of a real catalog.
