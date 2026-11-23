# Lightroom checks

The automatic tests cannot prove that Lightroom accepts what Dive Lens writes to its catalog, or that one **Undo** takes it all back, and they do not see the screens. These tests are played by hand in Lightroom Classic, on a test catalog, with fictional photos.

Play them before a release that touches the plug-in or the catalog writes, and after a large refactoring. Compare the result with the previous run: a difference is a finding, even if it looks harmless.

**How to read a test.** Each test has an id (TAG-01, COL-02…) and a title that says what it checks. **Before** gives the state to start from, **Steps** what to do, **Expected** what you should see. Fill in the [results sheet](#results-sheet) as you go.

## Contents

- [Before you start](#before-you-start)
  - [Prepare Lightroom: macOS](#prepare-lightroom-macos)
  - [Prepare Lightroom: Windows](#prepare-lightroom-windows)
  - [Test catalog and settings](#test-catalog-and-settings)
  - [Clean photos](#clean-photos)
  - [Test photos and dives](#test-photos-and-dives)
- [Tests by feature](#tests-by-feature)
  - [Tag Selection](#tag-selection): TAG-01 to TAG-08
  - [Create Collections](#create-collections): COL-01 to COL-04
  - [Camera clock offset](#camera-clock-offset): CLK-01 and CLK-02
  - [Units and altitude](#units-and-altitude): UNIT-01 and UNIT-02
  - [Settings screens](#settings-screens): SET-01 and SET-02
- [After the tests](#after-the-tests)
- [Results sheet](#results-sheet)

## Before you start

### Prepare Lightroom: macOS

Play on a development build. The installed plug-in and the one in the repository share the identifier `com.querton.divelens`, and we do not rely on what Lightroom does with two of them.

1. Quit Lightroom.
2. Move `~/Library/Application Support/Adobe/Lightroom/Modules/DiveLens.lrplugin` out of the Modules folder (for example onto the Desktop).
3. From `app/`, build the app: `npm run tauri build -- --bundles app`.
4. Open Lightroom: **File › Plug-in Manager › Add**, choose `plugin/DiveLens.lrdevplugin`.

Every test is played on macOS.

### Prepare Lightroom: Windows

The installed version is enough: the reference, or a trial installer from a manual Release workflow run on a branch other than `main`.

On Windows, **TAG-01, TAG-02, COL-01 and COL-02 are required**: they check that one Undo takes back both catalog writes. The other tests are welcome but optional.

If the reference shows on Windows that one Undo does not take back both writes, stop and report it: it is not fixed by a refactoring.

### Test catalog and settings

1. In Lightroom Classic: **File › New Catalog…**, in a temporary folder. Never use your usual catalog.
2. **File › Import Photos and Video…**: choose the folder `docs/contributing/lightroom-checks/`, and import it with **Copy** into a folder next to the test catalog. Never use **Add**: Lightroom could write metadata into the files of the repository.
3. Open Dive Lens settings (**Help › Plug-in Extras › Dive Lens Settings…**). Export your own settings first (**Export or import › Export…**): the tests replace them.
4. Set every setting to its base value:

| Settings section › setting | Base |
|---|---|
| Dive log › Application | Subsurface |
| Dive log › log file | `core/fixtures/subsurface/fictional.xml` |
| Display › Depth | Metres |
| Display › Temperature | °C |
| Writing on photos › Copy depth to GPS altitude | **Off** |
| Collections › Place in | Catalog root |
| Collections › Group in the set | Dive Lens |
| Camera cards › clock offset of Test Camera A and Test Camera B | 0:00 |

The language, the log level and the automatic update check have no effect on the tests.

Every test uses the base settings. A test that needs another setting changes it in its steps and sets it back in its last step.

Dive Lens remembers each camera's last clock offset outside the catalog, so a new catalog does not reset it. If a camera card in **Tag Selection** or **Create Collections** shows another offset than 0:00 (left by an earlier run), set it to 0:00 before **Apply**.

### Clean photos

Several tests start from clean photos: no Dive Lens values, no location, no GPS. They are cleared with a metadata preset.

Create the preset once:

1. In the Library module, Metadata panel: **Preset › Edit Presets…**.
2. Untick everything, then tick every field of the Dive Lens section, and the fields GPS, Altitude, Sublocation, City, State / Province, Country and ISO Country Code. Leave every ticked field empty.
3. **Preset › Save Current Settings as New Preset…**, name it "Clear Dive Lens", then **Done**.

To clean the photos: select all the test photos (**Edit › Select All**), then in the Metadata panel choose **Preset › Clear Dive Lens**.

### Test photos and dives

| Photo | Taken | Dive | Photo's own depth |
|---|---|---|---|
| A1 | 2025-06-14 09:40:00 | 101 | 18.0 m |
| A2 | 2025-06-14 09:50:00 | 101 | 15.0 m |
| A3 | 2025-06-14 10:00:00 | 101 | 10.0 m |
| B1 | 2025-06-14 14:10:00 | 102 | 12.0 m (first dive computer, Mares Genius; the second computer of dive 102 is not used for photo depth) |
| B2 | 2025-06-14 14:20:00 | 102 | (no value to check) |
| X1 | 2025-06-14 12:00:00 | none (between the two dives) | none |

The fictional dive log holds two dives that day:

| Dive | Time | Site | GPS |
|---|---|---|---|
| 101 | 09:30 to 10:12:30 | Atlantis / North Reef / Coral Garden | 12.5, -45.25 |
| 102 | 14:00 to 14:35 | Glacier Wall | -64.125, -60.5 |

Dive Lens fields are read in Lightroom's Metadata panel (Dive Lens section); the location fields in its IPTC / Location fields.

## Tests by feature

Within a feature, play the tests in order: some start from the state the previous one left.

### Tag Selection

#### TAG-01 · Tag three photos with one dive

**Before:** [clean photos](#clean-photos). Select A1, A2, A3.

**Steps**

1. **Library › Plug-in Extras › Tag Selection…**
2. Choose dive 101.
3. Click **Apply**.

**Expected**

- Dive Lens fields: dive 101, `MaxDepth` 18.4 m.
- Photo depth: A1 18.0 m, A2 15.0 m, A3 10.0 m.
- Location: country "Atlantis", city "North Reef", location "Coral Garden".
- GPS 12°30'N 45°15'W.
- GPS altitude empty.

#### TAG-02 · One Undo takes the tag back

**Before:** after TAG-01.

**Steps**

1. **Edit › Undo**, once.

**Expected**

- A1, A2, A3: Dive Lens fields, location and GPS empty.

#### TAG-03 · Tag a photo again with another dive

**Before:** A1, A2, A3 tagged with dive 101 (play TAG-01 again, since TAG-02 undid it). Select A1.

**Steps**

1. **Library › Plug-in Extras › Tag Selection…**
2. Choose dive 102.
3. Click **Apply**.

**Expected**

- No refusal.
- Dive Lens fields: dive 102, Glacier Wall.
- GPS -64.125, -60.5.
- Photo depth, photo temperature, elapsed time and clock offset empty.

#### TAG-04 · Tagging again with the same dive changes nothing

**Before:** after TAG-03. Select A1.

**Steps**

1. **Library › Plug-in Extras › Tag Selection…**
2. Choose dive 102 again.
3. Click **Apply**.

**Expected**

- No change.

#### TAG-05 · No photo selected: nothing is written

**Before:** any photos.

**Steps**

1. Select one photo.
2. **Library › Plug-in Extras › Tag Selection…** (the menu item is disabled when no photo is selected).
3. In Lightroom, deselect everything: **Edit › Select None**.
4. Note what the app shows.

**Expected**

- Nothing is written.

#### TAG-06 · The selection changes before Apply

**Before:** any photos. Select A1.

**Steps**

1. **Library › Plug-in Extras › Tag Selection…**
2. In Lightroom, select B1 instead.
3. Click **Apply** in the app.

**Expected**

- B1 never gets A1's values: either a refusal and nothing written, or the app switches to B1 and writes B1's values.

#### TAG-07 · The dive that covers the selection is marked

This test writes nothing.

**Before:** any photos. Select A1, A2, A3.

**Steps**

1. **Library › Plug-in Extras › Tag Selection…**, without choosing a dive.
2. With the app still open, select X1 alone in Lightroom.
3. Then select A1 and B1 together.

**Expected**

- After step 1: dive 101 marked "Covers the selection" and shown in detail.
- After step 2: no dive marked.
- After step 3: no dive marked.

#### TAG-08 · Search and rating filters

This test writes nothing.

**Before:** any selection.

**Steps**

1. **Library › Plug-in Extras › Tag Selection…**
2. Type "Glacier" in the search field.
3. Click **Clear filters**.
4. Set **Rating** to "5★ and more".
5. Click **Clear filters**.

**Expected**

- After step 2: dive 102 only.
- After step 4: dive 107 only.
- After steps 3 and 5: every dive back.

### Create Collections

#### COL-01 · Create collections for two dives

**Before:** [clean photos](#clean-photos). Select A1, A2, A3, B1, B2.

**Steps**

1. **Library › Plug-in Extras › Create Collections…**
2. Tick dives 101 and 102, then **Next**.
3. Click **Apply**.

**Expected**

- A "Dive Lens" set with one collection per dive (101, 102).
- Each photo in its dive's collection, Dive Lens fields filled.

#### COL-02 · One Undo takes the collections back

**Before:** after COL-01.

**Steps**

1. **Edit › Undo**, once.

**Expected**

- Everything COL-01 wrote is undone.
- A "Dive Lens" set created by COL-01 needs a second Undo.

#### COL-03 · Running again duplicates nothing

**Before:** the state after COL-01 (play COL-01 again, since COL-02 undid it).

**Steps**

1. Run **Create Collections…** again, as in COL-01.

**Expected**

- Nothing duplicated.

#### COL-04 · Country and period filters

This test writes nothing.

**Before:** any selection.

**Steps**

1. **Library › Plug-in Extras › Create Collections…**, and stay on the first step.
2. Set **Country** to Lemuria.
3. Click **Clear filters**.
4. Set **Period** from 2025-06-17 to 2025-06-17.
5. Click **Clear filters**.

**Expected**

- After step 2: dive 104 only.
- After step 4: dives 106 and 107 only.
- After steps 3 and 5: every dive back.

### Camera clock offset

Play these two tests one after the other: CLK-02 ends by setting Test Camera B back to 0:00, which the other tests need.

#### CLK-01 · Suggest a clock offset

**Before:** [clean photos](#clean-photos). Select B1, B2.

**Steps**

1. **Library › Plug-in Extras › Tag Selection…**
2. Choose dive **101** (not the one marked "Covers the selection").
3. On the Test Camera B card, click **Suggest**.
4. Click **Apply**.

**Expected**

- Suggest gives −4:30.
- B1 and B2: dive 101, Clock Offset -04:30.
- Photo depth: B1 18.0 m, B2 15.0 m.

#### CLK-02 · The offset is remembered

**Before:** after CLK-01. Select B1, B2.

**Steps**

1. **Library › Plug-in Extras › Tag Selection…**, look, and close without applying.
2. **Library › Plug-in Extras › Create Collections…**, tick dive 101, then **Next** to reach the camera cards. Look, and close without applying.
3. Reset: **Tag Selection…** on B1, B2, choose dive 102, set Test Camera B to 0:00, **Apply**.

**Expected**

- After step 1: dive 101 marked "Covers the selection"; Test Camera B card at −4:30 with "remembered offset".
- After step 2: Test Camera B card at −4:30 with "remembered offset".

### Units and altitude

#### UNIT-01 · Writing in feet, with the depth copied to the altitude

**Before:** [clean photos](#clean-photos). Select A1, A2, A3.

**Steps**

1. In Dive Lens settings, set **Display › Depth** to Feet and **Writing on photos › Copy depth to GPS altitude** to On.
2. **Library › Plug-in Extras › Tag Selection…**, choose dive 101, **Apply**.
3. Reset: set Depth back to Metres and the altitude copy back to Off.

**Expected**

- Photo depth: A1 about 59 ft.
- Max depth about 60 ft.
- GPS altitude about -18 m (Lightroom may show it in feet).

#### UNIT-02 · Depths in the dive list, in feet and in metres

This test writes nothing.

**Before:** any selection.

**Steps**

1. In Dive Lens settings, set **Display › Depth** to Feet.
2. Open **Tag Selection…** and read the depths in the dive list.
3. Set Depth back to Metres and read them again.

**Expected**

- After step 2: dive 101 60 ft, dive 108 102 ft.
- After step 3: dive 101 18.4 m, dive 108 31.2 m.

### Settings screens

#### SET-01 · Dive log not found

This test writes nothing.

**Before:** copy `core/fixtures/subsurface/fictional.xml` into the test folder, next to the test catalog.

**Steps**

1. In Dive Lens settings, **Dive log**: **Choose a file…** and choose that copy.
2. Quit Dive Lens.
3. Rename the copy.
4. Open **Tag Selection…**.
5. On the screen shown, **Choose a file…** and choose `core/fixtures/subsurface/fictional.xml` of the repository.

**Expected**

- After step 4: a screen "Subsurface dive log not found", with the copy's former path, the buttons **Choose a file…** and **Choose a folder…**, and no **Try again**.
- After step 5: the dive list.

#### SET-02 · Check for updates

This test writes nothing.

**Before:** connected to the internet.

**Steps**

1. In Dive Lens settings, **Updates**: click **Check now**.

**Expected**

- When the version played is the latest one published: "Up to date · last checked today at" the current time.
- Otherwise: "Version … available", with the latest version.

A development build never looks for updates by itself (only an installed copy does, at most once a day): on a development build, no update notice on opening proves nothing. SET-02 asks GitHub on any build.

## After the tests

1. In Dive Lens settings, import your own settings back (**Export or import › Import…**).
2. macOS development build: remove the repository plug-in in the Plug-in Manager, quit Lightroom, and put `DiveLens.lrplugin` back in Modules.

## Results sheet

Copy this sheet into the pull request and fill it in. Write **OK**, **not played**, or the difference described in words. On Windows, TAG-01, TAG-02, COL-01 and COL-02 are required.

Describe differences in words only. No real dive data, no screenshot of a real catalog.

```markdown
Lightroom checks — Dive Lens <version>
macOS <version>, Lightroom Classic <version>
Windows <version>, Lightroom Classic <version>

| Test | macOS | Windows |
|---|---|---|
| TAG-01 Tag three photos with one dive | | |
| TAG-02 One Undo takes the tag back | | |
| TAG-03 Tag a photo again with another dive | | |
| TAG-04 Tagging again with the same dive changes nothing | | |
| TAG-05 No photo selected: nothing is written | | |
| TAG-06 The selection changes before Apply | | |
| TAG-07 The dive that covers the selection is marked | | |
| TAG-08 Search and rating filters | | |
| COL-01 Create collections for two dives | | |
| COL-02 One Undo takes the collections back | | |
| COL-03 Running again duplicates nothing | | |
| COL-04 Country and period filters | | |
| CLK-01 Suggest a clock offset | | |
| CLK-02 The offset is remembered | | |
| UNIT-01 Writing in feet, with the depth copied to the altitude | | |
| UNIT-02 Depths in the dive list, in feet and in metres | | |
| SET-01 Dive log not found | | |
| SET-02 Check for updates | | |
```
