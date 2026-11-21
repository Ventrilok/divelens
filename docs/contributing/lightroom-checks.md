# Lightroom checks

The automatic tests cannot prove that Lightroom accepts what Dive Lens writes to its catalog, or that one **Undo** takes it all back. These thirteen checks are played by hand in Lightroom Classic, on a test catalog, with fictional photos.

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
| Camera cards › clock offset of Test Camera A and Test Camera B | 0:00 | 0:00 |

The language, the log level and the automatic update check have no effect on the checks.

Dive Lens remembers each camera's last clock offset outside the catalog, so a new catalog does not reset it. If a camera card in **Tag Selection** or **Create Collections** shows another offset than 0:00 (left by an earlier run), set it to 0:00 before **Apply**.

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
| 1 | Base | Clean. Select A1, A2, A3. | **Library › Plug-in Extras › Tag Selection…**, choose dive 101, **Apply**. | Dive Lens fields: dive 101, values as above. Photo depth: A1 18.0 m, A2 15.0 m, A3 10.0 m. GPS filled. GPS altitude empty. |
| 2 | Base | After 1. | **Edit › Undo** once. | Dive Lens fields, location and GPS empty. |
| 3 | Base | Start from the state after check 1 (run it again first if you undid it). Select A1. | **Library › Plug-in Extras › Tag Selection…**, choose dive 102, **Apply**. | Dive Lens fields: dive 102, Glacier Wall. GPS -64.125, -60.5. Photo depth, photo temperature, elapsed time and clock offset empty. No refusal. |
| 4 | Base | After 3. Select A1. | **Tag Selection…**, choose dive 102 again, **Apply**. | No change. |
| 5 | Base | Select one photo, then none. | Select one photo and open **Tag Selection…** (the menu item is disabled with no photo selected). Then, in Lightroom, deselect everything (**Edit › Select None**). Note what the app shows. | Nothing is written. |
| 6 | Base | Select A1, open the app on that selection. | Change the selection in Lightroom (select B1), then click **Apply** in the app. | B1 never gets A1's values: either a refusal and nothing written, or the app switches to B1 and writes B1's values. |
| 7 | Base | Clean. Select A1, A2, A3, B1, B2. | **Library › Plug-in Extras › Create Collections…** for dives 101 and 102. | A "Dive Lens" set with one collection per dive (101, 102). Each photo in its dive's collection, Dive Lens fields filled. |
| 8 | Base | After 7. | **Edit › Undo** once. | Everything check 7 wrote is undone. A "Dive Lens" set created by check 7 needs a second Undo. |
| 9 | Base | After 7 (run check 7 again first if you undid it). | Run **Create Collections…** again. | Nothing duplicated. |
| 10 | Depth **Feet**, Copy depth to GPS altitude **On**, everything else as Base | Clean. Select A1, A2, A3. | **Tag Selection…**, dive 101, **Apply**. Afterwards, set Depth back to Metres and the altitude copy back to Off. | Photo depth: A1 about 59 ft. Max depth about 60 ft. GPS altitude about -18 m (Lightroom may show it in feet). |
| 11 | Base | Windows only. | Play checks 2 and 8. | One Undo takes everything back. |
| 12 | Base | Clean. Select B1, B2. | **Tag Selection…**, choose dive **101** (not the one marked "Covers the selection"). On the Test Camera B card, click **Suggest**, then **Apply**. | Suggest gives −4:30. B1 and B2: dive 101, Clock Offset -04:30. Photo depth: B1 18.0 m, B2 15.0 m. |
| 13 | Base | After 12. Select B1, B2. | Open **Tag Selection…**. Then open **Create Collections…**, choose dive 101 and go to the cameras step. Close both without applying. To finish, set Test Camera B back to 0:00: **Tag Selection…** on B1, B2, dive 102, Test Camera B at 0:00, **Apply**. | Tag Selection: dive 101 marked "Covers the selection", Test Camera B card at −4:30 with "remembered offset". Create Collections: Test Camera B card at −4:30 with "remembered offset". |

If the reference shows on Windows that one Undo does not take back both writes, stop and report it: it is not fixed by a refactoring.

## Recording the result

In the pull request, give:

- the system (macOS or Windows) and the Lightroom Classic version;
- the Dive Lens version;
- one line per check, 1 to 13: **OK**, or the difference described in words.

Describe differences in words only. No real dive data, no screenshot of a real catalog.
