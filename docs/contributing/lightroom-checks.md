# Lightroom checks

The automatic tests cannot prove that Lightroom accepts what Dive Lens writes to its catalog, or that one **Undo** takes it all back, and they do not see the screens. These tests are played by hand in Lightroom Classic, on a test catalog, with fictional photos.

Play them before a release that touches the plug-in or the catalog writes, and after a large refactoring. Compare the result with the previous run: a difference is a finding, even if it looks harmless.

**How to read a test.** Each test has an id (JRN-01, CAT-02…) and a title that says what it checks. **Before** gives the state to start from, **Steps** what to do, **Expected** what you should see. Fill in the [results sheet](#results-sheet) as you go. The tests of earlier versions, which named the two former screens, are listed with their new ids under [Old identifiers](#old-identifiers).

## Contents

- [Before you start](#before-you-start)
  - [Prepare Lightroom: macOS](#prepare-lightroom-macos)
  - [Prepare Lightroom: Windows](#prepare-lightroom-windows)
  - [Test catalog and settings](#test-catalog-and-settings)
  - [Clean photos](#clean-photos)
  - [Clean start](#clean-start)
  - [Test photos and dives](#test-photos-and-dives)
  - [Shortcuts used in the steps](#shortcuts-used-in-the-steps)
- [Tests by feature](#tests-by-feature)
  - [Journey and frame](#journey-and-frame): JRN-01 to JRN-12
  - [Catalog path](#catalog-path): CAT-01 to CAT-12
  - [Selection path](#selection-path): SEL-01 to SEL-09
  - [Camera clock offset](#camera-clock-offset): CLK-01 and CLK-02
  - [Units and altitude](#units-and-altitude): UNIT-01 to UNIT-04
  - [Settings](#settings): SET-01 to SET-06
  - [Undo](#undo): UNDO-01 to UNDO-03
- [After the tests](#after-the-tests)
- [Results sheet](#results-sheet)
- [Old identifiers](#old-identifiers)

## Before you start

### Prepare Lightroom: macOS

Play on a development build. The installed plug-in and the one in the repository share the identifier `com.querton.divelens`, and we do not rely on what Lightroom does with two of them.

1. Quit Lightroom.
2. Move `~/Library/Application Support/Adobe/Lightroom/Modules/DiveLens.lrplugin` out of the Modules folder (for example onto the Desktop).
3. From `app/`, build the app: `npm run tauri build -- --bundles app`.
4. Open Lightroom: **File › Plug-in Manager › Add**, choose `plugin/DiveLens.lrdevplugin`.

Every test is played on macOS. JRN-10 alone needs installed copies instead of the development build: its **Before** says how.

### Prepare Lightroom: Windows

The installed version is enough: the reference, or a trial installer from a manual Release workflow run on a branch other than `main`.

On Windows, **UNDO-01, UNDO-02, UNDO-03 and CAT-08 are required**: they check that one Undo takes back both catalog writes, GPS and altitude included. The other tests are welcome but optional.

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
| Collections › Place in | Catalog root |
| Collections › Group in the set | Dive Lens |
| Clocks step › clock offset of Test Camera A and Test Camera B | 0:00 |

The language, the log level and the automatic update check have no effect on the tests, except where a test changes the language itself.

Copying the depth to the GPS altitude is no longer a setting: it is the option **Copy depth to GPS altitude** of the Review & Apply step, unticked on every run. The tests that need it tick it.

Every test uses the base settings. A test that needs another setting changes it in its steps and sets it back in its last step.

Dive Lens remembers each camera's last clock offset outside the catalog, so a new catalog does not reset it. If a camera card at **Clocks** shows another offset than 0:00 (left by an earlier run), set it to 0:00 before **Apply**, or forget it beforehand in **Settings › Remembered clock offsets** (**Forget**).

### Clean photos

Several tests start from clean photos: no Dive Lens values, no location, no GPS. They are cleared with a metadata preset.

Create the preset once:

1. In the Library module, Metadata panel: **Preset › Edit Presets…**.
2. Untick everything, then tick every field of the Dive Lens section, and the fields GPS, Altitude, Sublocation, City, State / Province, Country and ISO Country Code. Leave every ticked field empty.
3. **Preset › Save Current Settings as New Preset…**, name it "Clear Dive Lens", then **Done**.

To clean the photos: select all the test photos (**Edit › Select All**), then in the Metadata panel choose **Preset › Clear Dive Lens**.

### Clean start

A clean start is [clean photos](#clean-photos) and no collection written by Dive Lens: in the Collections panel, delete the "Dive Lens" collection set if there is one (right-click › **Delete**). The preset clears fields, not collections.

### Test photos and dives

| Photo | Camera | Taken | Dive | Photo's own depth |
|---|---|---|---|---|
| A1 | Test Camera A | 2025-06-14 09:40:00 | 101 | 18.0 m |
| A2 | Test Camera A | 2025-06-14 09:50:00 | 101 | 15.0 m |
| A3 | Test Camera A | 2025-06-14 10:00:00 | 101 | 10.0 m |
| B1 | Test Camera B | 2025-06-14 14:10:00 | 102 | 12.0 m (first dive computer, Mares Genius; the second computer of dive 102 is not used for photo depth) |
| B2 | Test Camera B | 2025-06-14 14:20:00 | 102 | (no value to check) |
| X1 | Test Camera A | 2025-06-14 12:00:00 | none (between the two dives) | none |

The fictional dive log holds ten dives. Two of them are on the day of the photos:

| Dive | Time | Site | GPS |
|---|---|---|---|
| 101 | 09:30 to 10:12:30 | Atlantis / North Reef / Coral Garden | 12.5, -45.25 |
| 102 | 14:00 to 14:35 | Glacier Wall (Atlantis, Frostville) | -64.125, -60.5 |

X1 is taken 108 minutes after the end of dive 101 (rounded up to the minute) and 120 minutes before the start of dive 102. The other dives used here: 103 (Shark Point / Bay, on 2025-06-15, no profile, no photo), 104 (Lemuria), 106 and 107 (both on 2025-06-17; 107 is rated 5★), 108 (31.2 m). The site used as another site is Récif d'Émeraude, country Atlantide, location Île Verte, GPS -20.5, 57.25.

Dive Lens fields are read in Lightroom's Metadata panel (Dive Lens section); the location fields in its IPTC / Location fields.

### Shortcuts used in the steps

- **Catalog path to Clocks**: **Library › Plug-in Extras › Open Dive Lens…**; at **Scope** choose **Entire catalog**, then **Next**; at **Dives** tick dives 101 and 102, then **Next**.
- **Catalog path to Review & Apply**: the same, then **Next** at **Clocks**.
- **Selection path with dive N**: with the photos selected in Lightroom, **Library › Plug-in Extras › Open Dive Lens…**; at **Scope** choose the **Lightroom selection** card, then **Next**; at **Dives** click dive N, then **Next**; at **Clocks**, **Next**. You are at **Review & Apply**.

## Tests by feature

Within a feature, play the tests in order: some start from the state the previous one left.

### Journey and frame

Nobody has looked at the real window of the journey yet: JRN-01 and JRN-02 come first. They walk once through the five steps of each path, only to see that every screen appears and reads right, in English and in French. The other tests check what is written.

#### JRN-01 · Every screen of the catalog path, in English and in French

**Before:** [clean start](#clean-start). Nothing selected in Lightroom (**Edit › Select None**).

**Steps**

1. **Library › Plug-in Extras › Open Dive Lens…**. Read the Scope step.
2. Choose **Entire catalog**, then **Next**. Read the Dives step, tick dives 101 and 102, then **Next**.
3. Read the Clocks step, then **Next**.
4. Read the Review & Apply step, then click **Apply to 5 photos**.
5. Read the summary, then click **Back to scope selection**. Then **Next**, and **Back**.
6. Click the gear, set **Display › Language** to Français, then **Retour**.
7. At « Périmètre », play steps 2 to 5 again, in French.
8. Reset: set the language back to its base value, then make a [clean start](#clean-start).

**Expected**

- On every screen, in both languages: nothing cut off or overlapping, no raw text key (such as `journey.opt.dive`), no English left on the French screens.
- Everywhere: at the top, "Dive Lens" and the gear (tooltip "Settings" / « Réglages »); the strip "Scope › Dives › Clocks › Review & Apply › Summary" / « Périmètre › Plongées › Horloges › Vérifier et appliquer › Récapitulatif », with the current step highlighted; at the bottom, the status bar: "Lightroom connected", "Catalog: <name>", "Catalog size: 6 items", "Selected photos: none", "Dive log app: Subsurface", "Dives logged: 10" / « Lightroom connecté », « Catalogue : … », « Taille du catalogue : 6 éléments », « Photos sélectionnées : aucune », « Application de carnet : Subsurface », « Plongées enregistrées : 10 ».
- At Scope: the title "Which photos to work on?" / « Sur quelles photos travailler ? »; two cards separated by "or" / « ou »: "Entire catalog" / « Tout le catalogue », and, greyed, "Lightroom selection" / « Les photos sélectionnées dans Lightroom » with "No photos selected" / « Aucune photo sélectionnée ». Once **Entire catalog** is chosen, the bar reads "Entire catalog" and "Next step: tick the dives" / « Tout le catalogue », « Étape suivante : cocher les plongées », next to "Next" / « Suivant ».
- At Dives: the line "Photos: entire catalog · tick the dives, Dive Lens finds the photos taken during each one" / « Photos : tout le catalogue · cochez les plongées, Dive Lens retrouve les photos prises pendant chacune », with no "Change" / « Modifier » link after it; the dive list grouped by month, with its search, filters, sort and detail card. Before any tick, "Next" is greyed with "Check one or more dives" / « Cochez une ou plusieurs plongées ». With 101 and 102 ticked: "2 dives selected" and "From 14 Jun 2025 to 14 Jun 2025 · photos looked for in the entire catalog" / « 2 plongées sélectionnées », « Du 14 juin 2025 au 14 juin 2025 · photos cherchées dans tout le catalogue », then "Deselect all", "Back", "Next" / « Tout désélectionner », « Précédent », « Suivant ».
- At Clocks: the line "Photos: entire catalog, from 14 Jun 2025 to 14 Jun 2025 · 6 photos from 2 cameras · 2 dives ticked" / « Photos : tout le catalogue, du 14 juin 2025 au 14 juin 2025 · 6 photos de 2 appareils · 2 plongées cochées ».
  - "Camera clocks" / « Horloge des appareils », with "Suggest for all" / « Suggérer pour tous ».
  - Two cards: Test Camera A ("3 in the dives · 1 outside" / « 3 dans les plongées · 1 hors ») and Test Camera B ("2 in the dives · 0 outside"), each with "Suggest" / « Suggérer » and "Ignore" / « Ignorer ».
  - "Include photos taken at the surface" / « Inclure les photos prises en surface », at 0 min.
  - "Preview by dive" / « Aperçu par plongée »: 101 with 3 photos and 102 with 2, both "Depth · temp. · time" / « Profondeur · temp. · temps ».
  - "Outside the dives · 1 photo" / « Hors plongée · 1 photo », with "1 Test Camera A".
  - The bar: "5 photos in 2 dives · 1 outside the dives" / « 5 photos dans 2 plongées · 1 hors plongée ».
- At Review & Apply: the title "What Dive Lens writes in Lightroom" / « Ce que Dive Lens écrit dans Lightroom ».
  - Under "Metadata" / « Métadonnées »: "Dive details" ticked, "5 photos · 5 with depth" / « Détails de la plongée », « 5 photos · 5 avec profondeur »; under it "Copy depth to GPS altitude" unticked, "No" / « Profondeur dans l'altitude GPS », « Non »; "Site details" ticked, "5 photos · location and GPS" / « Détails du site », « 5 photos · lieu et GPS », with "Each dive's site" chosen or "Other site, the same for all photos…" / « Site de chaque plongée », « Autre site, le même pour toutes les photos… », then "Location" and "GPS" / « Lieu », « GPS ».
  - Under "Collections": "Create collections" ticked, its text ending "for example Dive Lens › 101 - Coral Garden.", "2 collections", and the link "Place in Settings" / « Créer les collections », « par exemple Dive Lens › 101 - Coral Garden. », « 2 collections », « Emplacement dans Réglages ».
  - "Good to know before applying" / « À savoir avant d'appliquer »: "1 photo outside the dives gets nothing: on land, on the boat…" and "Units: m, °C" / « 1 photo hors plongée ne reçoit rien : à terre, sur le bateau… », « Unités : m, °C ».
  - The bar: "5 photos · 2 collections", "Undone in one step in Lightroom, GPS and altitude included", "Back", "Apply to 5 photos" / « 5 photos · 2 collections », « Annulable en une fois dans Lightroom, GPS et altitude compris », « Précédent », « Appliquer à 5 photos ».
- On the summary: "From 14 Jun 2025 to 14 Jun 2025 · 2 cameras · dive details, site details, collections" / « Du 14 juin 2025 au 14 juin 2025 · 2 appareils · détails de la plongée, détails du site, collections ».
  - "Written in Lightroom" / « Écrit dans Lightroom », with "Photos tagged" 5, "With depth" 5 and "Collections" 2 / « Photos taguées », « Avec profondeur », « Collections ».
  - Under "Dive Lens": "101 - Coral Garden", "3 photos · created" and "102 - Glacier Wall", "2 photos · created". The French run reads « complétée » instead.
  - "1 photo outside the dives got nothing." / « 1 photo hors plongée n'a rien reçu. ».
  - "To undo: Edit › Undo in Lightroom, in one step, GPS and altitude included." / « Pour tout défaire : Édition › Annuler dans Lightroom, en une seule fois, GPS et altitude compris. ».
  - "Back to scope selection" / « Retour au choix du périmètre ». A click on a step of the strip does nothing here.
- After "Back to scope selection" (step 5): Scope, with **Entire catalog** still chosen. **Next** leads to Dives with no dive ticked.

#### JRN-02 · Every screen of the selection path, in English and in French

**Before:** [clean start](#clean-start). Select A1, A2, A3 and X1.

**Steps**

1. **Library › Plug-in Extras › Open Dive Lens…**. Read the Scope step.
2. Choose the **Lightroom selection** card, then **Next**. Read the Dives step, click dive 101, then **Next**.
3. Read the Clocks step, then **Next**.
4. Read the Review & Apply step, then click **Apply to 4 photos**.
5. Read the summary, then click **Done**.
6. Click the gear, set **Display › Language** to Français, then **Retour**.
7. Play steps 1 to 5 again, in French.
8. Reset: set the language back to its base value, then make a [clean start](#clean-start).

**Expected**

- As in JRN-01: nothing cut off or overlapping, no raw text key, no English on the French screens; the status bar reads "Selected photos: 4" / « Photos sélectionnées : 4 ».
- At Scope: the card "Lightroom selection" / « Les 4 photos sélectionnées dans Lightroom » is lit, with "Test Camera A · Saturday, 14 June 2025" / « Test Camera A · samedi 14 juin 2025 », "You choose one dive: all 4 photos are attached to it, whatever their time." / « Vous choisissez une plongée : les 4 photos y sont toutes rattachées, quelle que soit leur heure. » and four thumbnails. The "Entire catalog" card adds "The Lightroom selection is ignored." / « La sélection Lightroom est ignorée. ». Once the card is chosen, the bar reads "The 4 selected photos" and "Next step: choose their dive" / « Les 4 photos sélectionnées », « Étape suivante : choisir leur plongée ».
- At Dives:
  - The line "Photos: the 4 selected in Lightroom · Test Camera A · Saturday, 14 June 2025" / « Photos : les 4 sélectionnées dans Lightroom · Test Camera A · samedi 14 juin 2025 », with no "Change" / « Modifier » link after it.
  - "Choose their dive: all 4 photos will be attached to it, whatever their time. Without a dive, you will only set a location, chosen at step 4." / « Choisissez leur plongée : les 4 photos y seront toutes rattachées, quelle que soit leur heure. Sans plongée, vous poserez seulement un lieu, choisi à l'étape 4. ».
  - The counter "10 dives · only one to choose" / « 10 plongées · une seule à choisir ».
  - The first row "No dive — location only", "chosen at step 4: boat, beach…" / « Aucune plongée — lieu seulement », « choisi à l'étape 4 : bateau, plage… ».
  - Before a choice, "Next" is greyed with "Choose a dive" / « Choisissez une plongée ». With 101 chosen, the bar reads "101 Coral Garden" and "The 4 selected photos will be attached to it" / « Les 4 photos sélectionnées y seront rattachées ».
- At Clocks:
  - The line "Photos: the 4 selected in Lightroom · 1 camera · dive 101 Coral Garden" / « Photos : les 4 sélectionnées dans Lightroom · 1 appareil · plongée 101 Coral Garden ».
  - The note "The 4 photos are attached to 101 Coral Garden, whatever their time. The clock only gives them a depth: only those taken during the dive get one." / « Les 4 photos sont rattachées à 101 Coral Garden, quelle que soit leur heure. L'horloge sert seulement à leur donner une profondeur : seules celles prises pendant la plongée en reçoivent une. ».
  - "Camera clocks" with one card, Test Camera A, "3 in the dive · 1 outside" / « 3 dans la plongée · 1 hors », "Suggest" and "Suggest for all"; no "Ignore", no "Include photos taken at the surface".
  - "Preview by dive": 101 with 4 photos.
  - "Outside the dive's times · 1 photo" / « Hors de l'horaire de la plongée · 1 photo »: "Taken at 12:00, 108 minutes after getting out of the water. Attached to the dive anyway: it gets its data and its location, without depth." / « Prise à 12:00, 108 minutes après la sortie de l'eau. Rattachée quand même à la plongée : elle reçoit ses infos et son lieu, sans profondeur. ».
  - The bar: "4 photos attached to 101 Coral Garden · 3 with depth" and "Nothing else to set." / « 4 photos rattachées à 101 Coral Garden · 3 avec profondeur », « Rien d'autre à régler. ».
- At Review & Apply: the same title, groups and options as in JRN-01, with these differences.
  - "Dive details": "4 photos · 3 with depth".
  - The site: "Dive's site: Coral Garden" chosen, or "Other site…" / « Site de la plongée : Coral Garden », « Autre site… ». "Site details": "4 photos · location and GPS".
  - "Create collections": "The dive's "number - site" collection: Dive Lens › 101 - Coral Garden." / « La collection « numéro - site » de la plongée : Dive Lens › 101 - Coral Garden. », and "1 collection".
  - "Good to know before applying": "1 photo taken after getting out of the water (12:00): dive data and location, without depth." / « 1 photo prise après la sortie de l'eau (12:00) : infos de la plongée et lieu, sans profondeur. », and "Units: m, °C".
  - The bar: "4 photos · 1 collection" and "Apply to 4 photos".
- On the summary:
  - "The 4 selected photos · dive 101 Coral Garden · dive details, site details, collection" / « Les 4 photos sélectionnées · plongée 101 Coral Garden · détails de la plongée, détails du site, collection ».
  - "Photos tagged" 4, "With depth" 3, "Collection" 1.
  - Under "Dive Lens": "101 - Coral Garden", "4 photos · created" (the French run: « complétée »).
  - "1 photo without depth: taken after getting out of the water." / « 1 photo sans profondeur : prise après la sortie de l'eau. ».
  - The Undo reminder, then "Done" / « Terminer », which leads to Scope, with the selection card still chosen.

#### JRN-03 · One menu item, enabled without a selection

This test writes nothing.

**Before:** Dive Lens app closed.

**Steps**

1. In Lightroom, **Edit › Select None**, then open **Library › Plug-in Extras**.
2. Choose **Open Dive Lens…**.
3. In the app, go to **Dives** (**Entire catalog**, **Next**). Then, in Lightroom, choose **Library › Plug-in Extras › Open Dive Lens…** again.
4. Choose **Help › Plug-in Extras › Dive Lens Settings…**.
5. **File › Plug-in Manager…**, choose Dive Lens: click **Open Dive Lens**, then **Open Settings**.

**Expected**

- After step 1: one Dive Lens item, **Open Dive Lens…**, enabled although nothing is selected. No **Create Collections…** or **Tag Selection…**.
- After step 2: the app opens at Scope; the selection card is greyed.
- After step 3: the app's window comes to the front at Scope, with **Entire catalog** still chosen.
- After step 4: the app shows Settings.
- After step 5: **Open Dive Lens** shows Scope; **Open Settings** shows Settings.
- In a French Lightroom, the two items read « Ouvrir Dive Lens… » and « Réglages de Dive Lens… » (optional).

#### JRN-04 · The Scope step follows the Lightroom selection live

This test writes nothing.

**Before:** any photos.

**Steps**

1. In Lightroom, **Edit › Select None**. **Library › Plug-in Extras › Open Dive Lens…**.
2. In Lightroom, select A1, A2, A3.
3. Select all six test photos.
4. In the app, choose the **Lightroom selection** card. Then, in Lightroom, **Edit › Select None**.

**Expected**

- After step 1: the selection card is greyed, with "Lightroom selection", "No photos selected" and "Select photos there to attach them all to a dive of your choice." The **Entire catalog** card does not say "The Lightroom selection is ignored." The status bar reads "Selected photos: none".
- After step 2: the selection card lights up: "Lightroom selection", "Test Camera A · Saturday, 14 June 2025", "You choose one dive: all 3 photos are attached to it, whatever their time.", three thumbnails. The **Entire catalog** card adds "The Lightroom selection is ignored." The status bar reads "Selected photos: 3".
- After step 3: "Test Camera A, Test Camera B · Saturday, 14 June 2025" and six thumbnails.
- After step 4: the card is greyed again; **Next** is greyed, and the bar shows "Select photos in Lightroom".

#### JRN-05 · The catalog size matches All Photographs

**Before:** the six test photos in the catalog.

**Steps**

1. **Library › Plug-in Extras › Open Dive Lens…**. Compare the status bar with the count Lightroom shows next to **Catalog › All Photographs**.
2. Choose **Entire catalog**, then **Next**.
3. In Lightroom, select X1 and choose **Photo › Remove Photo…**, then **Remove** (from the catalog only, not from the disk).
4. In the app, click **Back** to reach Scope.
5. Put X1 back: **Edit › Undo** in Lightroom; if Lightroom cannot undo it, import X1 again from the copied folder with **Add**.
6. In the app, **Next**, then **Back**.

**Expected**

- After step 1: "Catalog size: 6 items", the same number as All Photographs.
- After step 4: "Catalog size: 5 items", the same number as All Photographs.
- After step 6: "Catalog size: 6 items" again.

#### JRN-06 · "Open Dive Lens…" during the journey goes back to Scope

**Before:** [clean start](#clean-start).

**Steps**

1. [Catalog path to Clocks](#shortcuts-used-in-the-steps).
2. **Library › Plug-in Extras › Open Dive Lens…**.
3. In the app, **Next**.
4. **Next**, **Next**, then **Apply to 5 photos**.
5. On the summary, **Library › Plug-in Extras › Open Dive Lens…**, then **Next** in the app.
6. In Lightroom, select A1. [Selection path with dive 101](#shortcuts-used-in-the-steps), then **Apply to 1 photo**.
7. On the summary, **Library › Plug-in Extras › Open Dive Lens…**, then **Next** in the app.
8. Reset: [clean start](#clean-start).

**Expected**

- After step 2: Scope, **Entire catalog** still chosen.
- After step 3: Dives, with 101 and 102 still ticked.
- After step 5: Dives, with no dive ticked.
- After step 7: Dives, with no dive chosen.

#### JRN-07 · Settings, then Back, keep the step and the choices

This test writes nothing.

**Before:** any photos.

**Steps**

1. [Catalog path to Clocks](#shortcuts-used-in-the-steps). On the Test Camera A card, type 0:05 in the offset field.
2. Click the gear.
3. Click **Back**.
4. **Next**. At **Review & Apply**, untick **Site details**. Click the link **Place in Settings**, then **Back**.
5. In Lightroom, **Help › Plug-in Extras › Dive Lens Settings…**. Set **Display › Depth** to Feet and **Language** to Français, then click **Retour**.
6. Reset: click the gear, set **Depth** back to Metres and the language back to its base value, then **Back**. Go **Back** to **Clocks**, set Test Camera A to 0:00, and leave without applying.

**Expected**

- After step 2: Settings fill the window, with a **Back** button; the status bar stays.
- After step 3: Clocks, dives 101 and 102 ticked, Test Camera A at 0:05.
- After step 4: Review & Apply again, **Site details** still unticked.
- After step 5: « Vérifier et appliquer », in French, **Détails du site** still unticked, and « Unités : ft, °C ».

#### JRN-08 · Another dive log in Settings starts the journey again

This test writes nothing.

**Before:** copy `core/fixtures/subsurface/fictional.xml` into the test folder, next to the test catalog.

**Steps**

1. [Catalog path to Clocks](#shortcuts-used-in-the-steps).
2. Click the gear. **Dive log**: **Choose a file…** and choose that copy.
3. In Lightroom, click another photo.
4. In the app, click **Back**.
5. Reset: in Settings, **Choose a file…** and choose `core/fixtures/subsurface/fictional.xml` of the repository, then **Back**.

**Expected**

- After step 4: Scope, no card chosen. Going on, no dive is ticked at Dives. No orange notice. The status bar reads "Dives logged: 10".

#### JRN-09 · Nothing leaves Review & Apply during a write

The six test photos are written too fast to click during the write: this test uses a second, throwaway catalog with many photos.

**Before**

1. **File › New Catalog…**, in another temporary folder. Import the test folder with **Copy**.
2. Select all the photos and choose **Photo › Create Virtual Copy** (it may read **Create Virtual Copies** with several photos selected). Do it seven times in all, each time with every photo selected: 768 photos.

**Steps**

1. Select all the photos. [Selection path with dive 101](#shortcuts-used-in-the-steps), then click **Apply to 768 photos**.
2. While the app shows "Writing in Lightroom…" and its progress bar:
   1. choose **Library › Plug-in Extras › Open Dive Lens…**;
   2. choose **Help › Plug-in Extras › Dive Lens Settings…**;
   3. click the gear;
   4. click a step of the strip.
3. Go back to the test catalog: **File › Open Recent**.

**Expected**

- During the write: nothing leaves Review & Apply, and the gear is greyed.
- At the end: the summary of the write.
- If the update notice shows (installed copies only), its **See** does nothing during the write either.
- If the write ends before you can click, note **not played**.

#### JRN-10 · A plug-in of another version asks to restart Lightroom

**Before:** this test uses installed copies, not the development build.

1. Quit the Dive Lens app if it is open. Remove the development plug-in in the Plug-in Manager.
2. Install Dive Lens 2.1.1 with its installer, then restart Lightroom.
3. Have a trial installer of the version under test ready (manual Release workflow run, see [Prepare Lightroom: Windows](#prepare-lightroom-windows)).

**Steps**

1. Leave Lightroom open. Install the version under test with the trial installer.
2. Open the new app from the plug-in folder: `~/Library/Application Support/Adobe/Lightroom/Modules/DiveLens.lrplugin`, right-click › **Show Package Contents**, then open **Dive Lens.app**.
3. In Lightroom, open **Library › Plug-in Extras** and choose **Create Collections…**.
4. Quit Lightroom and start it again.
5. Reset:
   1. quit Lightroom and move the trial copy, `DiveLens.lrplugin` in Modules, to the Trash;
   2. start Lightroom and add `plugin/DiveLens.lrdevplugin` again (**File › Plug-in Manager › Add**).

   Leave the copy you set aside in [Prepare Lightroom: macOS](#prepare-lightroom-macos) where it is, until [After the tests](#after-the-tests).

**Expected**

- After step 2: the status bar reads "Restart Lightroom", and the notice "Restart Lightroom to finish updating Dive Lens." shows. Nothing can be written.
- After step 3:
  - Lightroom still lists **Create Collections…** and **Tag Selection…**: it reads its menus only when it starts.
  - Write down what choosing **Create Collections…** does: the app comes to the front, or Lightroom shows an error.
- After step 4: one item, **Open Dive Lens…**; the status bar reads "Lightroom connected".

#### JRN-11 · Lightroom quits during the journey

This test writes nothing.

**Before:** any photos.

**Steps**

1. [Catalog path to Clocks](#shortcuts-used-in-the-steps). On the Test Camera A card, type 0:05; under **Include photos taken at the surface**, click **+** twice (10 min).
2. Quit Lightroom. In the app, click **Next**.
3. Click **Back**. Start Lightroom again, on the test catalog.
4. Reset: set Test Camera A to 0:00, and leave without applying.

**Expected**

- After step 2:
  - the status bar reads "Lightroom not connected";
  - Clocks and Review & Apply say "Lightroom not connected: photos cannot be found or written.";
  - **Apply** is greyed.
- After step 3:
  - the status bar reads "Lightroom connected", and the photos are looked for again;
  - the Test Camera A card is still at 0:05, and the margin still at 10 min.

#### JRN-12 · The bottom bar stays in view at the smallest window size

**Before:** [clean start](#clean-start). Nothing selected in Lightroom (**Edit › Select None**).

**Steps**

1. **Library › Plug-in Extras › Open Dive Lens…**. Drag the corner of the app window to make it narrower and lower, until it stops: it cannot be made smaller.
2. At Scope, look at the bar, then choose **Entire catalog** and **Next**.
3. At Dives, scroll the dive list to its end. Click dive 101 and scroll its detail card to its end (profile, map, notes). Tick dives 101 and 102, then **Next**.
4. At Clocks, scroll to the end of the step, then **Next**.
5. At Review & Apply, choose **Other site, the same for all photos…** (the site picker makes the step longer) and scroll to the end. Choose **Each dive's site** again, then **Apply to 5 photos**.
6. On the summary, scroll to the end.
7. In Lightroom, select A1, A2, A3 and X1. [Selection path with dive 101](#shortcuts-used-in-the-steps), looking at each step on the way as above; at Review & Apply, choose **Other site…** and scroll to the end, then **Dive's site: Coral Garden** again, and **Apply to 4 photos**. Scroll the summary to its end.
8. Reset: make the window large again, then a [clean start](#clean-start).

**Expected**

- On every step of both paths:
  - the step strip stays at the top, and the bar (its text and its buttons: **Next**, **Back**, **Apply to N photos**, **Back to scope selection**, **Done**) stays at the bottom of the window, just above the status bar, whole;
  - only the content between them scrolls;
  - scrolled to its end, the content's last line shows whole above the bar, never under it;
  - nothing in the bar is cut off or overlapping (its text may wrap onto two lines).
- At Dives: the dive list and the detail card each scroll on their own, the bar still in view; when the window is too low for them, the whole step scrolls instead. Write down which of the two you see at the smallest size.

### Catalog path

#### CAT-01 · Create collections for two dives

**Before:** [clean start](#clean-start).

**Steps**

1. [Catalog path to Review & Apply](#shortcuts-used-in-the-steps).
2. Click **Apply to 5 photos**.

**Expected**

- A "Dive Lens" set with one collection per dive: "101 - Coral Garden" (A1, A2, A3) and "102 - Glacier Wall" (B1, B2).
- Dive Lens fields:
  - A1, A2, A3: dive 101, `MaxDepth` 18.4 m;
  - B1, B2: dive 102;
  - on the five photos: Clock Offset +00:00, Source "Subsurface".
- Photo depth: A1 18.0 m, A2 15.0 m, A3 10.0 m, B1 12.0 m.
- Location and GPS:
  - A1, A2, A3: country "Atlantis", city "North Reef", location "Coral Garden", GPS 12°30'N 45°15'W;
  - B1, B2: location "Glacier Wall", GPS -64.125, -60.5.
- GPS altitude empty.
- X1 untouched: no field, in no collection.

#### CAT-02 · Running again duplicates nothing

**Before:** after CAT-01.

**Steps**

1. [Catalog path to Review & Apply](#shortcuts-used-in-the-steps), then **Apply to 5 photos**.

**Expected**

- Nothing duplicated: one "Dive Lens" set, each collection holds its photos once.
- The summary lists both collections "added to".

#### CAT-03 · Country and period filters

This test writes nothing.

**Before:** any selection.

**Steps**

1. **Library › Plug-in Extras › Open Dive Lens…**, choose **Entire catalog**, **Next**, and stay on Dives.
2. Set **Country** to Lemuria.
3. Click **Clear filters**.
4. Set **Period** from 2025-06-17 to 2025-06-17.
5. Click **Clear filters**.

**Expected**

- After step 2: dive 104 only.
- After step 4: dives 106 and 107 only.
- After steps 3 and 5: every dive back.

#### CAT-04 · A dive without photos, handled separately

**Before:** [clean start](#clean-start).

**Steps**

1. **Library › Plug-in Extras › Open Dive Lens…**, choose **Entire catalog**, **Next**. Tick dives 101 and 103, then **Next**.
2. **Next**, then **Apply to 3 photos**.
3. On the summary, click **Handle dive 103 Shark Point / Bay separately**.
4. Reset: [clean start](#clean-start).

**Expected**

- At Clocks:
  - in the preview row of dive 103, its photo count, 0, and "No photo — clock changed?" are written in orange, not only tinted by a faint orange row;
  - the bar adds "Dive 103 Shark Point / Bay: no photo — clock changed?".
- At Review & Apply:
  - "Good to know before applying" starts with "Dive 103 Shark Point / Bay: no photo, no collection.", in orange;
  - **Create collections** reads "1 collection".
- On the summary:
  - only "101 - Coral Garden" is created;
  - the warning "Dive 103 Shark Point / Bay: no photo, no collection created. If the clock changed that day, handle it separately with its own offset.".
- After step 3: Clocks, with only dive 103 ticked and the photos looked for again. The search covers a day on either side of the dive, so the six test photos are found:
  - both camera cards read "0 photos in the dives";
  - in dive 103's row, the count 0 and "No photo — clock changed?" are written in orange;
  - "Outside the dives · 6 photos";
  - the bar reads "0 photos in 0 dives · 6 outside the dives".

#### CAT-05 · Dive details unticked

**Before:** [clean start](#clean-start).

**Steps**

1. [Catalog path to Review & Apply](#shortcuts-used-in-the-steps).
2. Untick **Dive details**.
3. Click **Apply to 5 photos**.
4. Reset: [clean start](#clean-start).

**Expected**

- After step 2: **Copy depth to GPS altitude** greyed; **Dive details** reads "Nothing"; the button still reads "Apply to 5 photos".
- After step 3:
  - No Dive Lens field on the five photos: no dive number, Source, Clock Offset or photo depth.
  - Location and GPS of each photo's dive, as in CAT-01.
  - The two collections, as in CAT-01.
  - The summary reads "With depth" 0.

#### CAT-06 · Site details unticked

**Before:** [clean start](#clean-start).

**Steps**

1. [Catalog path to Review & Apply](#shortcuts-used-in-the-steps).
2. Untick **Site details**.
3. Click **Apply to 5 photos**.
4. Reset: [clean start](#clean-start).

**Expected**

- After step 2: **Location** and **GPS** unticked and greyed, with the choice of the site; **Site details** reads "Nothing".
- After step 3: Dive Lens fields and collections as in CAT-01; no country, city, location or GPS on any photo.

#### CAT-07 · Create collections unticked

**Before:** [clean start](#clean-start).

**Steps**

1. [Catalog path to Review & Apply](#shortcuts-used-in-the-steps).
2. Untick **Create collections**.
3. Click **Apply to 5 photos**.
4. Reset: [clean start](#clean-start).

**Expected**

- After step 2: **Create collections** reads "None"; the bar reads "5 photos · no collection".
- After step 3:
  - Dive Lens fields, location and GPS as in CAT-01;
  - no "Dive Lens" set and no collection;
  - the summary reads "Collections" 0 and lists no collection.

#### CAT-08 · GPS alone, taken back by one Undo

Required on Windows.

**Before:** [clean start](#clean-start).

**Steps**

1. [Catalog path to Review & Apply](#shortcuts-used-in-the-steps).
2. Untick **Dive details**, **Location** and **Create collections**: only **GPS** stays ticked.
3. Click **Apply to 5 photos**.
4. Open Lightroom's **Edit** menu, then choose **Undo Dive Lens**, once.
5. Open the **Edit** menu again.

**Expected**

- After step 2: **Site details** reads "5 photos · GPS"; the bar reads "5 photos · no collection".
- After step 3:
  - only the GPS: A1, A2, A3 at 12°30'N 45°15'W, B1, B2 at -64.125, -60.5;
  - no Dive Lens field, no country, city or location, no collection.
- After step 4: no GPS left on any of the five photos.
- After step 5: the next Undo item is the one from before Apply (your clean start), not Dive Lens.
- If the GPS stays after one Undo, or a second **Undo Dive Lens** is offered, write it down.

#### CAT-09 · Another site for every photo

**Before:** [clean start](#clean-start).

**Steps**

1. [Catalog path to Review & Apply](#shortcuts-used-in-the-steps).
2. Choose **Other site, the same for all photos…**.
3. Choose the country Atlantide, the location Île Verte and the site Récif d'Émeraude.
4. Type 95 as the **Latitude**.
5. Type -20.6 as the **Latitude**.
6. Click **Apply to 5 photos**.
7. Reset: [clean start](#clean-start).

**Expected**

- After step 2: **Apply** greyed, with "Choose a site".
- After step 3: the site's GPS in the fields, -20.5 and 57.25; **Apply** enabled.
- After step 4: "Latitude between -90 and 90, longitude between -180 and 180, or both empty." and **Apply** greyed.
- After step 6:
  - On the five photos: country "Atlantide", city "Île Verte", location "Récif d'Émeraude", GPS 20°36'S 57°15'E.
  - Dive Lens fields of each photo's own dive, as in CAT-01.
  - The collections keep their names, "101 - Coral Garden" and "102 - Glacier Wall".

#### CAT-10 · Collections alone

**Before:** [clean start](#clean-start).

**Steps**

1. [Catalog path to Review & Apply](#shortcuts-used-in-the-steps). Untick **Dive details** and **Site details**.
2. Click **Apply to 5 photos**.
3. Play steps 1 and 2 again.
4. Open the **Edit** menu. If it reads **Undo Dive Lens**, choose it, once; otherwise write down what it reads and do not undo.
5. Reset: [clean start](#clean-start).

**Expected**

- After step 1: the bar reads "2 collections · nothing is written on the photos"; the button reads "Apply to 5 photos".
- After step 2:
  - a "Dive Lens" set with "101 - Coral Garden" (A1, A2, A3) and "102 - Glacier Wall" (B1, B2);
  - no field, location or GPS on any photo;
  - the summary reads "Photos tagged" 5, "With depth" 0, "Collections" 2, both "created".
- After step 3: the summary lists both collections "added to"; nothing changes in Lightroom.
- After step 4: write down what one Undo changed: nothing (the second run was its own step), or the collections emptied or gone.

#### CAT-11 · A surface photo goes to the closer dive

**Before:** [clean start](#clean-start).

**Steps**

1. [Catalog path to Clocks](#shortcuts-used-in-the-steps).
2. Under **Include photos taken at the surface**, click **+** until it reads 120 min.
3. **Next**, then **Apply to 6 photos**.
4. On the summary, **Back to scope selection**. At Scope, **Next**; tick dives 101 and 102, then **Next**.
5. Reset: [clean start](#clean-start).

**Expected**

- After step 2:
  - the block "Outside the dives" goes away;
  - the preview shows 4 photos for dive 101 and 2 for dive 102;
  - the Test Camera A card reads "4 in the dives · 0 outside".
- After step 3: X1 is in "101 - Coral Garden", not in "102 - Glacier Wall". It has:
  - dive 101's fields, location and GPS;
  - Clock Offset +00:00;
  - no photo depth, temperature or elapsed time.
- After step 4:
  - "Back to scope selection" led to Scope, with **Entire catalog** still chosen, and Dives showed no dive ticked;
  - at Clocks, the margin is back to 0 min.

#### CAT-12 · GPS and notes marks and filters

This test writes nothing.

**Before:** any selection.

**Steps**

1. **Library › Plug-in Extras › Open Dive Lens…**, choose **Entire catalog**, **Next**, and stay on Dives.
2. Look at the end of each row. Hover over the marks of dives 101, 102 and 103.
3. Click the filter **With GPS**.
4. Click **With notes** as well.
5. Click **With GPS** again, so that only **With notes** is on.
6. Click **Clear filters**.

**Expected**

- After step 2:
  - after the **Profile** column, a **GPS** column and a **Notes** column, each with its mark: in colour when the dive has it, greyed when not. Subsurface gives site positions, so the GPS column shows; a dive application that does not give them would have no GPS column and no **With GPS** filter;
  - dive 101: GPS and notes in colour, tooltips "Site GPS available" and "Notes available";
  - dive 102: GPS in colour, notes greyed, "No notes";
  - dive 103: GPS greyed, "No site GPS", notes in colour.
- After step 3: dives 100, 101, 102, 104, 108 and 109 only.
- After step 4: dives 100, 101 and 108 only.
- After step 5: dives 100, 101, 103 and 108 only.
- After step 6: every dive back, and both filters off.
- In French (optional): « Avec GPS », « Avec notes », « GPS du site disponible », « Pas de GPS du site », « Notes disponibles », « Pas de notes ».

### Selection path

#### SEL-01 · Tag three photos with one dive

**Before:** [clean start](#clean-start). Select A1, A2, A3.

**Steps**

1. [Selection path with dive 101](#shortcuts-used-in-the-steps).
2. Click **Apply to 3 photos**.

**Expected**

- Dive Lens fields: dive 101, `MaxDepth` 18.4 m.
- Photo depth: A1 18.0 m, A2 15.0 m, A3 10.0 m.
- Location: country "Atlantis", city "North Reef", location "Coral Garden".
- GPS 12°30'N 45°15'W.
- GPS altitude empty.
- A "Dive Lens" set with "101 - Coral Garden", holding A1, A2, A3; the summary lists it "created".

#### SEL-02 · The dive's collection is completed, without duplicates

**Before:** after SEL-01. Select A1, A2, A3 and X1.

**Steps**

1. [Selection path with dive 101](#shortcuts-used-in-the-steps). On the way, read the Clocks step.
2. Click **Apply to 4 photos**.

**Expected**

- At Clocks:
  - "Outside the dive's times · 1 photo";
  - "Taken at 12:00, 108 minutes after getting out of the water. Attached to the dive anyway: it gets its data and its location, without depth.".
- On the summary:
  - "101 - Coral Garden", "4 photos · added to";
  - "1 photo without depth: taken after getting out of the water.".
- "101 - Coral Garden" holds A1, A2, A3 and X1, each once.
- X1:
  - dive 101's fields, location and GPS;
  - no photo depth, temperature, elapsed time or clock offset.
- A1, A2, A3 keep the values of SEL-01.

#### SEL-03 · Tag a photo again with another dive

**Before:** after SEL-02. Select A1.

**Steps**

1. [Selection path with dive 102](#shortcuts-used-in-the-steps).
2. Click **Apply to 1 photo**.

**Expected**

- No refusal.
- Dive Lens fields: dive 102, Glacier Wall.
- City "Frostville"; GPS -64.125, -60.5.
- Photo depth, photo temperature, elapsed time and clock offset empty.
- A1 is in a new "102 - Glacier Wall" and still in "101 - Coral Garden".

#### SEL-04 · Tagging again with the same dive changes nothing

**Before:** after SEL-03. Select A1.

**Steps**

1. [Selection path with dive 102](#shortcuts-used-in-the-steps).
2. Click **Apply to 1 photo**.

**Expected**

- No change on A1.
- The summary lists "102 - Glacier Wall", "1 photo · added to"; the collection holds A1 once.

#### SEL-05 · The selection emptied: nothing is written

This test writes nothing.

**Before:** any photos. Select A1.

**Steps**

1. [Selection path with dive 101](#shortcuts-used-in-the-steps).
2. In Lightroom, deselect everything: **Edit › Select None**.
3. In the app, click **Scope** in the strip.

**Expected**

- After step 2:
  - the app stays at Review & Apply;
  - **Apply** is greyed, with "Select photos in Lightroom";
  - the status bar reads "Selected photos: none".
- After step 3: the selection card is greyed, "No photos selected", and **Next** is greyed.
- Nothing is written.

#### SEL-06 · The selection changes before Apply

**Before:** any photos. Select A1.

**Steps**

1. [Selection path with dive 101](#shortcuts-used-in-the-steps).
2. In Lightroom, select B1 instead.
3. Click **Apply** in the app.
4. Reset: [clean start](#clean-start) (B1 may also be in "101 - Coral Garden").

**Expected**

- B1 never gets A1's values. Either:
  - a refusal, "The selection changed in Lightroom. Nothing was written: check the per-photo data, then apply again.", and nothing written;
  - or the app follows B1 and writes B1's values: dive 101, without photo depth (B1 is taken after the dive).

#### SEL-07 · The dive that covers the selection is marked

This test writes nothing.

**Before:** any photos. Select A1, A2, A3.

**Steps**

1. **Library › Plug-in Extras › Open Dive Lens…**, choose the **Lightroom selection** card, **Next**, without choosing a dive.
2. With the app still open, select X1 alone in Lightroom.
3. Then select A1 and B1 together.

**Expected**

- After step 1:
  - dive 101 marked "Covers the selection" and shown in the detail card;
  - when Test Camera A has a remembered offset, under the row: "Covers the selection: 3 of the 3 photos fall in this dive, with the remembered offset of the Test Camera A (0:00).".
- After step 2: no dive marked.
- After step 3: no dive marked.

#### SEL-08 · Search and rating filters

This test writes nothing.

**Before:** any selection of at least one photo.

**Steps**

1. **Library › Plug-in Extras › Open Dive Lens…**, choose the **Lightroom selection** card, **Next**.
2. Type "Glacier" in the search field.
3. Click **Clear filters**.
4. Set **Rating** to "5★ and more".
5. Click **Clear filters**.

**Expected**

- After step 2: dive 102 only, under the row "No dive — location only".
- After step 4: dive 107 only.
- After steps 3 and 5: every dive back.

#### SEL-09 · No dive — location only

**Before:** [clean start](#clean-start). Select X1.

**Steps**

1. **Library › Plug-in Extras › Open Dive Lens…**, choose the **Lightroom selection** card, **Next**.
2. Click **No dive — location only**, then **Next**.
3. Click **Back**, then **Next** again.
4. Choose the country Atlantide, the location Île Verte and the site Récif d'Émeraude.
5. Click **Apply to 1 photo**, then **Done** on the summary.

**Expected**

- After step 2:
  - Review & Apply, with **Clocks** greyed in the strip;
  - **Dive details** and **Create collections** greyed and unticked;
  - "Dive's site: no dive chosen" greyed, **Other site…** chosen;
  - **Apply** greyed, with "Choose a site".
- After step 3: **Back** leads to Dives, and **Next** to Review & Apply.
- After step 5:
  - X1: country "Atlantide", city "Île Verte", location "Récif d'Émeraude", GPS 20°30'S 57°15'E;
  - no Dive Lens field (no dive number, no Source) and no collection;
  - the summary reads "The selected photo · site details", "Photos tagged" 1, "With depth" 0, "Collections" 0;
  - **Done** leads to Scope.

### Camera clock offset

Play these two tests one after the other: CLK-02 ends by setting Test Camera B back to 0:00, which the other tests need.

#### CLK-01 · Suggest a clock offset

**Before:** [clean photos](#clean-photos). Select B1, B2.

**Steps**

1. **Library › Plug-in Extras › Open Dive Lens…**, choose the **Lightroom selection** card, **Next**.
2. Click dive **101** (not the one marked "Covers the selection"), then **Next**.
3. On the Test Camera B card, click **Suggest**.
4. **Next**, then **Apply to 2 photos**.

**Expected**

- Suggest gives −4:30.
- B1 and B2: dive 101, Clock Offset -04:30.
- Photo depth: B1 18.0 m, B2 15.0 m.

#### CLK-02 · The offset is remembered, on both paths

**Before:** after CLK-01. Select B1, B2.

**Steps**

1. **Library › Plug-in Extras › Open Dive Lens…**, choose the **Lightroom selection** card, **Next**: look at Dives. Click dive 101, **Next**: look at the Test Camera B card.
2. **Library › Plug-in Extras › Open Dive Lens…**, choose **Entire catalog**, **Next**, tick dive 101, **Next**: look at the Test Camera B card. Leave without applying.
3. Reset: [selection path](#shortcuts-used-in-the-steps) on B1, B2 with dive 102, but at **Clocks** set Test Camera B to 0:00 before **Next**; then **Apply to 2 photos**.

**Expected**

- After step 1: dive 101 marked "Covers the selection"; the Test Camera B card at −4:30 with "remembered offset".
- After step 2: the Test Camera B card at −4:30 with "remembered offset".

### Units and altitude

#### UNIT-01 · Writing in feet, with the depth copied to the altitude

**Before:** [clean photos](#clean-photos). Select A1, A2, A3.

**Steps**

1. Click the gear, set **Display › Depth** to Feet, then **Back**.
2. [Selection path with dive 101](#shortcuts-used-in-the-steps). At **Review & Apply**, tick **Copy depth to GPS altitude**, then **Apply to 3 photos**.
3. Reset: set Depth back to Metres.

**Expected**

- At Review & Apply: "Units: ft, °C"; **Copy depth to GPS altitude** reads "3 photos".
- Photo depth: A1 about 59 ft.
- Max depth about 60 ft.
- GPS altitude about -18 m (Lightroom may show it in feet).

#### UNIT-02 · The altitude option is unticked on every run

**Before:** after UNIT-01. Select A1, A2, A3.

**Steps**

1. [Selection path with dive 101](#shortcuts-used-in-the-steps). Look at **Copy depth to GPS altitude**.
2. Untick **Dive details**, then tick it again.
3. Click **Apply to 3 photos** without ticking the altitude.
4. Reset: [clean photos](#clean-photos).

**Expected**

- After step 1: **Copy depth to GPS altitude** unticked, reading "No".
- During step 2: unticking **Dive details** greys it.
- After step 3:
  - the GPS altitude UNIT-01 copied stays, about -18 m on A1;
  - photo depth A1 18.0 m, in metres again.

#### UNIT-03 · Depths in the dive list, in feet and in metres

This test writes nothing.

**Before:** any selection.

**Steps**

1. Click the gear, set **Display › Depth** to Feet, then **Back**.
2. **Library › Plug-in Extras › Open Dive Lens…**, choose **Entire catalog**, **Next**, and read the depths in the dive list.
3. Set Depth back to Metres and read them again.

**Expected**

- After step 2: dive 101 60 ft, dive 108 102 ft.
- After step 3: dive 101 18.4 m, dive 108 31.2 m.

#### UNIT-04 · The GPS altitude shows under the GPS in the Dive Lens panel

Nobody has seen this line of the panel in Lightroom yet: this test observes and writes down.

**Before:** [clean photos](#clean-photos). Select A1 and X1.

**Steps**

1. [Selection path with dive 101](#shortcuts-used-in-the-steps). At **Review & Apply**, tick **Copy depth to GPS altitude**, then **Apply to 2 photos**.
2. In the Library module, select A1 alone. In the Metadata panel, choose **Dive Lens** in its menu and look at the **Site** section.
3. Select X1 alone and look again.
4. Reset: [clean start](#clean-start).

**Expected**

- After step 1: **Copy depth to GPS altitude** read "1 photo" before Apply (X1 is taken after the dive: no depth).
- After step 2: in **Site**, after location, city, country and GPS, a line for the GPS altitude, about -18 m below sea level. Write down its label and its value exactly as Lightroom shows them (Lightroom may show it in feet, or as a height below sea level).
- After step 3: write down what that line shows for a photo without altitude: an empty line, or no line.
- Write down whether the line can be edited in the panel.

### Settings

#### SET-01 · Dive log not found

This test writes nothing.

**Before:** copy `core/fixtures/subsurface/fictional.xml` into the test folder, next to the test catalog.

**Steps**

1. In Dive Lens settings, **Dive log**: **Choose a file…** and choose that copy.
2. Quit Dive Lens.
3. Rename the copy.
4. **Library › Plug-in Extras › Open Dive Lens…**.
5. On the screen shown, **Choose a file…** and choose `core/fixtures/subsurface/fictional.xml` of the repository.

**Expected**

- After step 4: a screen "Subsurface dive log not found", with the copy's former path, the buttons **Choose a file…** and **Choose a folder…**, and no **Try again**.
- After step 5: step 1 of the journey.

#### SET-02 · Check for updates

This test writes nothing.

**Before:** connected to the internet.

**Steps**

1. In Dive Lens settings, **Updates**: click **Check now**.

**Expected**

- When the version played is the latest one published: "Up to date · last checked today at" the current time.
- Otherwise: "Version … available", with the latest version.

A development build never looks for updates by itself (only an installed copy does, at most once a day): on a development build, no update notice on opening proves nothing. SET-02 asks GitHub on any build.

#### SET-03 · No altitude line in Settings; an older settings file still imports

This test writes nothing.

**Before**

1. With the base settings, in Dive Lens settings: **Export or import › Export…**.
2. In a text editor, add `"depthToAltitude": true,` right after the first `{` of that file, as Dive Lens 2.1.1 and earlier wrote it, and save.

**Steps**

1. In Dive Lens settings, look at the sections.
2. **Export or import › Import…**, choose that file, then **Replace**.
3. **Export…** again, to a new file, and open it in the text editor.
4. [Catalog path to Review & Apply](#shortcuts-used-in-the-steps), and leave without applying.

**Expected**

- After step 1: the sections Dive log, Display, Collections, Remembered clock offsets, Settings backup, Updates, Diagnostics, About, in that order. No "Writing on photos", no "Copy depth to GPS altitude".
- After step 2: "Settings imported."; every setting as before.
- After step 3: the new file has no `depthToAltitude`.
- After step 4: **Copy depth to GPS altitude** unticked.

#### SET-04 · The collection set chosen in Settings is missing

This test writes nothing.

**Before:** in Lightroom's Collections panel, create a collection set named "Trips" at the catalog root (**+** › **Create Collection Set…**).

**Steps**

1. In Dive Lens settings, **Collections › Place in**: choose Trips, then **Back**.
2. [Catalog path to Review & Apply](#shortcuts-used-in-the-steps).
3. Leave the app at Review & Apply. In Lightroom, delete the "Trips" set.
4. In the app: **Back**, **Back**, **Next**, **Next** (back from Review & Apply to Dives, then on to Review & Apply).
5. Untick **Create collections**.
6. Tick it again, then click **Open Settings**.
7. Reset: **Place in**: Catalog root, then **Back**; leave without applying.

**Expected**

- After step 2: the text of **Create collections** reads "…for example Trips › Dive Lens › 101 - Coral Garden.".
- After step 4:
  - "The collection set chosen in Settings (Trips) is no longer in the catalog. Choose another place in Settings.";
  - **Apply** greyed;
  - an **Open Settings** button.
- After step 5: the message goes; **Apply** is enabled.
- After step 6:
  - Settings show "Trips (not found)";
  - with "This collection set is no longer in the catalog: choose another.".
- After step 7: Review & Apply, **Apply** enabled.

#### SET-05 · Forget the remembered clock offset of one camera

**Before:** [clean start](#clean-start).

**Steps**

1. [Catalog path to Review & Apply](#shortcuts-used-in-the-steps) (at **Clocks**, set any card that is not at 0:00 to 0:00), then **Apply to 5 photos**. Test Camera A and Test Camera B are now remembered at 0:00.
2. In Lightroom, select B1 and B2. **Library › Plug-in Extras › Open Dive Lens…**, choose the **Lightroom selection** card, **Next**; click dive **101**, **Next**; on the Test Camera B card, click **Suggest** (−4:30); **Next**, then **Apply to 2 photos**.
3. Click the gear and read the section **Remembered clock offsets**.
4. On the Test Camera B line, click **Forget**.
5. Click **Back**. **Library › Plug-in Extras › Open Dive Lens…**, choose **Entire catalog**, **Next**, tick dives 101 and 102, **Next**: look at the two camera cards. Leave without applying.
6. In Lightroom, look at the Dive Lens fields of B1 and B2.
7. Click the gear, and on the Test Camera A line, click **Forget**. Then **Back**.
8. Reset: [clean start](#clean-start).

**Expected**

- After step 3:
  - the section sits between **Collections** and **Settings backup**, with "Kept for each camera after Apply and offered again at the Clocks step, for Subsurface.";
  - one line per camera, by name: "Test Camera A" with "0:00", and "Test Camera B" with "−4:30", each with a **Forget** button. No "no. …" under the names: the test photos carry no serial number. Cameras of your own remembered for Subsurface may be listed too: leave them.
- After step 4: the Test Camera B line goes at once, with no confirmation; the Test Camera A line stays, at 0:00.
- After step 5: the Test Camera A card at 0:00 with "remembered offset"; the Test Camera B card at 0:00 without "remembered offset".
- After step 6: B1 and B2 still carry dive 101 and Clock Offset -04:30: forgetting changes nothing on photos already written.
- After step 7: no Test Camera line left. Without cameras of your own, the section reads "No clock offset remembered.".
- In French (optional): « Décalages d'horloge mémorisés », « Gardés pour chaque appareil après « Appliquer » et proposés à nouveau à l'étape Horloges, pour Subsurface. », « Oublier », « Aucun décalage d'horloge mémorisé. ».

#### SET-06 · Forgetting an offset during a journey

**Before:** [clean start](#clean-start). Select B1 and B2.

**Steps**

1. **Library › Plug-in Extras › Open Dive Lens…**, choose the **Lightroom selection** card, **Next**; click dive **101**, **Next**; on the Test Camera B card, click **Suggest** (−4:30); **Next**, then **Apply to 2 photos**. On the summary, **Done**.
2. **Next**: look at Dives. Click dive 101, **Next**: look at the Test Camera B card.
3. Click the gear. Under **Remembered clock offsets**, click **Forget** on the Test Camera B line, then **Back**.
4. Click **Back**, to Dives.
5. **Next**, **Next**, then **Apply to 2 photos**. Click the gear and look at **Remembered clock offsets**; click **Forget** on the Test Camera B line, then **Back**.
6. On the summary, **Done**, then **Next**: look at Dives. Click dive 101, **Next**: look at the Test Camera B card. Leave without applying.
7. Reset: [clean start](#clean-start).

**Expected**

- After step 2:
  - dive 101 marked "Covers the selection", with under its row "Covers the selection: 2 of the 2 photos fall in this dive, with the remembered offset of the Test Camera B (−4:30).";
  - the Test Camera B card at −4:30 with "remembered offset".
- After step 3: back at Clocks, the Test Camera B card still at −4:30, but without "remembered offset"; "2 in the dive · 0 outside", as before.
- After step 4: dive 101 still marked "Covers the selection" (it follows the offset shown), but without the sentence under its row.
- After step 5:
  - B1 and B2 carry Clock Offset -04:30;
  - in Settings, "Test Camera B" with "−4:30" again: **Apply** remembered it again.
- After step 6:
  - dive 102 marked "Covers the selection", not 101, and no sentence under its row: the new run starts Test Camera B at 0:00;
  - the Test Camera B card at 0:00, without "remembered offset", in orange, "0 in the dive · 2 outside".

### Undo

#### UNDO-01 · Catalog path: one Undo takes everything back, GPS and altitude included

Required on Windows.

**Before:** [clean start](#clean-start): no "Dive Lens" set.

**Steps**

1. [Catalog path to Review & Apply](#shortcuts-used-in-the-steps). Tick **Copy depth to GPS altitude**, then **Apply to 5 photos**.
2. Look at the photos and at Lightroom's **Edit** menu.
3. **Edit › Undo Dive Lens**, once.
4. Open the **Edit** menu. If it reads **Undo Dive Lens**, choose it; otherwise write down what it reads and do not undo.

**Expected**

- After step 2:
  - GPS altitude: A1 -18.0 m, A2 -15.0 m, A3 -10.0 m, B1 -12.0 m;
  - the Edit menu reads **Undo Dive Lens** (in a French Lightroom, **Édition › Annuler Dive Lens**).
- After step 3:
  - on the five photos, the Dive Lens fields, country, city, location, GPS and GPS altitude are empty;
  - the two collections are gone;
  - write down whether the "Dive Lens" set is still there, empty, or gone too.
- After step 4: if the set was still there, this Undo removes it. Either way, the next Undo item is then no longer Dive Lens's.

#### UNDO-02 · Selection path: one Undo takes everything back, GPS and altitude included

Required on Windows.

**Before:** [clean start](#clean-start), then create an empty collection set named "Dive Lens" at the catalog root yourself (Collections panel, **+** › **Create Collection Set…**). Select A1, A2, A3.

**Steps**

1. [Selection path with dive 101](#shortcuts-used-in-the-steps). At **Review & Apply**, tick **Copy depth to GPS altitude**, then **Apply to 3 photos**.
2. **Edit › Undo Dive Lens**, once.
3. Open the **Edit** menu again.
4. Reset: [clean start](#clean-start).

**Expected**

- After step 1:
  - GPS altitude: A1 -18.0 m, A2 -15.0 m, A3 -10.0 m;
  - "101 - Coral Garden" created inside your "Dive Lens" set;
  - the Edit menu reads **Undo Dive Lens**.
- After step 2:
  - on A1, A2, A3, the Dive Lens fields, location, GPS and GPS altitude are empty;
  - "101 - Coral Garden" is gone; your "Dive Lens" set stays.
- After step 3: the next Undo item is no longer Dive Lens's.

#### UNDO-03 · Undo when Apply creates the "Dive Lens" set

Required on Windows.

**Before:** [clean start](#clean-start): no "Dive Lens" set. Select A1.

**Steps**

1. [Selection path with dive 101](#shortcuts-used-in-the-steps), then **Apply to 1 photo**.
2. **Edit › Undo Dive Lens**.
3. Open the **Edit** menu. If it reads **Undo Dive Lens**, choose it; otherwise write down what it reads and do not undo.

**Expected**

- After step 2:
  - A1 has no Dive Lens field, location or GPS left;
  - "101 - Coral Garden" is gone;
  - write down whether the "Dive Lens" set is still there, empty, or gone too.
- After step 3: if the set was still there, this Undo removes it. Either way, the next Undo item is then no longer Dive Lens's.

The set is created just before the write. If Lightroom then refuses the write (Lightroom busy, or an error), the set stays, empty: **Undo Dive Lens**, or deleting it in Lightroom, removes it. The checks cannot cause such a refusal on purpose: should one happen during any test, write down what you see.

## After the tests

1. In Dive Lens settings, import your own settings back (**Export or import › Import…**).
2. macOS development build: remove the repository plug-in in the Plug-in Manager, quit Lightroom, and put `DiveLens.lrplugin` back in Modules.

## Results sheet

Copy this sheet into the pull request and fill it in. Write **OK**, **not played**, or the difference described in words. On Windows, UNDO-01, UNDO-02, UNDO-03 and CAT-08 are required.

Describe differences in words only. No real dive data, no screenshot of a real catalog.

```markdown
Lightroom checks — Dive Lens <version>
macOS <version>, Lightroom Classic <version>
Windows <version>, Lightroom Classic <version>

| Test | macOS | Windows |
|---|---|---|
| JRN-01 Every screen of the catalog path, in English and in French | | |
| JRN-02 Every screen of the selection path, in English and in French | | |
| JRN-03 One menu item, enabled without a selection | | |
| JRN-04 The Scope step follows the Lightroom selection live | | |
| JRN-05 The catalog size matches All Photographs | | |
| JRN-06 "Open Dive Lens…" during the journey goes back to Scope | | |
| JRN-07 Settings, then Back, keep the step and the choices | | |
| JRN-08 Another dive log in Settings starts the journey again | | |
| JRN-09 Nothing leaves Review & Apply during a write | | |
| JRN-10 A plug-in of another version asks to restart Lightroom | | |
| JRN-11 Lightroom quits during the journey | | |
| JRN-12 The bottom bar stays in view at the smallest window size | | |
| CAT-01 Create collections for two dives | | |
| CAT-02 Running again duplicates nothing | | |
| CAT-03 Country and period filters | | |
| CAT-04 A dive without photos, handled separately | | |
| CAT-05 Dive details unticked | | |
| CAT-06 Site details unticked | | |
| CAT-07 Create collections unticked | | |
| CAT-08 GPS alone, taken back by one Undo | | |
| CAT-09 Another site for every photo | | |
| CAT-10 Collections alone | | |
| CAT-11 A surface photo goes to the closer dive | | |
| CAT-12 GPS and notes marks and filters | | |
| SEL-01 Tag three photos with one dive | | |
| SEL-02 The dive's collection is completed, without duplicates | | |
| SEL-03 Tag a photo again with another dive | | |
| SEL-04 Tagging again with the same dive changes nothing | | |
| SEL-05 The selection emptied: nothing is written | | |
| SEL-06 The selection changes before Apply | | |
| SEL-07 The dive that covers the selection is marked | | |
| SEL-08 Search and rating filters | | |
| SEL-09 No dive — location only | | |
| CLK-01 Suggest a clock offset | | |
| CLK-02 The offset is remembered, on both paths | | |
| UNIT-01 Writing in feet, with the depth copied to the altitude | | |
| UNIT-02 The altitude option is unticked on every run | | |
| UNIT-03 Depths in the dive list, in feet and in metres | | |
| UNIT-04 The GPS altitude shows under the GPS in the Dive Lens panel | | |
| SET-01 Dive log not found | | |
| SET-02 Check for updates | | |
| SET-03 No altitude line in Settings; an older settings file still imports | | |
| SET-04 The collection set chosen in Settings is missing | | |
| SET-05 Forget the remembered clock offset of one camera | | |
| SET-06 Forgetting an offset during a journey | | |
| UNDO-01 Catalog path: one Undo takes everything back, GPS and altitude included | | |
| UNDO-02 Selection path: one Undo takes everything back, GPS and altitude included | | |
| UNDO-03 Undo when Apply creates the "Dive Lens" set | | |
```

## Old identifiers

Until version 2.1.1, the tests named the two former screens, Tag Selection and Create Collections. Each of them is played again here, through the journey. None was removed.

| Old identifier | New identifier | What changed |
|---|---|---|
| TAG-01 Tag three photos with one dive | SEL-01 | The photos now also go in the dive's collection (option ticked by default). |
| TAG-02 One Undo takes the tag back | UNDO-02 | With the depth copied to the altitude, so that the altitude's Undo is checked too. |
| TAG-03 Tag a photo again with another dive | SEL-03 | |
| TAG-04 Tagging again with the same dive changes nothing | SEL-04 | |
| TAG-05 No photo selected: nothing is written | SEL-05 | The menu item is now enabled without a selection (JRN-03): the selection is emptied once the journey is open. |
| TAG-06 The selection changes before Apply | SEL-06 | |
| TAG-07 The dive that covers the selection is marked | SEL-07 | |
| TAG-08 Search and rating filters | SEL-08 | |
| COL-01 Create collections for two dives | CAT-01 | |
| COL-02 One Undo takes the collections back | UNDO-01 | With the depth copied to the altitude. The old check said the set needed a second Undo; the tester now writes down whether the set is still there after one Undo. |
| COL-03 Running again duplicates nothing | CAT-02 | |
| COL-04 Country and period filters | CAT-03 | |
| CLK-01 Suggest a clock offset | CLK-01 | |
| CLK-02 The offset is remembered | CLK-02 | |
| UNIT-01 Writing in feet, with the depth copied to the altitude | UNIT-01 | The altitude is ticked at Review & Apply, no longer in Settings. |
| UNIT-02 Depths in the dive list, in feet and in metres | UNIT-03 | |
| SET-01 Dive log not found | SET-01 | |
| SET-02 Check for updates | SET-02 | |
