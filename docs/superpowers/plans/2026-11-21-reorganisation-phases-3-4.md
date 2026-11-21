# Réorganisation, phases 3 et 4 — plan d'implémentation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Un seul circuit « Appliquer » pour « Tagger la sélection » et « Créer les collections » (plug-in, moteur, interface), après avoir aligné leurs comportements et corrigé les défauts visibles ; puis les deux corrections restantes et une version corrective.

**Architecture:** D'abord des pull requests de correction, chacune sur une seule surface, qui font se comporter pareil les deux copies et corrigent n°1, n°2 et les défauts visibles acceptés. Ensuite des pull requests de fusion qui ne font que déplacer : une enveloppe d'écriture dans le plug-in, un chemin d'écriture et un type d'erreur dans le moteur, un traducteur d'erreurs, une barre et un bloc « horloge des appareils » dans l'interface. Phase 4 : n°4 et n°13, puis la version.

**Tech Stack:** Lua 5.1 (plug-in Lightroom, testé sous mlua), Rust 2024 / Tauri 2 (crate `divelens`), React + TypeScript (Vitest, sans bibliothèque DOM).

**Spec:** `docs/superpowers/specs/2026-11-19-reorganisation-design.md` (phases 3 et 4, anti-objectifs, décisions réservées) et l'annexe « Notes pour la phase 3 » de `docs/superpowers/plans/2026-11-19-reorganisation-phases-0-2.md` (points 1 à 17, cités « A1 » à « A17 »).

## Global Constraints

- Jamais déplacement et correction dans la même pull request. Une PR de fusion ne change aucun test existant, sauf leurs chemins d'import.
- Rien ne change dans ce qui est écrit sur les photos, l'ordre des deux écritures, le nom d'annulation, ni le fait qu'un seul « Annuler » défait l'écriture (A5 excepté, voir le tableau).
- Entre les deux écritures du plug-in, rien n'est envoyé à l'app et rien ne cède la main ; GPS et altitude dans la seconde.
- Les messages entre l'app et le plug-in ne changent pas : on réutilise les raisons existantes (`noSelection`, `busy`, `badRequest`, `staleSearch`, `parentNotFound`, `error`, `gpsNotWritten`, `selectionChanged`…). Si un message devait changer, `PROTOCOL` monte dans `Link.lua` et `lightroom.rs` ensemble, et le propriétaire est prévenu avant.
- Le format du fichier de réglages et des fichiers de décalages ne change pas.
- Le numéro de version ne change qu'à la toute fin, par `npm run release -- patch`, lancé par le propriétaire.
- Interface : chaque décision testable va dans un module de règles (`tag/rules.ts`, `collections/rules.ts`, `apply/*.ts`) testé par Vitest ; aucune nouvelle dépendance.
- Plug-in : fichiers à plat, pas de sous-dossier. Aucun faux Lightroom : les décisions vont dans `Tagging.lua`, testé ; l'enveloppe reste mince.
- Textes visibles en anglais et en français, « Dive Lens » en deux mots. Les nouveaux textes sont proposés dans la PR et validés par le propriétaire avant fusion.
- Données de test fictives uniquement. Aucune référence privée (correspondance, fichiers personnels) dans le dépôt.
- Vérifications avant chaque commit : celles du guide des contributeurs pour la partie touchée. Push, ouverture de PR et fusion : seulement sur accord explicite du propriétaire, CI verte.
- Une PR qui change un résultat visible met à jour `docs/contributing/lightroom-checks.md` dans la même PR.

## Review Focus

