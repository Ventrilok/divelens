# Réorganisation de Dive Lens — plan des phases 0 à 2

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Écrire la recette Lightroom et en tirer la référence (phase 0), réorganiser le plug-in, le moteur et l'interface par fonctionnalité sans changer aucun comportement (phase 1), puis vérifier que tout se comporte comme la référence (phase 2).

**Architecture:** Déplacements purs, chacun protégé par des tests écrits avant (tests « épingles » qui passent sur le code actuel et doivent continuer à passer). Les décisions qui n'ont pas besoin de Lightroom quittent les enveloppes d'écriture du plug-in pour `Tagging.lua`, testé sous Lua 5.1. Les phases 3 et 4 (fusion des circuits « Appliquer », corrections) font l'objet d'un plan séparé, écrit après le point de contrôle de la phase 2 à partir des notes en annexe.

**Tech Stack:** Lua 5.1 (plug-in Lightroom Classic, sans compilation), Rust 2024 + Tauri 2 (crate `divelens`, bibliothèque `divelens_lib`), React + TypeScript (Vite, Vitest en environnement `node`, fuseau UTC+14).

**Spec:** `docs/superpowers/specs/2026-11-19-reorganisation-design.md`

## Global Constraints

- Aucun changement de comportement en phase 1 : ce qui est écrit sur les photos, l'ordre des deux écritures, « Annuler », les messages app ↔ plug-in, le format des réglages, les textes affichés, les journaux (mêmes lignes, mêmes préfixes « tagging refused » / « collections refused »).
- `local PROTOCOL = 6` reste dans `plugin/DiveLens.lrdevplugin/Link.lua`, écrit exactement ainsi, une seule fois (test `the_plugin_speaks_the_same_protocol`, `app/src-tauri/src/lightroom.rs:916-933`).
- Jamais renommés : `com.querton.divelens`, les identifiants de champs, l'ensemble « Dive Lens » et `<numéro> - <site>`, `com.querton.divelens.desktop`, les noms des commandes Tauri (ce sont les chaînes `invoke` de `app/src/api.ts`).
- Plug-in à plat, sans sous-dossier ; aucun fichier ajouté à `Info.lua` (les modules ne s'y déclarent pas). Aucun module n'a d'état mutable au niveau du fichier : tout passe par la session `s`. Aucun module ne fait `require "Link"` (cycle).
- Rust : modules style 2024 (`x.rs` à côté de `x/`, pas de `mod.rs`). Une commande déplacée devient `pub(crate)` et figure dans `generate_handler!` par son chemin de module (`tag::tag_selection`) ; pas de `pub use` comme substitut. Les `cfg_attr(..., allow(dead_code))` suivent leurs éléments.
- Rien ne bouge dans `core/` ni `libdivecomputer-sys/`. Le sous-module `third_party/libdivecomputer` ne bouge pas.
- Le numéro de version ne change pas.
- Une branche par pull request (`docs/…`, `test/…`, `refactor/…`), Conventional Commits, CI verte macOS + Windows, fusion par squash. Push, ouverture de PR et fusion : seulement avec l'accord explicite du propriétaire.
- Le dépôt local du propriétaire porte des fichiers à ne jamais indexer : `CLAUDE.md` (modifications en cours) et des documents privés non suivis. Toujours `git add <chemins>` explicites, jamais `git add -A` ni `git add .`.
- `CLAUDE.md` n'est modifié qu'avec l'accord du propriétaire : les chemins devenus faux y sont signalés dans la PR, pas corrigés.
- Avant chaque commit, les vérifications du guide des contributeurs pour la partie touchée : toujours `cargo fmt --all --check` ; plug-in, moteur ou interface : dans `app/`, `npm test` puis `npm run build`, puis `cargo clippy -p divelens --all-targets -- -D warnings` et `cargo test -p divelens`.
- Données de test fictives uniquement. Aucun catalogue Lightroom (`.lrcat`) dans le dépôt.
- Les numéros de ligne cités sont ceux du 2026-11-19, avant tout changement.

## Review Focus

1. **Nom introuvable après un déplacement dans le plug-in** (un `local` resté dans `Link.lua`, comme `send` ou `WRITE_TIMEOUT`, utilisé dans un nouveau fichier) : Lua 5.1 le compile comme une variable globale qui vaut `nil` à l'exécution, et seule l'exécution dans Lightroom le révèle. → Tâche 5 : `plugin_globals_are_known` liste, pour chaque fichier du plug-in, les globales lues **dans tout le fichier, corps de fonctions compris** (lecture du code compilé, instruction `GETGLOBAL` de Lua 5.1) et les compare à une liste autorisée. Le test de chargement strict ne voit que le chargement ; la recette ne provoque pas les branches d'erreur (`busy`, `parentNotFound`, `error`), là où un appel à `nil` ramènerait l'écran bloqué.
2. **Cycle de `require` ou mauvais nom de module** : invisible aux tests actuels, qui ne chargent pas `Link.lua`. → Tâche 5 : `plugin_modules_load_without_lightroom`.
3. **Commande Tauri oubliée dans `generate_handler!`** : ça compile, puis l'écran échoue à l'appel. → Tâche 6 : `every_invoked_command_is_registered`, écrit avant tout déplacement du moteur.
4. **Ordre des effets changé dans « Tagger la sélection »** en sortant la liste des plongées : le focus ne suivrait plus la plongée qui couvre la sélection. Pas testable sous Vitest (environnement `node`). → Tâche 14 : le hook est appelé exactement là où sont les lignes 146-156 aujourd'hui ; contrôle à la main en phase 2.
5. **Code compilé seulement sous Windows** (mises à jour, unités, journal) déplacé avec un mauvais chemin : seule la CI Windows le voit. → Tâches 8 et 12 : CI Windows verte exigée avant fusion, et `cfg_attr` déplacés avec leurs éléments.

---

## Phase 0 — Recette et référence

### Task 0: Recette Lightroom, photos fictives, documents du chantier

PR `docs: add the lightroom checks and the reorganisation design`, branche `docs/lightroom-checks`.

**Files:**
- Create: `docs/contributing/lightroom-checks.md` (en anglais, comme le reste du guide des contributeurs)
- Create: `docs/contributing/lightroom-checks/A1.jpg`, `A2.jpg`, `A3.jpg`, `B1.jpg`, `B2.jpg`, `X1.jpg`
- Modify: `CONTRIBUTING.md`, section « Trying the plug-in in Lightroom » : une phrase qui renvoie à la recette
- Add (déjà écrits, non indexés) : `docs/superpowers/specs/2026-11-19-reorganisation-design.md`, `docs/superpowers/plans/2026-11-19-reorganisation-phases-0-2.md`

**Interfaces:**
- Produces: la recette (scénarios numérotés 1 à 11) et le jeu de photos, utilisés aux tâches 15 et suivantes et par le plan des phases 3-4.

- [ ] **Step 1: Fabriquer les six photos.** Script jetable dans le dossier scratchpad (jamais commité), Python 3 sans dépendance : une image unie par photo (PNG écrit avec `zlib`), convertie en JPEG par `sips -s format jpeg`, puis segment APP1 Exif inséré avec `Make`, `Model` et `DateTimeOriginal`. Valeurs (heures murales, aucun fuseau) :

| Fichier | Make | Model | DateTimeOriginal | Rôle |
|---|---|---|---|---|
| A1.jpg | Dive Lens | Test Camera A | 2025:06:14 09:40:00 | plongée 101, 10 min (18,0 m) |
| A2.jpg | Dive Lens | Test Camera A | 2025:06:14 09:50:00 | plongée 101, 20 min (15,0 m) |
| A3.jpg | Dive Lens | Test Camera A | 2025:06:14 10:00:00 | plongée 101, 30 min (10,0 m) |
| B1.jpg | Dive Lens | Test Camera B | 2025:06:14 14:10:00 | plongée 102, 10 min (12,0 m) |
| B2.jpg | Dive Lens | Test Camera B | 2025:06:14 14:20:00 | plongée 102, 20 min |
| X1.jpg | Dive Lens | Test Camera A | 2025:06:14 12:00:00 | entre deux plongées |

Plongées du carnet fictif `core/fixtures/subsurface/fictional.xml` : 101 le 2025-06-14 de 09:30 à 10:12:30, site « Atlantis / North Reef / Coral Garden », GPS 12.5, -45.25 ; 102 le même jour de 14:00 à 14:35, site « Glacier Wall », GPS -64.125, -60.5.

- [ ] **Step 2: Vérifier les photos.**
Run: `for f in docs/contributing/lightroom-checks/*.jpg; do sips -g creation -g make -g model "$f"; done`
Expected: pour chaque fichier, la date, `make: Dive Lens` et le modèle du tableau. Chaque fichier fait moins de 20 Ko.

- [ ] **Step 3: Écrire `docs/contributing/lightroom-checks.md`.** Contenu :
  - But : ce que les tests automatiques ne peuvent pas prouver (Lightroom accepte nos écritures, un seul « Annuler »).
  - Préparation : un catalogue neuf (File › New Catalog, dans un dossier temporaire, jamais le catalogue habituel) ; importer le dossier `docs/contributing/lightroom-checks/` en **Copy** vers un dossier à côté de ce catalogue, jamais en « Add » (Lightroom pourrait écrire des métadonnées dans les fichiers du dépôt) ; Dive Lens réglé sur Subsurface, carnet `core/fixtures/subsurface/fictional.xml`, unités métriques, option altitude désactivée sauf au scénario 10.
  - Jouer la recette sur une version de développement (macOS) : le plug-in installé et celui du dépôt portent le même identifiant `com.querton.divelens`, et on ne s'appuie pas sur ce que Lightroom fait de deux plug-ins identiques. Étapes : quitter Lightroom ; déplacer `~/Library/Application Support/Adobe/Lightroom/Modules/DiveLens.lrplugin` hors du dossier Modules (par exemple sur le Bureau) ; construire l'app (`npm run tauri build -- --bundles app` dans `app/`) ; ouvrir Lightroom, File › Plug-in Manager › Add › `plugin/DiveLens.lrdevplugin` ; jouer la recette. Après : retirer le plug-in du dépôt dans le Plug-in Manager, quitter Lightroom, remettre `DiveLens.lrplugin` dans Modules.
  - Windows : la version installée suffit (référence et contrôle avec un installateur).
  - Les onze scénarios de la spec, chacun avec point de départ, action et ce qu'on doit voir. Valeurs attendues connues : A1 → `MaxDepth` 18.4 m, pays « Atlantis », ville « North Reef », lieu « Coral Garden », GPS 12°30'N 45°15'W ; profondeur propre à la photo 18,0 m pour A1, 15,0 m pour A2, 10,0 m pour A3. Scénario 6 : « jamais les valeurs propres à l'ancienne sélection sur d'autres photos : soit un refus sans rien écrire, soit l'app décrit à nouveau la sélection et écrit sur la nouvelle ». Scénario 8 : noter ce qu'un seul « Annuler » défait ; si un ensemble de collections vient d'être créé, il forme son propre pas d'annulation (écriture séparée dans le code actuel).
  - Comment noter le résultat dans une PR : système, version de Lightroom, version de Dive Lens, puis une ligne par scénario (OK / écart décrit en mots, sans donnée réelle).
- [ ] **Step 4: Ajouter dans `CONTRIBUTING.md`** (section « Trying the plug-in in Lightroom ») : « Before a release that touches the plug-in or the catalog writes, run the [Lightroom checks](docs/contributing/lightroom-checks.md). »
- [ ] **Step 5: Vérifier et commiter.**
Run: `cargo fmt --all --check`
Expected: aucune sortie.
```bash
git fetch origin
git checkout -b docs/lightroom-checks origin/main
git add docs/contributing/lightroom-checks.md docs/contributing/lightroom-checks/*.jpg CONTRIBUTING.md docs/superpowers/specs/2026-11-19-reorganisation-design.md docs/superpowers/plans/2026-11-19-reorganisation-phases-0-2.md
git commit -m "docs: add the lightroom checks and the reorganisation design"
```
- [ ] **Step 6: Demander au propriétaire** l'accord pour pousser et ouvrir la PR.

### Task 0b: Référence (manuel, propriétaire)

- [ ] **Step 1:** Le propriétaire joue les scénarios 1 à 10 sur macOS avec une version de développement construite depuis `origin/main` au commit de départ (le code de 2.1.1), selon la section « Jouer la recette sur une version de développement » : ce sont les mêmes conditions que le contrôle de la phase 2. Résultat et commit noté dans la PR de la tâche 0.
- [ ] **Step 2:** Scénario 11 par un testeur Windows avec l'installateur 2.1.1 publié, si disponible ; sinon noté « non joué ».
- [ ] **Step 3:** Si, sur Windows, un seul « Annuler » ne défait pas les deux écritures : arrêter le chantier et le signaler au propriétaire.
- [ ] **Step 4:** Fusion de la PR de la tâche 0, avec l'accord du propriétaire.

---

## Phase 1 — Déplacements purs

Règle de la phase : une différence découverte entre deux copies censées être identiques n'est pas réglée ici. Elle s'ajoute à l'annexe « Notes pour la phase 3 » de ce plan (commit dans la PR en cours).

### Task 1: Découper les tests du plug-in (n°16)

PR `test(plugin): split the plug-in tests by area` (tâches 1 et 2), branche `test/split-plugin-tests`.

**Files:**
- Modify: `app/src-tauri/tests/common/mod.rs` (ajout de `#![allow(dead_code)]` en tête : chaque fichier de tests compile tout `common`, et clippy `-D warnings` refuse un utilitaire non utilisé)
- Create: `app/src-tauri/tests/plugin_tools.rs`, `plugin_manifest.rs`, `plugin_tag_selection.rs`, `plugin_collections.rs`, `plugin_retagging.rs`
- Delete: `app/src-tauri/tests/plugin_lua.rs`

**Interfaces:**
- Produces: dans `tests/common/mod.rs`, en plus de `plugin_dir()` et `lua()` : `pub fn run(lua: &Lua, code: &str)`, `pub fn sample_dive() -> divelens_core::Dive`, `pub const WRITES_LUA: &str`. `plan` et `tag_writes` vont dans `plugin_tag_selection.rs` ; `Before`, `BEFORE_LUA`, `set_before`, `retag*`, `refile_writes` dans `plugin_retagging.rs`.

Répartition (lignes de `plugin_lua.rs`) :

| Fichier | Tests |
|---|---|
| plugin_tools.rs | runs_under_plain_lua_5_1 (16), json_encodes_what_rust_reads (24), json_decodes_what_rust_writes (52), json_rejects_malformed_input (79), framing_splits_lines_whatever_lightroom_hands_over (95), large_collections_request_is_framed_quickly (1716) |
| plugin_manifest.rs | plugin_keeps_its_identifier_and_fields (127), panel_shows_every_field_once_in_titled_sections (156), every_plugin_file_compiles_under_lua_5_1 (316), menu_items_open_existing_plugin_files (334), every_plugin_text_has_its_french_translation (391) avec `localized_texts` et `placeholders` |
| plugin_tag_selection.rs | tagging_plan_writes_dive_fields_and_clears_missing_ones (249), tagging_plan_location_only_and_bad_requests (279), tagging_reads_wall_clock_dates (302), photos_are_described_with_wall_clock_times (426), tagging_writes_gps_in_a_write_of_its_own (605), progress_tells_the_first_write_only (753), tag_selection_plans_per_photo_values_and_checks_the_selection (1578) |
| plugin_collections.rs | collections_plan_writes_dive_data_and_per_photo_values (450), collections_write_gps_and_altitude_in_a_write_of_their_own (658), collection_sets_are_found_by_path_and_listed (1688) |
| plugin_retagging.rs | le contenu de `mod retagging` (926-1576, 9 tests) au niveau du fichier, avec ses utilitaires ; les `super::` deviennent des imports de `common` |

- [ ] **Step 1: Compter les tests avant.**
Run: `cargo test -p divelens --test plugin_lua -- --list 2>/dev/null | grep -c ': test$'`
Expected: `30`
- [ ] **Step 2: Déplacer** selon le tableau, sans modifier le corps d'un seul test. Seuls changent les `use` et les chemins `super::`.
- [ ] **Step 3: Compter après.**
Run: `for t in plugin_tools plugin_manifest plugin_tag_selection plugin_collections plugin_retagging; do cargo test -p divelens --test $t -- --list 2>/dev/null | grep -c ': test$'; done`
Expected: `6`, `5`, `7`, `3`, `9` (total 30, les mêmes noms de tests ; `version.rs` n'est pas touché).
- [ ] **Step 4: Vérifier.**
Run: `cargo fmt --all --check && cargo clippy -p divelens --all-targets -- -D warnings && cargo test -p divelens`
Expected: tout passe, aucun avertissement.
- [ ] **Step 5: Commit** `test(plugin): split the plug-in tests by area`. Noter dans la future PR que `CLAUDE.md:24` cite encore `plugin_lua.rs`.

### Task 2: Test n°6 — vraies valeurs pour température basse, note et ville

**Files:**
- Modify: `app/src-tauri/tests/plugin_tag_selection.rs`

**Interfaces:**
- Consumes: `plan(&TagRequest) -> Result<Vec<String>, String>` (tâche 1). Ne pas modifier `sample_dive` (≈ 20 tests en dépendent) : construire la plongée dans le test.

- [ ] **Step 1: Écrire le test.**
```rust
#[test]
fn dive_values_reach_the_plugin_by_name_with_real_values() {
    use divelens_core::tags::{DiveTags, LocationTags, TagRequest, Units};
    let mut dive = common::sample_dive();
    dive.temp_low_c = Some(24.5);
    dive.rating = Some(4);
    dive.site.location = Some("North Reef".into());
    let lines = plan(&TagRequest {
        dive: Some(DiveTags::new(&dive, Units::default(), "Subsurface")),
        location: Some(LocationTags::new(&dive.site, None, None)),
        ..TagRequest::default()
    })
    .unwrap();
    assert!(lines.contains(&"plugin TempLow=24.5°C".to_string()), "{lines:?}");
    assert!(lines.contains(&"plugin Rating=★★★★".to_string()), "{lines:?}");
    assert!(lines.contains(&"raw city=North Reef".to_string()), "{lines:?}");
}
```
- [ ] **Step 2: Lancer.**
Run: `cargo test -p divelens --test plugin_tag_selection dive_values_reach_the_plugin`
Expected: PASS (le code actuel est correct ; ce test épingle les noms des deux côtés). S'il échoue, s'arrêter et le signaler : c'est un défaut existant, pas un effet du chantier.
- [ ] **Step 3: Vérifications de la tâche 1, step 4.**
- [ ] **Step 4: Commit** `test(plugin): check real dive values reach the plug-in`. Push et PR (tâches 1-2) avec l'accord du propriétaire.

### Task 3: Liste des valeurs relues déduite des listes existantes (n°3)

PR `refactor(plugin): move write decisions to the tested rules` (tâches 3 et 4), branche `refactor/plugin-write-decisions`.

**Files:**
- Modify: `plugin/DiveLens.lrdevplugin/Tagging.lua:97-110`
- Test: `app/src-tauri/tests/plugin_retagging.rs`

- [ ] **Step 1: Écrire le test épingle.**
```rust
#[test]
fn retag_fields_are_the_dive_identity_then_the_photo_values_then_the_marker() {
    let lua = common::lua();
    let fields: Vec<String> = lua
        .load(r#"return require("Tagging").RETAG_FIELDS"#)
        .eval()
        .unwrap();
    assert_eq!(
        fields,
        ["Source", "StartDate", "DiveId", "PhotoDepth", "PhotoTemp",
         "PhotoElapsed", "ClockOffset", "WrittenAltitude"]
    );
}
```
- [ ] **Step 2: Lancer.** Expected: PASS sur le code actuel.
- [ ] **Step 3: Construire `Tagging.RETAG_FIELDS`** après la définition de `Tagging.MARKER` (ligne 104) : les éléments de `DIVE_IDENTITY`, puis ceux de `PER_PHOTO`, puis `Tagging.MARKER`, dans cet ordre.
- [ ] **Step 4: Lancer.** Run: `cargo test -p divelens --test plugin_retagging` Expected: PASS.
- [ ] **Step 5: Commit** `refactor(plugin): derive the fields read back from the existing lists`.

### Task 4: Décisions des enveloppes d'écriture vers `Tagging.lua`

**Files:**
- Modify: `plugin/DiveLens.lrdevplugin/Tagging.lua` (nouvelles fonctions pures)
- Modify: `plugin/DiveLens.lrdevplugin/Link.lua:171-212, 285-348, 389-441, 503-605` (les enveloppes appellent ces fonctions)
- Test: `app/src-tauri/tests/plugin_tag_selection.rs`, `app/src-tauri/tests/plugin_collections.rs`

**Interfaces** (toutes dans `Tagging.lua`, sans `import` ni `Log` : la journalisation reste dans les enveloppes) :
- Produces:
  - `Tagging.photoId(photo) -> string` (déplacée de Link.lua:171-173, inchangée)
  - `Tagging.timeRange(list) -> table` (déplacée de Link.lua:188-212, inchangée ; « heures de prise de vue par appareil »)
  - `Tagging.findChild(list, name) -> object|nil` (déplacée de Link.lua:503-507, inchangée)
  - `Tagging.writeFailure(ok, status) -> reason|nil, detail` : `not ok` → `"error", tostring(status)` ; `status ~= "executed"` → `"busy"` ; sinon `nil`
  - `Tagging.isAltitudeClear(change) -> boolean` : `change.field == "gpsAltitude" and change.value == nil` (Link.lua:296)
  - `Tagging.secondWriteResult(ok, status, cleared, refused) -> table` : `{written=false, error=tostring(status)}` si `not ok` ; `{written=false}` si non exécuté ; sinon `{written=true, cleared=cleared, refused=refused}`. Le raccourci `#later == 0` reste dans l'enveloppe (aucune écriture lancée).
  - `Tagging.applyReply(id, outcome, done) -> message` avec `outcome = {ok, status, second}` où `second` vaut `false` quand la première écriture a échoué ou n'a pas été exécutée. Ordre : `writeFailure` d'abord, puis `second.written`, puis `done`.
  - `Tagging.selectionRefusal(perPhoto, described, photos, idOf) -> nil|"staleSelection"|"selectionChanged"` (Link.lua:403-407)
  - `Tagging.tagWrites(changes, photos, before, perPhoto, idOf) -> {writes = {{photo, changes}…}, later = {{photo, changes}…}, cleared = n, withValues = n}` (boucle de Link.lua:415-423, sans `write` ni `advance`)
  - `Tagging.resolvePhotos(groups, byId) -> total, found` : pose `p.photo = byId[p.id]` sur chaque photo prévue ; `total` compte toutes les photos demandées, trouvées ou non (Link.lua:556-563)
  - `Tagging.collectionWrites(groups, before, existing) -> {groups = {[gi] = {steps = {{photo|nil, changes|nil}…}, photos = {…}}}, later = {…}, results = {[gi] = {collection, count, created}}}` : un `step` par photo demandée (l'enveloppe appelle `advance` pour chacun, comme aujourd'hui) ; `created = not existing[g.collection]`, y compris quand `#photos == 0` (Link.lua:570-588)

- [ ] **Step 1: Écrire les tests** (chaque test exécute du Lua qui renvoie `Json.encode(résultat)` et compare à `serde_json::json!`) :
  - `apply_reply_says_why_nothing_or_part_was_written` : `applyReply(7, {ok=false, status="boom", second=false}, done)` → `{"type":"applyFailed","id":7,"reason":"error","detail":"boom"}` ; `{ok=true, status="aborted", second=false}` → `{"type":"applyFailed","id":7,"reason":"busy"}` ; `{ok=true, status="executed", second={written=false, error="locked"}}` → `{"type":"applyFailed","id":7,"reason":"gpsNotWritten","detail":"locked"}` ; `second={written=false}` → même chose sans `detail` ; `second={written=true}` → `done` tel quel (`{"type":"applied","id":7,"count":3}`).
  - `second_write_result_keeps_lightroom_refusals` : les trois formes de `secondWriteResult` ; `isAltitudeClear({field="gpsAltitude"})` vrai, `isAltitudeClear({field="gpsAltitude", value=-18})` faux, `isAltitudeClear({field="gps"})` faux.
  - `selection_refusal_checks_the_described_selection` : `perPhoto=nil` → `nil` ; `described=nil` → `"staleSelection"` ; `described.id ~= perPhoto.selection` → `"staleSelection"` ; mêmes ids mais photos différentes → `"selectionChanged"` ; mêmes photos → `nil`.
  - `tag_writes_keeps_photo_order_and_counts` : deux photos, changements de `plan(sample_dive)` avec GPS, valeurs propres pour la première seulement → `#writes == 2` dans l'ordre des photos, `withValues == 1`, `later` contient les deux photos (GPS), `cleared == 0` ; avec un `before` portant une autre plongée sur la seconde → `cleared == 1`.
  - `collection_writes_step_every_requested_photo` : deux groupes, une photo demandée absente du catalogue → `total` la compte, `steps` a une entrée pour elle (photo `nil`), `results[gi].count` ne la compte pas ; un nom présent dans `existing` → `created == false`, absent → `true`, y compris pour un groupe sans photo trouvée.
- [ ] **Step 2: Lancer.** Run: `cargo test -p divelens --test plugin_tag_selection --test plugin_collections` Expected: FAIL (fonctions absentes).
- [ ] **Step 3: Implémenter** les fonctions dans `Tagging.lua`, puis faire appeler ces fonctions par les enveloppes de `Link.lua`. Invariants : la ligne `local second = ok and status == "executed" and writeGpsLater(...)` reste avant toute réponse, puis `s.lastHeard`, puis la réponse ; les journaux restent identiques (la réponse `error`/`busy` passe par `fail`, `gpsNotWritten` garde son `Log.warn` propre, le succès garde `logSecondWrite` puis sa ligne `Log.info`) ; rien n'est envoyé ni ne cède la main entre les deux écritures. Mettre à jour les commentaires de `Tagging.lua:4, 258-259, 346` qui citent Link.lua.
- [ ] **Step 4: Lancer.** Run: `cargo test -p divelens` Expected: PASS, y compris tous les tests existants.
- [ ] **Step 5: Vérifications** (Global Constraints) puis commit `refactor(plugin): move write decisions to the tested rules`. La description de PR précise : planifier les écritures avant la boucle arrête une erreur de `photoWrites` avant toute écriture plutôt qu'en cours de route ; `photoWrites` est pure et renvoie toujours, donc aucun cas réel connu. Push et PR (tâches 3-4) avec l'accord du propriétaire.

### Task 5: Découper `Link.lua`

PR `refactor(plugin): split the link file by role`, branche `refactor/split-link`.

**Files:**
- Create: `plugin/DiveLens.lrdevplugin/Send.lua`, `Selection.lua`, `CatalogWrite.lua`, `ApplyTags.lua`, `ApplyCollections.lua`
- Modify: `plugin/DiveLens.lrdevplugin/Link.lua` (garde transport, jeton, aiguillage, `PROTOCOL`)
- Test: `app/src-tauri/tests/plugin_manifest.rs`

`CatalogWrite.lua` est créé dès maintenant pour héberger les utilitaires d'écriture déjà en un seul exemplaire ; l'enveloppe unique « deux écritures, un seul Annuler » y arrivera en phase 3. *(Choix du constructeur.)*

**Interfaces:**
- Produces:
  - `Send.toApp(s, message)` (Link.lua:126-134, inchangée)
  - `Selection.push(s, force)`, `Selection.sendThumbnails(s)`, `Selection.describe(s, message)`, `Selection.describePhotos(photos)`, `Selection.sendPhotos(s, id, list)` ; constantes `THUMBNAILS`, `THUMBNAIL_SIZE`, `THUMBNAILS_PER_TICK`, `PHOTOS_PER_MESSAGE` et fonctions locales `selection`, `describedSelection`, `requestThumbnails` (Link.lua:48-50, 141-169, 175-186, 214-255, 443-469)
  - `CatalogWrite.WRITE_TIMEOUT`, `CatalogWrite.PROGRESS_EVERY`, `CatalogWrite.formatDate(iso)`, `CatalogWrite.write(photo, changes)`, `CatalogWrite.writeGpsLater(catalog, undoName, later)`, `CatalogWrite.advance(s, id, total, progress)`, `CatalogWrite.photosBefore(catalog, photos)`, `CatalogWrite.logSecondWrite(later, second)` ; locales `writeChange`, `writeGps` (Link.lua:259-376)
  - `ApplyTags.run(s, message)` (Link.lua:378-441)
  - `ApplyCollections.find(s, message)`, `ApplyCollections.run(s, message)`, `ApplyCollections.listSets(s, message)` (Link.lua:473-501, 509-611)
  - Dépendances autorisées : Link → Send, Selection, ApplyTags, ApplyCollections ; Selection, ApplyTags, ApplyCollections → Send, CatalogWrite, Tagging, Json, Log ; CatalogWrite → Send, Tagging, Log. Rien ne requiert Link.

- [ ] **Step 1: Écrire le test de chargement.** `plugin_modules_load_without_lightroom` : dans `common::lua()`, définir un `stub` Lua qui accepte tout (une table dont `__index` renvoie `stub` et dont `__call` renvoie `stub`, pour que `import "LrLogger"("DiveLens")` ou `LrPathUtils.child(...)` au chargement ne lèvent pas d'erreur), puis `import = function() return stub end`, `_PLUGIN = {path = "/tmp", id = "com.querton.divelens"}`, `WIN_ENV = false`, puis `setmetatable(_G, {__index = function(_, k) error("undefined global " .. k, 2) end})`, puis `require` chacun de `Send`, `Selection`, `CatalogWrite`, `ApplyTags`, `ApplyCollections`, `Link`, et vérifier que chaque fonction listée dans Interfaces est de type `function` (et `WRITE_TIMEOUT`, `PROGRESS_EVERY` des nombres). Écrire aussi, dès maintenant, `plugin_globals_are_known` (décrit au step 4) et le faire passer sur le code actuel.
- [ ] **Step 2: Lancer.** Expected: FAIL sur `Send` (module absent). Vérifier d'abord, en ne gardant que `require "Link"`, que le test passe sur le code actuel : si `Link.lua` lit un global non prévu au chargement, ajouter ce global aux stubs avant de continuer.
- [ ] **Step 3: Déplacer** selon Interfaces, sans changer une ligne de logique. `Link.handle` appelle `ApplyTags.run`, `ApplyCollections.find/run/listSets`, `Selection.describe` ; la boucle `run` appelle `Selection.push` et `Selection.sendThumbnails`. Les champs de `s` restent ceux d'aujourd'hui (`closeSockets` et `Link.start` continuent de les réinitialiser). Découper l'en-tête de Link.lua (lignes 1-23) entre les fichiers.
- [ ] **Step 4: Contrôle des globales, corps de fonctions compris.** Test `plugin_globals_are_known` dans `plugin_manifest.rs`, écrit au step 1, avant le déplacement, et qui passe sur le code actuel ; il doit encore passer ici : pour chaque `.lua` du plug-in, compiler sous Lua 5.1, obtenir le code compilé par `string.dump`, parcourir récursivement les prototypes de fonctions (en-tête Lua 5.1 : entiers et `size_t` de la taille indiquée par l'en-tête, constantes, instructions de 32 bits, opcode = 6 bits de poids faible, `GETGLOBAL` = 5, index de constante = champ `Bx`) et relever les noms lus en global. Assert : chaque nom appartient à la liste autorisée — bibliothèques Lua 5.1 standard (`assert`, `error`, `ipairs`, `pairs`, `pcall`, `require`, `select`, `setmetatable`, `getmetatable`, `tonumber`, `tostring`, `type`, `unpack`, `next`, `rawget`, `rawset`, `math`, `string`, `table`, `os`, `io`, `_G`, `_VERSION`), globales Lightroom (`import`, `LOC`, `_PLUGIN`, `WIN_ENV`, `MAC_ENV`), et les globales que le plug-in utilise déjà aujourd'hui (relevées au premier passage et listées une par une dans le test, par exemple `DiveLensLink` s'il est lu ainsi). Après le déplacement, toute globale nouvelle (`send`, `write`, `WRITE_TIMEOUT`…) fait échouer le test.
- [ ] **Step 5: Lancer.** Run: `cargo test -p divelens` Expected: PASS, dont `the_plugin_speaks_the_same_protocol` et `every_plugin_text_has_its_french_translation` (les clés d'annulation déplacées gardent le même texte anglais).
- [ ] **Step 6: Vérifications** puis commit `refactor(plugin): split the link file by role`. Push et PR avec l'accord du propriétaire. La PR signale que `CLAUDE.md:23` cite encore `Link.lua` comme tout le côté plug-in.
- [ ] **Step 7: Recette du plug-in seul (recommandée, décision du propriétaire).** Avant de fusionner cette PR, le propriétaire rejoue les scénarios 1 à 9 depuis cette branche, dans les conditions de la référence. Les tâches 3 à 5 sont les seuls changements du plug-in : une différence ici n'a qu'une cause possible, au lieu de sept PR en phase 2.

### Task 6: Filet du moteur — toutes les commandes appelées sont enregistrées

PR `refactor(app): split the engine, settings and updates` (tâches 6 à 9), branche `refactor/engine-settings`.

**Files:**
- Create: `app/src-tauri/tests/commands.rs`

- [ ] **Step 1: Écrire le test.** `every_invoked_command_is_registered` : lire `app/src/api.ts` (via `CARGO_MANIFEST_DIR`/`../src/api.ts`), extraire les noms de `invoke<…>("nom"` ; lire `app/src-tauri/src/lib.rs`, extraire le bloc `generate_handler![…]` et le dernier segment de chaque chemin (`tag::tag_selection` → `tag_selection`). Assert : les deux ensembles sont égaux et comptent 27 noms.
- [ ] **Step 2: Lancer.** Run: `cargo test -p divelens --test commands` Expected: PASS sur le code actuel.
- [ ] **Step 3: Commit** `test(app): check every command the screens call is registered`.

### Task 7: Lecture/écriture sûre des petits fichiers en un exemplaire

**Files:**
- Create: `app/src-tauri/src/files.rs`
- Modify: `app/src-tauri/src/settings.rs:308-324`, `offsets.rs:34-49`, `update.rs:202-214`, `lib.rs:222-224` (`config_dir` déplacée dans `files.rs`)

**Interfaces:**
- Produces: `pub(crate) fn read_json<T: serde::de::DeserializeOwned + Default>(path: &Path) -> T`, `pub(crate) fn write_json<T: serde::Serialize>(dir: &Path, name: &str, value: &T) -> io::Result<()>`, `pub(crate) fn config_dir(app: &tauri::AppHandle) -> PathBuf` (même `expect("no config folder")`).
- Comportement exact des trois copies actuelles : lecture `fs::read_to_string(path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()` ; écriture `create_dir_all(dir)?`, puis `dir.join(format!("{name}.tmp"))`, `fs::write(&tmp, serde_json::to_string_pretty(value)?)?`, `fs::rename(tmp, dir.join(name))`. Le verrou `LOCK` des réglages et l'estampille de version (réglages à la lecture, décalages à l'écriture) restent chez l'appelant. Aucun verrou ajouté ni retiré.

- [ ] **Step 1: Tests** dans `files.rs` : `missing_or_invalid_file_reads_as_default` (fichier absent, puis contenu `not json` → `Default`) ; `write_creates_the_folder_and_leaves_no_temporary_file` (dossier inexistant créé, fichier relu égal, aucun `{name}.tmp` restant) ; `written_json_is_pretty` (contenu égal à `serde_json::to_string_pretty`).
- [ ] **Step 2: Lancer.** Run: `cargo test -p divelens files::` Expected: FAIL (module absent).
- [ ] **Step 3: Implémenter `files.rs`**, puis remplacer les trois copies et `config_dir` (16 appels).
- [ ] **Step 4: Lancer.** Run: `cargo test -p divelens` Expected: PASS, dont les tests existants des réglages, décalages et mises à jour.
- [ ] **Step 5: Commit** `refactor(app): read and write small files in one place`.

### Task 8: Réglages et carnet (n°10, partie 1)

**Files:**
- Create: `app/src-tauri/src/settings/file.rs` (tout l'actuel `settings.rs`), `app/src-tauri/src/settings/units.rs` (tout l'actuel `units.rs`)
- Create (nouveau contenu): `app/src-tauri/src/settings.rs`
- Modify: `app/src-tauri/src/dive_log.rs`, `app/src-tauri/src/lib.rs`, `CONTRIBUTING.md:189`
- Delete: `app/src-tauri/src/units.rs`

**Interfaces:**
- Produces:
  - `settings.rs` : `mod file; mod units;`, `pub(crate) use file::{Settings, LogLevel, Legacy, DepthChoice, ImportError, …}` (tout ce que `crate::settings::…` désigne aujourd'hui : dive_log.rs:16, 181 ; applog.rs:18 ; lightroom.rs:30 ; lib.rs:33, 365, 1543), `pub(crate) use units::UiUnits` ; les commandes `dive_apps`, `app_info`, `get_settings`, `update_settings`, `export_settings`, `check_settings_file`, `import_settings` ; `LogFile`, `AppInfo`, `ImportCheck`, `import_check`, `ChangeError`, `change_settings`, `change_unless_writing`, `source_units`, `units_in_use` (lib.rs:35-196, 264-278) ; le test `settings_cannot_switch_the_dive_log_during_a_write` et ses utilitaires (lib.rs:1608-1671).
  - `dive_log.rs` reçoit : `DiveLog`, `LoadError`, `login_name`, `standard_path`, `source_info`, `source_info_of`, `open`, `load_dives`, `ProfileError` + `From<LogChanged>`, `dive_profile`, `set_source_path` (lib.rs:198-262, 280-361).
- Points de compilation connus : `include_str!` de `settings.rs:810` devient `../../tests/fixtures/fictional-settings-export-v3.json` ; les `cfg_attr(…, allow(dead_code))` de `units.rs:63, 76, 87, 94, 110` suivent leurs éléments ; code Windows de `units.rs:50-54, 118-140` vérifié par la CI Windows.

- [ ] **Step 1: Déplacer**, sans changer une ligne de logique ; mettre à jour `generate_handler!` avec les chemins `settings::…` et `dive_log::…`.
- [ ] **Step 2: `CONTRIBUTING.md:189`** : `CURRENT_VERSION` est désormais dans `app/src-tauri/src/settings/file.rs`.
- [ ] **Step 3: Lancer.** Run: `cargo fmt --all --check && cargo clippy -p divelens --all-targets -- -D warnings && cargo test -p divelens` Expected: PASS, dont `every_invoked_command_is_registered`.
- [ ] **Step 4: Commit** `refactor(app): move settings and dive-log commands to their modules`.

### Task 9: Mises à jour (n°10, partie 2)

**Files:**
- Create: `app/src-tauri/src/update/install.rs`
- Modify: `app/src-tauri/src/update.rs`, `app/src-tauri/src/lib.rs`

**Interfaces:**
- Produces:
  - `update/install.rs` : `PLUGIN_ID`, `STAGING`, `PREVIOUS`, `INSTALLER_DIR`, `Copy` et ses méthodes, `check_installable`, `container`, `is_plugin_folder`, `can_write_in`, `writable`, `plugin_version`, `AFTER_UPDATE`, `clean_up`, `InstallError`, `io`, `fresh_staging`, `check_package`, `swap`, `install`, `install_from`, `relaunch`, `original_path` (update.rs:35-154, 295-615) et leurs tests (update.rs:690-849) ; utilitaires de test `temp_dir` et `plugin` (621-643) partagés via un module de test `pub(super)`.
  - `update.rs` garde la vérification et l'état (30-45, 156-293), reçoit `Updates`, `update_status_of`, `set_phase`, `update_status`, `find_update`, `check_update`, `dismiss_update`, `install_update` (lib.rs:1024-1272), et ses tests (649-688, 851-864). `pub(crate) use install::Copy` si nécessaire pour les chemins existants.
- Les commandes déplacées appellent `app.state::<Updates>()` (pas d'import `tauri::State` non qualifié : `update::State` existe). Les `cfg_attr` de update.rs:338, 353, 381 suivent leurs éléments. Code Windows 478-519 vérifié par la CI Windows.

- [ ] **Step 1: Déplacer**, `generate_handler!` en `update::…`.
- [ ] **Step 2: Lancer.** Même commande que la tâche 8. Expected: PASS.
- [ ] **Step 3: Commit** `refactor(app): move update commands next to the update logic`. Push et PR (tâches 6-9) avec l'accord du propriétaire ; la PR rappelle que la CI Windows est le seul juge du code Windows déplacé.

### Task 10: Partie commune d'« Appliquer » et « Créer les collections » (n°10, partie 3)

PR `refactor(app): split the engine by feature` (tâches 10 à 12), branche `refactor/engine-features`.

**Files:**
- Create: `app/src-tauri/src/apply.rs`, `apply/errors.rs`, `apply/searches.rs`, `apply/offsets.rs` (tout l'actuel `offsets.rs`), `collections/plan.rs` (l'actuel `collections.rs` hors éléments partagés)
- Create (nouveau contenu): `app/src-tauri/src/collections.rs`
- Delete: `app/src-tauri/src/offsets.rs`
- Modify: `app/src-tauri/src/lib.rs`, `app/src-tauri/src/selection.rs` (imports), `CONTRIBUTING.md:189` (chemin de `offsets.rs`)

**Interfaces:**
- Produces:
  - `apply.rs` : `mod errors; mod searches; mod offsets;`, `send`, `ask<T>` (lib.rs:781-804).
  - `apply/errors.rs` : `CollectionsError` + `From<LogChanged>` + `From<ApplyOutcome>` (lib.rs:737-779), nom inchangé (format envoyé à l'interface identique ; les commandes de Tag l'utilisent aussi).
  - `apply/searches.rs` : `Camera`, `Photo`, `Shots`, `offset_of`, `suggest_for` (devenues `pub(crate)`), `CameraCount`, `Searches<S>` **sans paramètre par défaut** (les usages écrivent `Searches<Session>` : lib.rs:712, 1550 et les tests), `keep_search` + ses deux tests (lib.rs:715-735, 1548-1606).
  - `collections.rs` : `mod plan;`, `CollectionsState`, `find_collection_photos`, `collections_preview`, `suggest_offsets`, `CollectionsResult`, `apply_collections`, `collection_sets` (lib.rs:709-713, 806-1022).
  - `collections/plan.rs` : `DAYS_AROUND`, `Session`, `Period`, `Choices`, `DiveCount`, `Preview`, `search_days`, `impl Session`, tests (collections.rs:22, 145-200, 212-224, 283-917 ; les tests de `Searches` 833-903 restent ici car ils utilisent la fixture `session()`).

- [ ] **Step 1: Déplacer**, `generate_handler!` en `collections::…`.
- [ ] **Step 2: `CONTRIBUTING.md:189`** : `CURRENT_VERSION` des décalages est désormais dans `app/src-tauri/src/apply/offsets.rs`.
- [ ] **Step 3: Lancer.** Même commande que la tâche 8. Expected: PASS.
- [ ] **Step 4: Commit** `refactor(app): move shared apply code and collections commands`.

### Task 11: « Tagger la sélection » et ses règles testées (n°10, partie 4)

**Files:**
- Create: `app/src-tauri/src/tag.rs`, `tag/rules.rs`, `tag/selection.rs` (tout l'actuel `selection.rs`)
- Delete: `app/src-tauri/src/selection.rs`
- Modify: `app/src-tauri/src/lib.rs`

**Interfaces:**
- Produces:
  - `tag.rs` : `mod rules; mod selection;`, `TagState`, `TagChoice`, `LocationChoice` (devenus `pub(crate)`), `TagError` + `From<LogChanged>`, `tag_request` (qui n'enchaîne plus que les règles), `tag_selection`, `find_selection_photos`, `tag_preview`, `suggest_tag_offsets` (lib.rs:409-707).
  - `tag/rules.rs` :
    - `fn check_gps(lat: Option<f64>, lon: Option<f64>) -> Result<(), TagError>`
    - `fn find_dive(dives: Vec<Dive>, id: &str) -> Result<Dive, TagError>`
    - `fn find_site(sites: Vec<Site>, site_id: &str) -> Result<Site, TagError>`
    - `fn location(choice: Option<&LocationChoice>, dive: Option<&Dive>, sites: impl FnOnce() -> Result<Vec<Site>, divelens_core::Error>) -> Result<Option<LocationTags>, TagError>`
    - `fn base_request(dive: Option<&Dive>, location: Option<LocationTags>, units: Units, source: &str) -> Result<TagRequest, TagError>`
    - `fn described(sel: &Searches<Selection>, search: Option<u64>, dive: &Dive, offsets: &HashMap<String, i32>) -> Result<Option<(u64, Vec<InDive>)>, TagError>`
    - `fn apply_per_photo(req: &mut TagRequest, outcome: Result<PerPhoto, Refusal>, depth_to_altitude: impl FnOnce() -> bool) -> Result<(BTreeMap<String, i32>, Option<divelens_core::Error>), TagError>`
    - `fn remember_offsets(outcome: &ApplyOutcome, applied: &BTreeMap<String, i32>) -> bool`
  - Ordre des erreurs inchangé (lib.rs:484-567) : `open`, unités, plongée (lue seulement si `dive_id`), `NothingChosen` pour le site de la plongée sans plongée, `BadGps` **avant** la lecture des sites, sites, `NothingChosen`, valeurs par photo ; et tout cela avant le contrôle `NotConnected`. Les lectures restent paresseuses (fermetures) pour que la même erreur gagne.
  - Le réglage `depth_to_altitude` n'est lu que dans la branche « valeurs par photo » ; l'avertissement « profil illisible » reste journalisé à l'identique par l'appelant à partir de l'`Option<Error>` renvoyée.

- [ ] **Step 1: Tests** dans `tag/rules.rs`, écrits contre les fonctions extraites mais vérifiant le comportement actuel :
  - `half_a_gps_position_is_refused` : `check_gps(Some(12.5), None)` et `(None, Some(-45.25))` → `BadGps` ; `(Some(91.0), Some(0.0))` → `BadGps` ; `(Some(12.5), Some(-45.25))` et `(None, None)` → `Ok`.
  - `unknown_dive_or_site_is_refused` : `find_dive(vec![], "x")` → `UnknownDive` ; `find_site(vec![], "x")` → `UnknownSite` ; un id présent → l'élément, le premier s'il y en a deux.
  - `bad_gps_wins_before_the_sites_are_read` : `location(Some(&Site{ lat: Some(1.0), lon: None, … }), …, || panic!("sites read"))` → `BadGps` sans panique.
  - `the_dive_site_needs_a_dive` : `location(Some(&DiveSite), None, …)` → `NothingChosen` ; `base_request(None, None, …)` → `NothingChosen`.
  - `an_unknown_description_is_no_photos` : `described` avec `Some(n)` absent → `NoPhotos` ; `None` → `Ok(None)`.
  - `altitude_setting_is_read_only_for_per_photo_values` : `apply_per_photo` avec `Err(Refusal::Describe)` → `NoPhotos` et la fermeture n'est pas appelée ; `Err(Refusal::Profile(_))` → `Source` ; `Err(Refusal::Unreadable(e))` → `Ok((vide, Some(e)))`.
  - `offsets_are_remembered_only_when_something_was_written` : appliqué + décalages non vides → `true` ; appliqué + vide → `false` ; échec sans écriture → `false` ; GPS non écrit (`half_written`) + non vide → `true`.
  Les variantes exactes de `TagError` et `Refusal` sont celles de lib.rs:442-468 et selection.rs:249-258 ; utiliser `matches!` quand le type n'implémente pas `PartialEq`.
- [ ] **Step 2: Lancer.** Run: `cargo test -p divelens tag::rules` Expected: FAIL (module absent).
- [ ] **Step 3: Extraire** les règles et déplacer les commandes ; `generate_handler!` en `tag::…`.
- [ ] **Step 4: Lancer.** Même commande que la tâche 8. Expected: PASS.
- [ ] **Step 5: Commit** `refactor(app): move tag selection to its module and test its rules`.

### Task 12: Lien avec Lightroom, `lib.rs` réduit au démarrage (n°10, partie 5)

**Files:**
- Create: `app/src-tauri/src/lightroom/events.rs`, `app/src-tauri/src/lightroom/commands.rs`
- Modify: `app/src-tauri/src/lightroom.rs` (protocole inchangé, `PROTOCOL` y reste ; ajoute `pub(crate) mod events; pub(crate) mod commands;`), `app/src-tauri/src/lib.rs`

**Interfaces:**
- Produces:
  - `lightroom/events.rs` : `Lightroom` (état), `SelectionPreview`, `show_status`, `show_screen`, `bring_to_front`, `lightroom_link` avec sa réaction aux événements (lib.rs:375-387, 1291-1428).
  - `lightroom/commands.rs` : `send_log_level`, `selection_preview`, `lightroom_status`, `lightroom_writing`, `initial_screen`, `screen_arg` et le test `reads_screen_argument` (lib.rs:363-373, 389-407, 1274-1289, 1673-1695).
  - `lib.rs` : déclarations de modules et `pub fn run()` seulement (appelée par `main.rs:5`) ; `generate_handler!` liste les 27 commandes par chemin de module.

- [ ] **Step 1: Déplacer.**
- [ ] **Step 2: Lancer.** Même commande que la tâche 8. Expected: PASS ; `lib.rs` ne contient plus que les `mod`, `run()` et ses imports.
- [ ] **Step 3: Mettre à jour les commentaires périmés** : `app/src/types.ts:130, 156, 166, 221, 288` (fichiers Rust cités), `tag/selection.rs:5`, `collections/plan.rs:3`, et les liens de doc venus de l'ancien `collections.rs:166` et `:233`.
- [ ] **Step 4: Commit** `refactor(app): move the lightroom link out of lib.rs`. Push et PR (tâches 10-12) avec l'accord du propriétaire.

### Task 13: Interface rangée par fonctionnalité (n°12, partie 1)

PR `refactor(app): organise the front end by feature` (tâches 13 et 14), branche `refactor/front-end-features`.

**Files:** déplacements dans `app/src/` (chaque test suit son fichier, y compris ceux qui font `await import("./x")`) :

| Actuel | Cible |
|---|---|
| screens/DivesScreen.tsx | collections/DivesStep.tsx (composant renommé `DivesStep`) |
| screens/CollectionsScreen.tsx, collectionPlace.ts (+ test) | collections/ |
| components/Steps.tsx | ui/ |
| screens/TagScreen.tsx, components/SitePicker.tsx, covers.ts (+ test), sites.ts (+ test) | tag/ |
| screens/SettingsScreen.tsx, screens/SourceError.tsx, backup.ts (+ test) | settings/ |
| components/UpdateBanner.tsx, updates.ts (+ test) | updates/ |
| components/DiveList.tsx, FilterBar.tsx, DiveDetail.tsx, DiveMap.tsx, ProfileChart.tsx, filters.ts (+ test), profiles.ts (+ test) | dives/ |
| components/CameraCard.tsx, cameras.ts (+ test) | apply/ |
| components/Icon.tsx, Stars.tsx, Sidebar.tsx, LogNotice.tsx, format.ts (+ test) | ui/ |
| main.tsx, App.tsx, App.css, api.ts, types.ts, i18n.ts, vite-env.d.ts, diveLog.ts (+ test), sourceText.ts (+ test) | restent à la racine (utilisés partout ; `api.ts` dépend de `diveLog.ts`) |

`DiveMap.tsx` et `DiveDetail.tsx` vont ensemble (import paresseux `./DiveMap`). `collectionPlace.ts` est déplacé entier, y compris `unitsInUse` (sa place sera revue en phase 4 avec le n°4).

- [ ] **Step 1: Compter les tests avant.** Run (dans `app/`): `npm test 2>&1 | grep -E "Test Files|Tests "` Expected: noter les deux nombres.
- [ ] **Step 2: Déplacer** avec `git mv` ; mettre à jour les imports relatifs (aucun alias n'existe ; `noUnusedLocals` refuse un import devenu inutile).
- [ ] **Step 3: Lancer.** Run (dans `app/`): `npm test && npm run build` Expected: mêmes nombres de fichiers de test et de tests qu'au step 1, build sans erreur. Puis `cargo test -p divelens` (le moteur embarque `app/dist`) : PASS.
- [ ] **Step 4: Commit** `refactor(app): organise the front end by feature`.

### Task 14: Règles « quand Appliquer est permis » sorties et testées ; liste des plongées partagée (n°12, partie 2)

**Files:**
- Create: `app/src/tag/rules.ts`, `app/src/tag/rules.test.ts`, `app/src/collections/rules.ts`, `app/src/collections/rules.test.ts`, `app/src/dives/useDiveBrowser.ts`, `app/src/dives/EmptyList.tsx`
- Modify: `app/src/tag/TagScreen.tsx`, `app/src/collections/CollectionsScreen.tsx`, `app/src/collections/DivesStep.tsx`

**Interfaces:**
- Produces (fichiers `rules.ts` en TypeScript pur, sans import React à l'exécution ; les tests utilisent `setLanguage("en")` dans `beforeEach`) :
  - `tag/rules.ts` :
    - `tagBlocker(s: { connected: boolean; selected: number; withDive: boolean; withPlace: boolean; chosen: boolean; needsValues: boolean; readFailure: string | null; profileLoading: boolean; current: boolean; freshCounts: boolean; usesDiveSite: boolean; pickedSite: boolean; gpsError: boolean }): string | null` — reprend TagScreen.tsx:307-317 dans le même ordre (déconnecté, sélectionner dans Lightroom, choisir quelque chose, choisir une plongée, échec de lecture, profil en chargement, lecture des valeurs par photo, choisir un site, GPS invalide).
    - `tagSummary(blocker: string | null, chosen: Dive | null, placeName: string | null | undefined): string` — TagScreen.tsx:305 et 319-325, y compris le cas où `placeName` est nul.
    - `canApplyTag(blocker: string | null, writing: boolean): boolean` — `blocker === null && !writing`.
    - `selectionChanged(e: ApplyOutcome | TagError): boolean` — déplacée telle quelle (TagScreen.tsx:92-95).
  - `collections/rules.ts` :
    - `collectionsBlocker(i: { connected: boolean; search: Search; parentMissing: boolean; parent: string[]; preview: CollectionsPreview | null }): string | null` — CollectionsScreen.tsx:368-373 dans le même ordre ; le type `Search` (lignes 30-34) déménage ici.
    - `canApplyCollections(blocker: string | null, preview: CollectionsPreview | null, writing: boolean): boolean` — `blocker === null && preview !== null && !writing` (ligne 593 ; la condition « aperçu absent » est cachée et doit rester distincte du bloqueur).
    - `barText(blocker: string | null, place: string): string` — `blocker ?? t("coll.writes", { place })` (ligne 573).
    - `writesNote(units: Units, depthToAltitude: boolean): string` — lignes 575-576.
  - `dives/useDiveBrowser.ts` : `useDiveBrowser(dives: Dive[], sort: Sort): { filters; setFilters; shown: Dive[]; focusedId: string | null; setFocusedId; focused: Dive | null }` — reprend à l'identique DivesStep (ex-DivesScreen) 39-55 et TagScreen 146-156 et 290 (`useDeferredValue`, `sortDives(applyFilters(dives, deferredFilters, index), sort)`, focus qui suit la liste visible).
  - `dives/EmptyList.tsx` : l'état vide identique de DivesStep 131-140 et TagScreen 428-437.
  - Ce qui reste propre à chaque écran : le mode de `DiveList` (`multi` / `single` + `mark`), la classe du conteneur (`dives-body` / `dives-body tag-dives`), le sélecteur de tri (étape 1 seulement), le montage conditionnel `withDive` et l'effet « plongée qui couvre » de Tag (266-268), qui doit rester **après** l'appel de `useDiveBrowser`.

- [ ] **Step 1: Écrire les tests.**
  - `tag/rules.test.ts` : un cas par bloqueur, chacun avec toutes les conditions précédentes satisfaites ; « déconnecté » l'emporte sur tout ; `canApplyTag(null, true) === false` ; `tagSummary` quand le site de la plongée n'a pas de nom (renvoie le résumé « plongée seule », comme aujourd'hui).
  - `collections/rules.test.ts` : un cas par bloqueur dans l'ordre ; `canApplyCollections(null, null, false) === false` (épingle la condition cachée) ; `barText(null, "Dive Lens")` égal à `t("coll.writes", { place: "Dive Lens" })` ; `writesNote` pour métrique/impérial × altitude oui/non.
- [ ] **Step 2: Lancer.** Run (dans `app/`): `npm test` Expected: FAIL (modules absents).
- [ ] **Step 3: Extraire** les règles et le hook ; les écrans appellent ces fonctions là où le code se trouvait.
- [ ] **Step 4: Lancer.** Run (dans `app/`): `npm test && npm run build`, puis `cargo test -p divelens` Expected: PASS.
- [ ] **Step 5: Commit** `refactor(app): test when apply is allowed and share the dive list`. Push et PR (tâches 13-14) avec l'accord du propriétaire.

---

## Phase 2 — Point de contrôle

### Task 15: Recette rejouée et comparée à la référence

- [ ] **Step 1:** Toutes les PR de la phase 1 sont fusionnées ; CI verte sur `main`.
- [ ] **Step 2: Build macOS** depuis `main` à jour. Run (dans `app/`): `npm ci && npm run tauri build -- --bundles app` Expected: `target/release/bundle/macos/Dive Lens.app` ; le plug-in du dépôt (`plugin/DiveLens.lrdevplugin`) ajouté dans Lightroom lance cette app.
- [ ] **Step 3: Installateur Windows d'essai** : branche jetable `test/checkpoint-phase-2` depuis `main`, puis, avec l'accord du propriétaire, lancement manuel du workflow Release sur cette branche (aucune publication ; l'installateur est un artefact de la CI).
- [ ] **Step 4: Le propriétaire rejoue les scénarios 1 à 10** sur macOS ; un testeur Windows rejoue le 11. Chaque résultat est comparé à la référence de la tâche 0b : toute différence arrête le chantier jusqu'à explication.
- [ ] **Step 5: Contrôle des écrans à la main** (ce que Vitest ne peut pas voir) : dans « Tagger la sélection », la plongée qui couvre la sélection reçoit le focus ; les filtres fonctionnent à l'étape 1 de « Créer les collections » et dans « Tagger la sélection » ; changer les unités dans Réglages change la liste ; un carnet introuvable affiche l'écran d'erreur du carnet ; la bannière de mise à jour s'affiche comme avant.
- [ ] **Step 6:** Supprimer la branche `test/checkpoint-phase-2` (avec l'accord du propriétaire). Écrire le plan des phases 3 et 4 à partir de l'annexe ci-dessous.

---

## Annexe — Notes pour la phase 3

Différences et défauts trouvés en cartographiant le code le 2026-11-19. Aucun n'est corrigé en phase 1.

**Plug-in, entre les deux enveloppes d'écriture**
1. « Tagger la sélection » ne lit l'état des photos que si la requête écrit une plongée ; « Créer les collections » le lit toujours (chaque groupe a une plongée, même résultat en pratique).
2. Le total de « Créer les collections » compte les photos demandées, y compris celles introuvables (barre de progression et journal).
3. `created` est vrai pour un groupe sans photo trouvée (aucune collection créée), et pour un second groupe du même nouveau nom.
4. « Créer les collections » ignore le compte des valeurs d'une ancienne plongée remplacées.
5. « Créer les collections » fait une troisième écriture (l'ensemble de collections), sous son propre nom d'annulation.
6. Le message `apply` (Tag) n'est pas protégé : défaut n°2.

**Moteur**
7. L'option « altitude depuis la profondeur » n'est lue, pour Tag, que dans la branche des valeurs par photo ; « Créer les collections » l'applique toujours.
8. Les décalages d'horloge sont mémorisés par Tag seulement si quelque chose a été écrit et que des décalages existent ; par « Créer les collections » à chaque réponse correcte ou à moitié écrite.
9. Les commandes de Tag renvoient `CollectionsError` pour la recherche et l'aperçu, `TagError` pour l'écriture.
10. Deux réactions aux verrous empoisonnés : se rétablir (réglages, carnet, journal) ou paniquer (toutes les commandes de lib.rs, dont le fil du lien) : n°13, phase 4.

**Interface**
11. Trois traducteurs d'erreurs : `TagScreen.failure`, `TagScreen.describeFailure`, `CollectionsScreen.failure`. Code brut pour `nothingChosen`, `logChanged`, `source`, `badGps`, `unknownDive`, `unknownSite` dans Tag : n°1.
12. Tag reste sur « Lecture de la sélection… » quand `describeFailure` renvoie `null` (`noPhotos`, `logChanged`).
13. Choix du lieu dans Tag : avec `placeKind` encore à « site de la plongée » mais aucun site utilisable, un GPS saisi invalide n'est ni signalé ni bloquant, et il est abandonné en silence à l'écriture.
14. « Créer les collections » : bouton Appliquer grisé sans explication quand l'aperçu a échoué.
15. Les réglages de chaque appareil (Suggérer, Ignorer, décalage) restent actifs pendant une écriture ; « Tout suggérer » est désactivé.
16. Les messages d'échec de recherche de « Créer les collections » reprennent le vocabulaire d'écriture (« Rien n'a été écrit… », « Annuler ») : point ouvert depuis le 2026-11-14.
17. Une sauvegarde de réglage qui échoue laisse la nouvelle valeur à l'écran (unités, altitude, emplacement, langue, tri) : n°4, phase 4.

Les points 12 à 15 sont des défauts visibles nouveaux, à présenter au propriétaire avec le plan des phases 3-4.