1. Une erreur Lua imprévue après la première écriture : l'app ne doit jamais dire « Rien n'a été écrit » alors que des photos ont changé (Task 1).
2. Lightroom occupé pendant la seconde écriture : les deux fonctionnalités disent « tout écrit sauf GPS et altitude », avec le bon nom d'annulation, avant comme après la fusion (Tasks 1, 7).
3. Sélection changée entre la description et « Appliquer » : refus et rien d'écrit, identique avant et après la fusion (Tasks 3, 8).
4. Liaison perdue pendant une écriture : l'écran quitte « écriture en cours » et le dit (Tasks 1, 8).
5. Enregistrement d'un réglage qui échoue (dossier en lecture seule) : l'écran revient à l'ancienne valeur et le dit (Task 11).

---

## Sort de chaque point de l'annexe

| Point | Sort | Où |
|---|---|---|
| A1 Tag ne lit l'état des photos que s'il écrit une plongée | Gardé : devient un paramètre de l'enveloppe commune (`before` facultatif) | Task 7 |
| A2 Total = photos demandées, introuvables comprises | Gardé : la barre avance d'un pas par photo demandée et atteint toujours le total ; commenté | Task 7 |
| A3 `created` vrai pour un groupe sans photo, et pour un second groupe du même nouveau nom | Corrigé : « créée » seulement pour une collection réellement créée, une fois | Task 2 |
| A4 Collections ne compte pas les valeurs d'une ancienne plongée remplacées | Aligné : les deux journaux donnent ce compte | Task 2 |
| A5 L'ensemble « Dive Lens » est créé dans sa propre écriture | **Gardé différent** : Lightroom ne laisse pas utiliser un ensemble créé dans l'écriture en cours ; le contrôle 8 documente le second « Annuler » | — |
| A6 Message `apply` non protégé (n°2) | Corrigé | Task 1 |
| A7 Option altitude lue par Tag seulement avec valeurs par photo | Aligné : lue au même endroit pour les deux, résultat identique (sans valeur par photo, rien à copier) | Task 3 |
| A8 Décalages mémorisés selon deux règles | Aligné sur la règle de Tag : écrit ou à moitié écrit, et au moins un décalage appliqué | Task 3 |
| A9 Deux types d'erreur pour Tag | Aligné : l'écriture de Tag renvoie ses refus de Lightroom comme Collections (en erreur) | Task 3 |
| A10 Verrous empoisonnés | Phase 4 (n°13) | Task 12 |
| A11 Trois traducteurs, code brut dans Tag (n°1) | Textes lisibles en Task 4 ; un seul traducteur en Task 9 | Tasks 4, 9 |
| A12 Tag bloqué sur « Lecture de la sélection… » | Si accepté par le propriétaire | Task 5 |
| A13 GPS invalide abandonné en silence | Si accepté | Task 5 |
| A14 « Appliquer » grisé sans explication (Collections) | Si accepté | Task 6 |
| A15 Réglages des appareils actifs pendant une écriture | Si accepté | Task 6 |
| A16 Échec de recherche dit « Rien n'a été écrit… réappliquez » | Corrigé : textes propres à la recherche | Task 4 |
| A17 Réglage non enregistré laissé à l'écran (n°4) | Phase 4 | Task 11 |

**Différences voulues avec la référence 2.1.1** (à ne pas prendre pour des régressions à la recette) : messages d'erreur de « Tagger la sélection » lisibles ; messages d'échec de recherche reformulés ; libellé « créée » correct dans le résumé de « Créer les collections » ; défauts A12 à A15 corrigés s'ils sont acceptés. Aucun des contrôles 1 à 18 ne change de résultat attendu, sauf mention dans la PR concernée.

**Windows.** Pas de testeur : le contrôle 11 reste « non joué », pas d'installateur d'essai. Risque résiduel accepté par le propriétaire ou non : l'enveloppe fusionnée est exactement ce dont dépend « un seul Annuler » sous Windows, et la version corrective arrivera chez les utilisateurs Windows sans ce contrôle.

## Pull requests

| PR | Branche | Tasks | Nature |
|---|---|---|---|
| 3a | `fix/plugin-always-answers` | 1, 2 | correction (plug-in) |
| 3b | `fix/apply-engine-align` | 3 | correction (moteur + adaptation de l'écran Tag) |
| 3c | `fix/apply-messages` | 4 | correction (textes) |
| 3d | `fix/apply-screen-defects` | 5, 6 | correction (écrans), selon décision |
| 3e | `refactor/one-catalog-write` | 7 | fusion (plug-in) |
| 3f | `refactor/one-apply-engine` | 8 | fusion (moteur) |
| 3g | `refactor/one-apply-interface` | 9, 10 | fusion (interface) + test n°5 |
| — | — | recette | contrôles 1 à 18 sur macOS |
| 4a | `fix/settings-revert-on-failure` | 11 | correction n°4 |
| 4b | `fix/recover-after-internal-failure` | 12 | correction n°13 |
| — | — | 13 | recette, puis version |

Chaque PR part de `main` à jour, après fusion de la précédente (pas d'empilement cette fois : chaque fusion est courte).

---

## Phase 3 — corrections

### Task 1: Le plug-in répond toujours, et jamais « rien d'écrit » à tort (n°2)

**Files:**
- Modify: `plugin/DiveLens.lrdevplugin/Tagging.lua` (nouvelle règle `Tagging.crashReply`)
- Modify: `plugin/DiveLens.lrdevplugin/Link.lua:178-197` (le message `apply` passe par le gestionnaire protégé)
- Modify: `plugin/DiveLens.lrdevplugin/ApplyTags.lua`, `ApplyCollections.lua` (étape courante, réponse envoyée avant les lignes de journal)
- Test: `app/src-tauri/tests/plugin_tag_selection.rs`

**Interfaces:**
- Produces: `Tagging.crashReply(id, stage, err) -> message | nil`, `stage` ∈ `"before"`, `"afterFirst"`, `"replied"`. Utilisé par Task 7.
- Produces: chaque enveloppe tient `s.applyStage = { id = id, stage = ... }` ; le gestionnaire protégé de `Link.lua` le lit en cas d'erreur.

- [ ] **Step 1: Tests de la règle**, dans `plugin_tag_selection.rs`, sur le modèle des tests de `Tagging.applyReply` :
  - `crash_before_the_first_write_says_nothing_written` : `crashReply(7, "before", "boom")` → `{ type = "applyFailed", id = 7, reason = "error", detail = "boom" }`.
  - `crash_after_the_first_write_says_written_but_gps` : `crashReply(7, "afterFirst", "boom")` → `reason = "gpsNotWritten"`, `detail = "boom"`.
  - `crash_after_the_reply_sends_nothing` : `crashReply(7, "replied", "boom")` → `nil`.
- [ ] **Step 2:** `cargo test -p divelens --test plugin_tag_selection crash_` → FAIL (`crashReply` absente).
- [ ] **Step 3:** Écrire `Tagging.crashReply`. Dans `Link.lua`, ajouter `apply = ApplyTags.run` à la table des gestionnaires protégés ; sur erreur, envoyer `Tagging.crashReply(message.id, stage, err)` (étape lue dans `s.applyStage` si son `id` est celui du message, sinon `"before"`), rien si `nil`. Dans les deux enveloppes : `stage = "before"` à l'entrée, `"afterFirst"` dès que la première écriture a renvoyé `"executed"`, `"replied"` juste après `Send.toApp(s, reply)` ; déplacer l'envoi de la réponse **avant** les lignes de journal qui suivent.
- [ ] **Step 4:** `cargo test -p divelens` → PASS (y compris « chaque fichier se compile »).
- [ ] **Step 5: Commit** `fix(plugin): always answer an apply, even on an unexpected error`

### Task 2: « Créée » seulement quand la collection l'est ; même journal pour les deux (A3, A4)

**Files:**
- Modify: `plugin/DiveLens.lrdevplugin/Tagging.lua` (`Tagging.collectionWrites`, comptes renvoyés)
- Modify: `plugin/DiveLens.lrdevplugin/ApplyCollections.lua` (ligne de journal)
- Test: `app/src-tauri/tests/plugin_collections.rs`

- [ ] **Step 1: Tests** :
  - `a_group_without_photo_found_is_not_created` : un groupe dont aucune photo n'est trouvée a `created = false`.
  - `a_second_group_of_the_same_new_name_is_completed` : deux groupes « 101 - Coral Garden » absents du catalogue → le premier `created = true`, le second `false`.
  - `collection_writes_count_replaced_dive_values` : le résultat de `collectionWrites` porte `cleared`, le nombre de photos dont une autre plongée a été effacée, comme `tagWrites`.
- [ ] **Step 2:** `cargo test -p divelens --test plugin_collections` → FAIL.
- [ ] **Step 3:** Corriger `collectionWrites` ; journal de Collections : « …, another dive's values cleared or replaced on %d » comme Tag.
- [ ] **Step 4:** `cargo test -p divelens` → PASS.
- [ ] **Step 5: Commit** `fix(plugin): mark only collections actually created as created`

### Task 3: Le moteur traite les deux écritures pareil (A7, A8, A9)

**Files:**
- Modify: `app/src-tauri/src/tag.rs` (`tag_selection`, `TagError`)
- Modify: `app/src-tauri/src/tag/rules.rs` (`remember_offsets` devient la règle commune)
- Modify: `app/src-tauri/src/collections.rs` (`apply_collections` : mémoire des décalages, option altitude)
- Modify: `app/src/types.ts`, `app/src/api.ts`, `app/src/tag/TagScreen.tsx`, `app/src/tag/rules.ts` (`selectionChanged`)
- Test: `app/src-tauri/src/tag/rules.rs` (tests), `app/src-tauri/src/lightroom.rs` (tests existants), `app/src/tag/rules.test.ts`

**Interfaces:**
- Produces: `TagError::Lightroom { reason: String, detail: Option<String> }` (même forme sérialisée que `CollectionsError::Lightroom` : `{ "error": "lightroom", "reason", "detail" }`).
- Produces: `tag_selection(...) -> Result<TagApplied, TagError>`, `TagApplied { count: usize }` sérialisé `{ "count": n }`. Un refus de Lightroom (`ApplyOutcome::Failed`) devient `Err(TagError::Lightroom { reason, detail })`.
- Produces: `Command::Apply { request, reply: mpsc::Sender<Result<usize, ApplyOutcome>> }`, sur le modèle de `Command::ApplyCollections` (traduction interne à `lightroom.rs`, aucun message vers le plug-in ne change).
- Produces: `crate::apply::offsets::should_remember(outcome: &ApplyOutcome, applied: &BTreeMap<String, i32>) -> bool` (l'actuelle `rules::remember_offsets`, déplacée ici), utilisée par les deux commandes. Pour Collections, `applied` = décalages des appareils dont au moins une photo a été écrite (ce que `Session::remember` enregistre aujourd'hui).
- Produces: `TagRequest::copy_depth_to_altitude` appelé pour Tag hors de la branche « valeurs par photo », comme pour Collections.

- [ ] **Step 1: Tests** :
  - `offsets_are_not_saved_when_none_was_applied` (Collections) : réponse écrite, aucun appareil → `should_remember` faux, fichier des décalages inchangé.
  - `a_refused_tagging_is_an_error` : `ApplyOutcome::Failed { reason: "busy" }` → `Err(TagError::Lightroom { reason: "busy", .. })`.
  - `the_altitude_option_leaves_a_request_without_photo_values_unchanged` : `copy_depth_to_altitude` sur une requête sans photo ne change rien.
  - `a_lost_link_during_tagging_is_link_lost` : canal de réponse fermé sans réponse → `Err(TagError::LinkLost)`.
  - Les tests existants de `Tagging.applyReply` (Lightroom occupé pendant la seconde écriture → `gpsNotWritten`) restent inchangés et passent.
  - Front : `selectionChanged` reconnaît `{ error: "lightroom", reason: "selectionChanged" }`.
- [ ] **Step 2:** `cargo test -p divelens` et `npm test` → FAIL sur ces tests.
- [ ] **Step 3:** Implémenter ; adapter `TagScreen.apply` : succès = `count`, échec = `catch`.
- [ ] **Step 4:** `npm test`, `npm run build`, `cargo clippy -p divelens --all-targets -- -D warnings`, `cargo test -p divelens` → PASS.
- [ ] **Step 5: Commit** `fix(app): handle both apply circuits' answers and offsets alike`

### Task 4: Messages lisibles partout (n°1, A16)

**Files:**
- Modify: `app/src/i18n.ts` (nouvelles clés en/fr)
- Modify: `app/src/tag/TagScreen.tsx` (`failure`, `describeFailure`), `app/src/collections/CollectionsScreen.tsx` (`failure` : cas de la recherche séparés de ceux de l'écriture)
- Test: `app/src/tag/TagScreen.test.ts`, `app/src/collections/CollectionsScreen.test.ts` (les traducteurs sont exportés de leur écran, sans être déplacés : le déplacement vient en Task 9)

- [ ] **Step 1: Tests** : pour chaque erreur de `TagError` (`nothingChosen`, `notConnected`, `linkLost`, `badGps`, `unknownDive`, `unknownSite`, `logChanged`, `noPhotos`, `source`, `lightroom` × chaque raison), le texte n'est pas vide et ne contient ni le code (`unknownDive`…) ni `{` ; idem pour les échecs de recherche de Collections, dont aucun texte ne contient « written » / « écrit » ni « apply again » / « réappliquez ». En anglais et en français (`setLang`).
- [ ] **Step 2:** `npm test` → FAIL.
- [ ] **Step 3:** Ajouter les clés. Textes proposés (à valider par le propriétaire dans la PR) :
  - `tag.err.nothingChosen` : « Choose a dive or a place to write. » / « Choisissez une plongée ou un lieu à écrire. »
  - `tag.err.badGps` : « The GPS position is not valid. Nothing was written. » / « La position GPS n'est pas valide. Rien n'a été écrit. »
  - `tag.err.unknownDive` : « This dive is no longer in the dive log. Nothing was written: choose a dive again. » / « Cette plongée n'est plus dans le carnet. Rien n'a été écrit : choisissez de nouveau une plongée. »
  - `tag.err.unknownSite` : même forme pour le site.
  - `tag.err.source` : « The dive log could not be read. Nothing was written: {detail} » / « Le carnet n'a pas pu être lu. Rien n'a été écrit : {detail} »
  - `logChanged` : la clé existante `log.changedNothingWritten`.
  - Recherche de Collections (`coll.search.err.*`) : « The photos could not be looked for in Lightroom: {detail}. Try again. », « The link with Lightroom was lost: try again once Lightroom is connected. », etc., sans parler d'écriture.
- [ ] **Step 4:** `npm test`, `npm run build` → PASS.
- [ ] **Step 5: Commit** `fix(app): show a readable message for every apply and search failure`

### Task 5: « Tagger la sélection » ne reste plus bloqué, et refuse un GPS invalide (A12, A13) — si accepté

**Files:**
- Modify: `app/src/tag/rules.ts`, `app/src/tag/TagScreen.tsx`
- Test: `app/src/tag/rules.test.ts`

**Interfaces:**
- Produces: `readingAfterFailure(e: CollectionsError, live: boolean): { reading: boolean; message: string | null }` — `logChanged`/`noPhotos` d'une description encore la plus récente terminent la lecture (`reading: false`) et relancent une description (`again`), au lieu de laisser « Lecture de la sélection… ».
- Produces: `gpsError` vrai dès que le lieu écrit est un autre site (`!usesDiveSite`, quel que soit `placeKind`) et que la position saisie n'est pas valide.

- [ ] **Step 1: Tests** : `readingAfterFailure({ error: "noPhotos" }, true)` → `reading: false` ; `(…, false)` → rien ne change ; `tagBlocker` avec `placeKind: "diveSite"`, site de la plongée inutilisable, GPS saisi invalide → `t("tag.badGps")`.
- [ ] **Step 2:** `npm test` → FAIL.
- [ ] **Step 3:** Implémenter ; l'écran utilise ces règles.
- [ ] **Step 4:** `npm test`, `npm run build` → PASS.
- [ ] **Step 5: Commit** `fix(app): never leave tag selection reading, refuse an invalid gps`

### Task 6: « Créer les collections » dit pourquoi « Appliquer » est grisé ; appareils figés pendant l'écriture (A14, A15) — si accepté

**Files:**
- Modify: `app/src/collections/rules.ts` (`collectionsBlocker`), `app/src/collections/CollectionsScreen.tsx`, `app/src/tag/TagScreen.tsx`, `app/src/apply/CameraCard.tsx` (prop `disabled`)
- Modify: `app/src/i18n.ts` (`coll.previewFailed`, en/fr)
- Test: `app/src/collections/rules.test.ts`

- [ ] **Step 1: Tests** : `collectionsBlocker` avec aperçu en échec → texte `coll.previewFailed` (non vide, en/fr) ; `canApplyCollections` inchangé.
- [ ] **Step 2:** `npm test` → FAIL.
- [ ] **Step 3:** Implémenter ; pendant une écriture, chaque carte d'appareil reçoit `disabled` (Suggérer, Ignorer, champ et curseur du décalage), dans les deux écrans.
- [ ] **Step 4:** `npm test`, `npm run build` → PASS.
- [ ] **Step 5: Commit** `fix(app): say why apply is unavailable, freeze cameras while writing`

---

## Phase 3 — fusion (aucun changement de comportement)

### Task 7: Une seule enveloppe d'écriture dans le plug-in

**Files:**
- Modify: `plugin/DiveLens.lrdevplugin/CatalogWrite.lua` (nouvelle `CatalogWrite.apply`)
- Modify: `ApplyTags.lua`, `ApplyCollections.lua` (ne gardent que leur préparation et leur `job`)
- Test: tous les tests `plugin_*.rs` inchangés ; `the_plugin_files_compile` (existant)

**Interfaces:**
- Consumes: `Tagging.crashReply` (Task 1), `Tagging.applyReply`, `CatalogWrite.writeGpsLater`.
- Produces: `CatalogWrite.apply(s, id, job)` qui envoie exactement une réponse. `job` = `{ undoName, total, before = function(catalog) -> map | nil, writeFirst = function(catalog, before, advance) -> { later = {...}, result = ... }, done = function(result) -> message, log = function(result, second) }`. `before` absent = pas de lecture (A1) ; l'ensemble « Dive Lens » de Collections reste créé avant, hors de `apply` (A5).

- [ ] **Step 1:** Écrire `CatalogWrite.apply` en reprenant mot pour mot la séquence actuelle (pcall, première écriture, seconde aussitôt, `s.lastHeard`, `applyReply`, envoi, journal, étapes de Task 1).
- [ ] **Step 2:** Réécrire `ApplyTags.run` et `ApplyCollections.run` sur `CatalogWrite.apply`.
- [ ] **Step 3:** `cargo test -p divelens` → PASS sans toucher un test.
- [ ] **Step 4: Commit** `refactor(plugin): write both features through one catalog write`

### Task 8: Un seul chemin d'écriture et un seul type d'erreur dans le moteur

**Files:**
- Modify: `app/src-tauri/src/apply/errors.rs` (`CollectionsError` → `ApplyError`, qui reçoit `BadGps`, `UnknownDive`, `UnknownSite` de `TagError`)
- Modify: `app/src-tauri/src/apply.rs` (nouvelle `write_and_remember`)
- Modify: `app/src-tauri/src/tag.rs`, `tag/rules.rs`, `collections.rs`, `collections/plan.rs`, `lib.rs`
- Test: tests existants (`tag/rules.rs`, `collections/plan.rs`, `lightroom.rs`, `tests/commands.rs`) inchangés hors renommage

**Interfaces:**
- Consumes: `should_remember` (Task 3).
- Produces: `ApplyError` sérialisé exactement comme les deux types actuels (`{ "error": "<camelCase>" , ... }`).
- Produces: `apply::write_and_remember<T>(app: &AppHandle, info: &SourceInfo, command: impl FnOnce(mpsc::Sender<Result<T, ApplyOutcome>>) -> Command, applied: &BTreeMap<String, i32>) -> Result<T, ApplyError>` : envoie, attend, mémorise les décalages selon `should_remember`, journalise.

- [ ] **Step 1:** Renommer et fusionner les types ; `tag_selection` et `apply_collections` passent par `write_and_remember`.
- [ ] **Step 2:** `cargo clippy -p divelens --all-targets -- -D warnings`, `cargo test -p divelens` → PASS ; `tests/commands.rs` : même liste de commandes.
- [ ] **Step 3: Commit** `refactor(app): one write path and one error type for apply`

### Task 9: Un traducteur, une barre, un bloc « horloge des appareils » dans l'interface

**Files:**
- Create: `app/src/apply/failure.ts` (+ `failure.test.ts`), `app/src/apply/ProgressBar.tsx`, `app/src/apply/CameraClock.tsx`
- Modify: `app/src/types.ts` (`ApplyError`), `tag/TagScreen.tsx`, `tag/rules.ts`, `collections/CollectionsScreen.tsx`, `collections/rules.ts`
- Test: les tests des traducteurs (Task 4) déplacés vers `apply/failure.test.ts` sans changer leurs assertions

**Interfaces:**
- Produces: `applyFailure(e: ApplyError, feature: "tag" | "collections"): string`, `searchFailure(e: ApplyError, feature: "tag" | "collections"): string | null`. `feature` choisit les textes propres à chaque écran (nom d'annulation, « sélection » / « photos trouvées »).
- Produces: `<ProgressBar done total label />`, `<CameraClock cameras offsets onChange onSuggest disabled ... />` (props reprises du bloc commun aux deux écrans).

- [ ] **Step 1:** Déplacer, sans changer un texte ni un comportement.
- [ ] **Step 2:** `npm test`, `npm run build` → PASS.
- [ ] **Step 3: Commit** `refactor(app): one error translator, progress bar and camera block`

### Task 10: Test n°5 — plug-in, moteur et interface parlent des mêmes choses

**Files:**
- Create: `app/src-tauri/tests/apply_contract.rs`
- Modify: `plugin/DiveLens.lrdevplugin/Tagging.lua` (`Tagging.REASONS`, liste de toutes les raisons d'`applyFailed`), `app/src-tauri/src/lightroom.rs` (`pub const APPLY_REASONS: &[&str]`)

Sur le modèle de `the_plugin_speaks_the_same_protocol` (lecture de texte ligne par ligne, CRLF toléré) et des états Lua de `tests/common` :

- [ ] **Step 1: Tests** :
  - `plugin_and_app_know_the_same_reasons` : `Tagging.REASONS` (lu sous mlua) == `APPLY_REASONS`.
  - `every_reason_has_a_text` : chaque raison apparaît comme `case "<raison>"` dans `app/src/apply/failure.ts`.
  - `the_undo_names_match` : les noms d'annulation de `ApplyTags.lua`/`ApplyCollections.lua` (anglais, texte après `=` du `LOC`) et de `TranslatedStrings_fr.txt` apparaissent dans les textes `gpsNotWritten` de `i18n.ts`, en anglais et en français.
  - `the_screens_match` : les noms d'écran passés par `MenuTagSelection.lua`, `MenuCreateCollections.lua`, `MenuSettings.lua` sont ceux que l'app accepte (`FromPlugin::Show`, `--screen`).
- [ ] **Step 2:** `cargo test -p divelens --test apply_contract` → FAIL tant que les listes n'existent pas, puis PASS.
- [ ] **Step 3: Commit** `test: check that plugin, engine and screens share reasons and names`

### Recette de la phase 3

- [ ] Le propriétaire joue les contrôles 1 à 18 sur macOS avec la version construite depuis `main` (ses copies de catalogue réelles pour 1 à 10, les photos fictives pour 11 à 18). Résultat comparé à la phase 2 ; seules les différences voulues listées plus haut sont admises. Contrôle 11 : non joué.

---

## Phase 4 — corrections restantes et version

### Task 11: Un réglage non enregistré revient à l'écran (n°4, A17)

**Files:**
- Create: `app/src/settings/save.ts` (+ `save.test.ts`)
- Modify: `app/src/settings/SettingsScreen.tsx` (unités, altitude, emplacement des collections, langue, tri)

**Interfaces:**
- Produces: `saveSetting<T>(previous: T, next: T, save: (v: T) => Promise<void>): Promise<{ value: T; error: string | null }>` — sur échec, `value = previous` et un message lisible (`settings.notSaved`, en/fr, avec le détail).

- [ ] **Step 1: Tests** : `save` résolu → `{ value: next, error: null }` ; `save` rejeté → `{ value: previous, error }` non vide, en/fr.
- [ ] **Step 2:** `npm test` → FAIL.
- [ ] **Step 3:** Implémenter ; chaque réglage de l'écran passe par `saveSetting`.
- [ ] **Step 4:** `npm test`, `npm run build` → PASS.
- [ ] **Step 5: Commit** `fix(app): show the previous setting when saving it fails`

### Task 12: L'app se rétablit partout après une panne interne (n°13, A10)

**Files:**
- Create: `app/src-tauri/src/locks.rs`
- Modify: tous les `.lock().expect(...)` de `app/src-tauri/src/` (lib.rs, tag*, collections*, apply*, lightroom*, update*)

**Interfaces:**
- Produces: `pub(crate) fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T>` : verrou empoisonné → `PoisonError::into_inner`, avec un `log::error!` ; même réaction que réglages, carnet et journal aujourd'hui.

- [ ] **Step 1: Tests** : `a_poisoned_lock_is_recovered` (un fil panique en tenant le verrou ; `lock` rend la valeur) ; `no_lock_panics_anymore` : aucun fichier de `src/` ne contient `.lock().expect(` ni `.lock().unwrap(` (lecture des sources, comme le test du protocole).
- [ ] **Step 2:** `cargo test -p divelens locks` → FAIL.
- [ ] **Step 3:** Implémenter et remplacer.
- [ ] **Step 4:** `cargo clippy -p divelens --all-targets -- -D warnings`, `cargo test -p divelens` → PASS.
- [ ] **Step 5: Commit** `fix(app): recover from an internal failure instead of stopping`

### Task 13: Recette finale et version corrective

- [ ] Le propriétaire rejoue les contrôles 1 à 18 sur macOS (11 non joué) avec la version construite depuis `main`.
- [ ] Le propriétaire lance, depuis `app/` sur `main` à jour : `npm run release -- patch` (2.1.1 → 2.1.2). La PR `build: release 2.1.2` se fusionne d'elle-même une fois la CI verte ; le brouillon de version est relu puis publié par le propriétaire.
- [ ] Après publication : vérifier qu'une copie 2.1.1 installée propose bien la 2.1.2.
- [ ] Nettoyage, avec l'accord du propriétaire : copies de travail `.worktrees/*`, branches distantes déjà fusionnées.
