# Parcours unifié, phases 2 à 4 — plan d'implémentation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remplacer la barre latérale et les deux écrans « Créer les collections » et « Tagger la sélection » par le cadre (en-tête, barre d'état, Réglages en plein écran) et le parcours en cinq étapes, avec un seul menu Lightroom et un seul nom d'annulation, puis réécrire les vérifications Lightroom et le guide.

**Architecture:** Le moteur d'écriture de la phase 1 ne change pas de forme : le parcours envoie toujours des options, et l'ancien circuit « sans option » disparaît avec les anciens écrans. Côté écran, un seul état de parcours (chemin, étape, plongées, options) vit dans un module pur et testé ; chaque étape est **un** composant qui reçoit le chemin et s'y adapte. Les deux lectures de photos existantes (recherche sur une période, description de la sélection) deviennent deux crochets extraits des écrans actuels, montés une fois dans l'enveloppe du parcours pour que les réglages d'horloge survivent aux allers-retours entre étapes et à l'ouverture des Réglages.

**Tech Stack:** TypeScript / React (front, tests Vitest sur les modules purs, fuseau UTC+14), Rust 2024 / Tauri 2 (crate `divelens`), Lua 5.1 (plug-in, testé sous mlua par `cargo test -p divelens`).

**Spec:** `docs/spec-v1.md` (F1, F2, F5, F7, sections 3 et 6) fait foi pour les textes français et anglais et pour les comportements ; `docs/superpowers/specs/2026-11-24-parcours-unifie-design.md` pour l'intention et les anti-objectifs ; l'annexe « Notes pour les phases 2 à 4 » de `docs/superpowers/plans/2026-11-24-parcours-unifie-phases-0-1.md` pour ce que la phase 1 a livré. Les maquettes sont privées et ne sont pas liées depuis le dépôt : le contrôleur les fournit à l'exécutant de chaque tâche d'écran, hors dépôt. Elles font foi pour la mise en page.

## Global Constraints

- Dans le doute, on garde ce que fait la version 2.1.1. Tout écart visible qui n'est ni dans la spécification ni dans « Choix signalés au propriétaire » ci-dessous est signalé, jamais décidé par l'exécutant.
- Chaque étape est **un seul composant**, qui s'adapte au chemin. Deux copies d'un écran, une par chemin, sont un échec (anti-objectif de la note).
- Les décisions vivent dans des modules purs testés (`journey/*.ts`, `Tagging.lua`, fonctions Rust pures) ; les composants et les enveloppes Lua restent minces. Aucun test existant n'est supprimé, désactivé ou assoupli : un test dont l'objet disparaît (ancien écran, ancien menu) est adapté à la nouvelle réalité, avec la même exigence.
- Les tests du plug-in pour les cas « toutes options cochées » (`plugin_collections.rs`, `plugin_tag_selection.rs`, `plugin_retagging.rs`, `plugin_options.rs`) passent **sans modification** de leurs valeurs attendues ; seuls changent les textes qu'une tâche nomme (nom d'annulation, menus).
- Deux écritures, un seul nom d'annulation, GPS et altitude dans la seconde ; entre les deux, rien n'est envoyé à l'app et rien ne cède la main. La création des collections reste dans la première.
- `PROTOCOL` reste à 7 : rien n'a été publié depuis la phase 1, l'app et le plug-in montent ensemble.
- Le numéro de version ne change pas dans ce plan. La publication (bêta, puis version mineure) est faite par le propriétaire.
- Le format du fichier de réglages ne change pas de version : le réglage d'altitude n'est plus lu, mais un fichier ou un export qui le contient reste lisible.
- Textes : chaque texte en anglais et en français, mêmes clés dans `app/src/i18n/en.ts` et `fr.ts`, « Dive Lens » en deux mots. Les textes sont ceux de la spécification, mot pour mot ; un texte absent de la spécification reprend celui de l'application actuelle. Une clé qui n'est plus utilisée est supprimée dans les deux fichiers.
- Rien de ce qui est interdit de renommer ne bouge (identifiant du plug-in, identifiants des champs, ensemble « Dive Lens », noms « numéro - site », identifiant de l'app).
- Données de test et exemples fictifs uniquement, avec des noms de site inventés. Aucun lien vers les maquettes dans le dépôt, ni dans une pull request.
- Plug-in : fichiers à plat, pas de `collectgarbage`, aucun faux Lightroom.
- Avant chaque commit : les vérifications du guide des contributeurs pour la partie touchée. Trois pull requests : phase 2 (`feat/app-frame`), phase 3 (`feat/unified-journey`), phase 4 (`docs/unified-journey-guide`). Fusion quand la CI est verte sur macOS et Windows.

## Choix signalés au propriétaire

Faits pour ce plan, visibles à l'écran, à corriger s'ils ne conviennent pas :

1. **Phase 2, accès provisoire aux deux anciens écrans** : deux onglets sous l'en-tête, « Créer les collections » et « Tagger la sélection ». Ils n'existent qu'entre les phases 2 et 3 et ne sont jamais publiés.
2. **Options de l'étape 4** : elles repartent à leur valeur par défaut à chaque nouveau passage (après le Récapitulatif, un changement de chemin, un nouveau carnet), mais sont gardées quand on revient à une étape précédente puis qu'on retourne à l'étape 4, ou quand on ouvre les Réglages.
3. **Explication « Couvre la sélection : 4 des 5 photos… »** : pour compter, la sélection est lue photo par photo dès l'entrée dans l'étape 2 du chemin « sélection », et non plus seulement une fois une plongée avec courbe choisie.
4. **Bloc « Hors de l'horaire de la plongée »** : au plus cinq photos détaillées (« Prise à 15:45, 7 minutes après la sortie de l'eau… »), puis « et N autres » (EN « and N more ») ; le nombre par appareil est toujours complet.
5. **GPS non écrit** (seconde écriture refusée par Lightroom) : on reste à l'étape 4 avec le message actuel, sans Récapitulatif, comme aujourd'hui.
6. **Refus « La sélection a changé… vérifiez les données par photo »** : texte inchangé, affiché à l'étape 4.
7. **Vérifications et guide écrits avant l'essai du propriétaire**, et non après comme le disait la note : il a ainsi la liste à dérouler dès son premier essai ; les corrections issues de l'essai les mettent à jour.
8. **Page d'accueil du site** (`docs/website/`) : elle cite les deux anciennes fonctionnalités ; elle n'est pas modifiée par ce plan, son texte est une décision du propriétaire.

## Review Focus

1. La sélection change dans Lightroom entre l'étape 3 et « Appliquer » : rien n'est écrit, le refus s'affiche à l'étape 4, la sélection est relue, on reste à l'étape 4 (Tasks 2, 6, 7).
2. La sélection devient vide en cours de parcours : on reste sur l'étape, « Appliquer » grisé avec « Sélectionnez des photos dans Lightroom » ; à l'étape 1, la carte se grise et « Suivant » aussi si ce chemin était choisi (Tasks 5, 8).
3. Lightroom se déconnecte à l'étape 3 ou 4, puis revient : avertissement, « Appliquer » grisé, puis nouvelle recherche des photos qui garde les décalages, les appareils ignorés et la marge déjà réglés (Task 6).
4. Le carnet change pendant que les Réglages sont ouverts : « Retour » ramène à l'étape 1 sans bandeau : il n'apparaît qu'après un refus (F2) ; plus aucune plongée de l'ancien carnet n'est cochée ni envoyée (Tasks 1, 5, 8).
5. « GPS » seul coché avec un site sans GPS, ou les trois options décochées : « Appliquer » grisé ou refus « rien à écrire » affiché, jamais une écriture vide qui crée une étape d'annulation (Tasks 5, 7).

---

## Fichiers touchés

| Fichier | Rôle |
|---|---|
| `app/src/ui/Header.tsx`, `StatusBar.tsx`, `statusBar.ts` (+ test) | Task 1 : cadre |
| `app/src/ui/Sidebar.tsx` | Task 1 : supprimé |
| `app/src/App.tsx`, `App.css` | Tasks 1, 8 |
| `app/src-tauri/src/tag.rs`, `tag/rules.rs`, `tag/selection.rs` | Tasks 2, 3 |
| `app/src-tauri/src/collections.rs`, `collections/plan.rs` | Tasks 2, 3 |
| `app/src-tauri/src/apply/options.rs`, `settings/file.rs` | Task 3 |
| `app/src-tauri/src/lightroom.rs` → `lightroom/tests.rs` | Task 3 : tests déplacés |
| `plugin/DiveLens.lrdevplugin/Info.lua`, `MenuOpen.lua` (nouveau), `MenuCreateCollections.lua` et `MenuTagSelection.lua` (supprimés), `CatalogWrite.lua`, `ApplyTags.lua`, `ApplyCollections.lua`, `Tagging.lua`, `PluginInfoProvider.lua`, `Launch.lua`, `TranslatedStrings_fr.txt` | Task 4 |
| `app/src-tauri/tests/plugin_manifest.rs`, `commands.rs` | Tasks 3, 4 |
| `app/src/journey/state.ts`, `rules.ts`, `texts.ts` (+ tests) | Task 5 |
| `app/src/journey/useCatalogPhotos.ts`, `useSelectionPhotos.ts`, `ClocksStep.tsx` | Task 6 |
| `app/src/journey/ReviewStep.tsx`, `SummaryStep.tsx` | Task 7 |
| `app/src/journey/Journey.tsx`, `PhotosStep.tsx`, `DivesStep.tsx`, `app/src/ui/Steps.tsx` | Task 8 |
| `app/src/collections/CollectionsScreen.tsx`, `DivesStep.tsx`, `rules.ts`, `app/src/tag/TagScreen.tsx`, `tag/rules.ts` | Task 8 : supprimés, ce qui sert encore déplacé dans `journey/` |
| `app/src/settings/SettingsScreen.tsx` | Tasks 1 (« Retour »), 8 (ligne d'altitude retirée) |
| `app/src/i18n/en.ts`, `fr.ts`, `app/src/types.ts`, `app/src/api.ts` | Toutes |
| `docs/contributing/lightroom-checks.md`, `docs/user/en.md`, `fr.md`, `README.md`, `CONTRIBUTING.md`, `CLAUDE.md` | Task 9 |

---

## Phase 2 — Cadre

### Task 1 : en-tête, barre d'état, Réglages en plein écran

Branche `feat/app-frame`, pull request `feat(app): replace the sidebar with a header and a status bar`.

**Files:**
- Create: `app/src/ui/Header.tsx`, `app/src/ui/StatusBar.tsx`, `app/src/ui/statusBar.ts`, `app/src/ui/statusBar.test.ts`
- Delete: `app/src/ui/Sidebar.tsx` (le type `Screen` et `isReadyScreen` vont dans `app/src/ui/screens.ts`)
- Modify: `app/src/App.tsx`, `app/src/App.css`, `app/src/settings/SettingsScreen.tsx`, `app/src/types.ts`, `app/src/i18n/en.ts`, `fr.ts`

**Interfaces:**
- Consumes: `LightroomStatus.catalogPhotos` (phase 1) ; l'app demande déjà la taille du catalogue à la connexion.
- Produces:
  - `statusBar.ts` : `lightroomItems(lr: LightroomStatus): { state: "connected" | "disconnected" | "restart"; title: string; hint: string | null; items: string[] }` et `logItems(log: DiveLog | null): string[]`.
  - `Header({ onSettings, settingsDisabled, inSettings })`, `StatusBar({ lr, log })`.
  - `SettingsScreen` reçoit `onBack: () => void`.
  - `App` : `screen` vaut toujours `"collections" | "tag" | "settings"` ; `const [before, setBefore]` retient l'écran d'où l'on a ouvert les Réglages.

- [ ] **Step 1 : tests de `statusBar.ts`** (`statusBar.test.ts`), avec les textes de F1 :
  - connecté, catalogue « Voyage », 48 409 photos, 12 sélectionnées → `items` = `["Catalogue : Voyage", "Taille du catalogue : 48 409 éléments", "Photos sélectionnées : 12"]` (EN `"Catalog: Voyage"`, `"Catalog size: 48,409 items"`, `"Selected photos: 12"`) ;
  - aucune sélection → `"Photos sélectionnées : aucune"` (EN `"Selected photos: none"`) ;
  - `catalogPhotos: null` → pas d'élément « Taille du catalogue » ;
  - déconnecté → `state: "disconnected"`, `items` vide, titre et aide actuels (`lr.disconnected`, `lr.disconnectedHint`) ; `restartLightroom` → `state: "restart"`, textes actuels ;
  - carnet lu → `["Application de carnet : <nom>", "Plongées enregistrées : N"]` (EN `"Dive log app: <name>"`, `"Dives logged: N"`) ; sans carnet → `[]`.
- [ ] **Step 2 :** lancer `npm test`, les voir échouer.
- [ ] **Step 3 : écrire `statusBar.ts`, `StatusBar.tsx`, `Header.tsx`.** En-tête : logo et « Dive Lens » à gauche, roue dentée à droite avec la bulle « Réglages » / « Settings », `aria-pressed` quand les Réglages sont ouverts, inactive quand `settingsDisabled`. Barre d'état en bas, groupe Lightroom à gauche, groupe carnet à droite.
- [ ] **Step 4 : `App.tsx`.** Supprimer la barre latérale. Mise en page en colonne : en-tête, contenu, barre d'état. La roue dentée ouvre les Réglages en plein écran à la place du contenu et retient l'écran quitté ; « Retour » (EN « Back »), en haut de l'écran Réglages, y ramène. La roue dentée est inactive pendant une écriture (`writing`). Les écrans du parcours restent **montés** pendant que les Réglages sont affichés (masqués par l'attribut `hidden`), pour que leurs choix soient retrouvés au retour ; un carnet changé les remet à zéro par le mécanisme existant (`otherLog`). Le menu Aide ouvre toujours les Réglages ; « Retour » ramène alors à l'écran par défaut.
- [ ] **Step 5 : onglets provisoires.** Sous l'en-tête, deux onglets « Créer les collections » / « Tagger la sélection » (clés `nav.collections`, `nav.tag` existantes) qui remplacent la navigation de la barre latérale, avec les mêmes règles (`open`). Un commentaire dit qu'ils disparaissent en phase 3.
- [ ] **Step 6 :** `npm test`, `npm run build`, `cargo fmt --all --check`, clippy de l'app, `cargo test -p divelens`. Tout passe.
- [ ] **Step 7 :** commit, push, pull request (section « Visible choices » : cadre décidé dans F1 ; onglets provisoires proposés par ce changement, jamais publiés).

---

## Phase 3 — Parcours

Branche `feat/unified-journey`, une seule pull request `feat: replace the two features with the five-step journey`, Tasks 2 à 8. Après chaque tâche, l'application construite fonctionne encore avec les anciens écrans, jusqu'à la Task 8 qui les remplace.

### Task 2 : ce que les étapes 3 à 5 demandent au moteur

**Files:**
- Modify: `app/src-tauri/src/tag.rs`, `tag/rules.rs`, `tag/selection.rs`, `collections.rs`, `collections/plan.rs`, `core/src/tags.rs` (seulement si `keep` doit garder `selection`), `app/src/types.ts`

**Interfaces:**
- Produces:
  - `TagApplied { count, with_depth: usize, collection }` (`withDepth`) : nombre de photos qui ont reçu des données par photo ; 0 quand « Détails de la plongée » est décochée.
  - `TagPreview.outside: Vec<OutsidePhoto>` et `TagPreview.outside_more: usize` ; `OutsidePhoto { time: String /* heure corrigée, "YYYY-MM-DDTHH:MM:SS" */, camera: String /* clé */, minutes: i64 /* négatif : avant la mise à l'eau ; positif : après la sortie */ }`. Les 5 premières photos datées hors de la plongée, par heure corrigée ; `outside_more` : les autres. Sans plongée : vide.
  - `CollectionsResult` avec des options : `photos` = photos écrites ou rangées dans une collection ; `with_depth` = photos qui ont reçu des données par photo (0 si la plongée est décochée) ; `empty_dives` = plongées cochées sans aucune photo rangée, **calculé avant** la passe `keep` : une plongée dont le groupe est retiré parce qu'il ne reste rien à écrire n'est pas « sans photo ».
  - Demande du chemin « sélection » : `TagRequest.selection` n'est renseigné que lorsque des données par photo sont envoyées, comme dans la 2.1.1. « Détails de la plongée » décochée, plongée sans courbe ou « Aucune plongée » : « Appliquer » écrit sur la sélection du moment (F5, « Refus “La sélection a changé” »). Corrigé le 2026-11-25 : la première version de ce plan nommait la sélection même plongée décochée, contre la spécification.
  - Chemin « catalogue », « Autre site » avec ni lieu ni GPS gardés : le site n'est pas cherché dans le carnet (pas de refus « site inconnu » pour rien).

- [ ] **Step 1 : tests qui échouent**, un par ligne ci-dessus :
  - `tag_applied_counts_photos_with_depth` : 3 photos, 2 dans la plongée → `with_depth == 2` ; mêmes photos, plongée décochée → `0`.
  - `preview_lists_photos_outside_the_dive` : plongée 10:00–10:45, photos à 09:53, 10:20, 10:52 → `outside` = `[(09:53, -7), (10:52, 7)]` ; 7 photos dehors → 5 listées, `outside_more == 2` ; les décalages affichés s'appliquent ; une photo sans heure n'est pas listée.
  - `a_group_left_with_nothing_is_not_an_empty_dive` : seule option « Détails du site », plongée sans site → le groupe disparaît de la demande, la plongée n'est pas dans `empty_dives` ; une plongée sans aucune photo y est toujours.
  - `another_site_without_place_or_gps_reads_no_site` : `otherSite` vers un site inconnu, `place == false && gps == false`, collections cochées → pas de refus.
- [ ] **Step 2 :** `cargo test -p divelens`, les voir échouer pour la bonne raison.
- [ ] **Step 3 :** implémenter.
- [ ] **Step 4 :** `cargo test -p divelens` et `cargo test -p divelens-core` passent ; `plugin_options.rs` passe sans modification.
- [ ] **Step 5 :** types TypeScript (`TagApplied.withDepth`, `TagPreview.outside`, `outsideMore`, `OutsidePhoto`, `LightroomStatus.selectionCameras` déjà envoyé par le moteur et pas encore typé). `npm run build`.
- [ ] **Step 6 :** vérifications, commit `feat(app): give the journey its counts and outside photos`.

### Task 3 : options obligatoires, réglage d'altitude retiré du moteur, rangement

**Files:**
- Modify: `app/src-tauri/src/tag.rs`, `tag/rules.rs`, `collections.rs`, `apply/options.rs`, `settings/file.rs`, `lightroom.rs`, `tests/commands.rs`, `app/src/types.ts`, `app/src/api.ts`, et les deux écrans actuels (une ligne chacun)
- Create: `app/src-tauri/src/lightroom/tests.rs`

**Interfaces:**
- Produces:
  - `WriteOptions { dive: bool, altitude: bool, place: bool, gps: bool, collections: bool }` : `altitude` n'est plus facultatif.
  - `TagChoice.options: WriteOptions` et `apply_collections(…, options: WriteOptions, other_site: Option<LocationChoice>)` : obligatoires. Le circuit « sans option » est supprimé.
  - Le moteur ne lit plus le réglage d'altitude : seule l'option envoyée compte. Le champ `depth_to_altitude` de `Settings` reste jusqu'à la Task 8, car l'écran Réglages actuel l'affiche encore. TypeScript : `WriteOptions.altitude: boolean`.

- [ ] **Step 1 : tests.**
  - Les tests du circuit « sans option » deviennent « tout coché, altitude donnée » et attendent **la même demande** qu'avant (mêmes valeurs attendues).
  - Petits tests de l'annexe : un groupe retiré pendant qu'un autre reste ; une réponse `collectionsApplied` avec un groupe sans nom lue côté Rust ; la commande `refresh_catalog_size` ; renommer `tag_request_another_site_is_not_applied_here` et `unknown_dive_or_site_is_refused` selon ce qu'ils vérifient ; reformuler l'assertion `{ collection = "x" }` de `plugin_collections.rs`.
- [ ] **Step 2 :** implémenter. Les deux écrans actuels envoient `{ dive, altitude: settings.depthToAltitude, place, gps, collections }` reconstruits depuis leurs cases, pour rester utilisables jusqu'à la Task 8 : la valeur d'altitude vient du réglage, lu côté écran.
- [ ] **Step 3 :** déplacer les tests de `lightroom.rs` dans `lightroom/tests.rs` (`#[cfg(test)] mod tests;`), sans en changer un seul.
- [ ] **Step 4 :** vérifications complètes ; `git diff --stat` montre que `plugin_*.rs` n'a changé que pour les tests nommés.
- [ ] **Step 5 :** commit `refactor(app): make write options mandatory`.

### Task 4 : un seul menu, un seul nom d'annulation

**Files:**
- Create: `plugin/DiveLens.lrdevplugin/MenuOpen.lua`
- Delete: `MenuCreateCollections.lua`, `MenuTagSelection.lua`
- Modify: `Info.lua`, `CatalogWrite.lua`, `ApplyTags.lua`, `ApplyCollections.lua`, `Tagging.lua`, `PluginInfoProvider.lua`, `Launch.lua`, `TranslatedStrings_fr.txt`, `app/src-tauri/tests/plugin_manifest.rs`, `app/src-tauri/src/lightroom/commands.rs`, `app/src/ui/screens.ts`, `app/src/App.tsx`, les textes d'erreur qui citent l'ancien nom d'annulation (`app/src/i18n/*.ts`, `app/src/apply/failure.ts`)

**Interfaces:**
- Produces:
  - Menu Bibliothèque : un seul élément, clé `$$$/DiveLens/Menu/Open=Open Dive Lens…` (FR « Ouvrir Dive Lens… »), fichier `MenuOpen.lua`, **sans** `enabledWhen` : actif avec ou sans sélection. Menu Aide inchangé.
  - Nom d'écran envoyé à l'app : `"journey"` (menu et bouton « Ouvrir Dive Lens » du gestionnaire) ou `"settings"`. `isReadyScreen` accepte ces deux noms ; jusqu'à la Task 8, `"journey"` ouvre l'écran « Créer les collections ».
  - `CatalogWrite.UNDO_NAME = LOC "$$$/DiveLens/Undo/DiveLens=Dive Lens"`, utilisé par `ApplyTags.lua` et `ApplyCollections.lua` ; les deux anciennes clés disparaissent.
  - `Tagging.collectionWrites(plan)` : la liste des groupes qui créent ou complètent une collection, à la place des deux gardes d'enveloppe (`g.collection and #group.photos > 0` et son équivalent dans `ApplyTags.lua`).

- [ ] **Step 1 : tests.** `plugin_manifest.rs` : les menus sont exactement `["Open Dive Lens…", "Dive Lens Settings…"]`, chaque fichier existe, aucun élément n'a `enabledWhen` ; chaque clé `$$$/DiveLens/…` du plug-in a sa traduction française et aucune traduction n'est orpheline (le test existant, qui doit continuer de passer) ; nom d'annulation : `"Dive Lens"` en anglais et en français dans les deux enveloppes. `collectionWrites` : un groupe sans collection, un groupe sans photo, un groupe normal → seul le dernier. `screen_arg` : `--screen journey`.
- [ ] **Step 2 :** les voir échouer, implémenter, les voir passer.
- [ ] **Step 3 :** textes d'erreur de l'app qui citent « Tagger la sélection (Dive Lens) » ou « Créer les collections (Dive Lens) » comme nom d'annulation → « Dive Lens » ; en-têtes de commentaire des fichiers Lua mis à jour.
- [ ] **Step 4 :** vérifications, commit `feat(plugin): open Dive Lens from a single menu`.

### Task 5 : l'état du parcours et ses règles

**Files:**
- Create: `app/src/journey/state.ts`, `state.test.ts`, `rules.ts`, `rules.test.ts`, `texts.ts`, `texts.test.ts`
- Modify: `app/src/i18n/en.ts`, `fr.ts` (tous les textes de F5)

**Interfaces:**
- Produces (`state.ts`) :

```ts
export type Path = "catalog" | "selection";
export type Step = 1 | 2 | 3 | 4 | 5;
/** Selection path, step 2: a dive, "no dive — location only", or nothing chosen yet. */
export type DivePick = { kind: "none" } | { kind: "noDive" } | { kind: "dive"; id: string };
export interface Options {
  dive: boolean; altitude: boolean; place: boolean; gps: boolean; collections: boolean;
  site: "diveSite" | "other"; pick: SitePick;
}
export interface Journey {
  step: Step; path: Path | null;
  dives: Set<string>;      // catalog path
  dive: DivePick;          // selection path
  options: Options;
  run: number;             // changes when the run starts again: the photo hooks reset on it
}
export const start: Journey;
export const defaultOptions: Options; // everything ticked, altitude unticked, site "diveSite"
export type Action =
  | { type: "choosePath"; path: Path } | { type: "next" } | { type: "back" }
  | { type: "goTo"; step: Step } | { type: "toggleDive"; id: string } | { type: "setDives"; ids: string[] }
  | { type: "pickDive"; pick: DivePick } | { type: "setOptions"; options: Partial<Options> }
  | { type: "applied" } | { type: "backToDives" } | { type: "handleSeparately"; ids: string[] }
  | { type: "done" } | { type: "menu" } | { type: "logChanged" };
export function reduce(j: Journey, a: Action): Journey;
export function stepStates(j: Journey): ("done" | "on" | "next" | "skipped")[];
```

- Règles de `reduce`, chacune avec son test :
  - `choosePath` vers un autre chemin : plongées, choix de plongée et options repartent de zéro, `run + 1` ; vers le même : rien ne change.
  - `next` : 1→2 (chemin choisi) ; 2→3, ou 2→4 quand `dive.kind === "noDive"` ; 3→4. `back` : l'inverse, 4→2 pour « Aucune plongée ». Pas de `back` à l'étape 1 ni à l'étape 5.
  - `goTo` : seulement vers une étape déjà faite, jamais depuis l'étape 5, jamais vers l'étape 3 pour « Aucune plongée ».
  - `pickDive` `noDive` : `options.site` passe à `"other"` ; `dive` : revient à `"diveSite"`.
  - `applied` : étape 5. `backToDives` (catalogue) : étape 2, aucune plongée cochée, options par défaut, `run + 1`. `handleSeparately` : étape 3 avec ces seules plongées, options par défaut, `run + 1`. `done` (sélection) : étape 1, tout à zéro sauf le chemin, `run + 1`.
  - `menu` (« Ouvrir Dive Lens… ») : étape 1 en gardant les choix ; depuis l'étape 5, comme `backToDives` ou `done` puis étape 1.
  - `logChanged` : `start` avec `run + 1`.
  - Si le carnet ne contient plus aucune plongée cochée (chemin « catalogue ») aux étapes 3 ou 4, retour à l'étape 2 (règle actuelle de `App.tsx`).
  - `stepStates` : « Horloges » vaut `"skipped"` pour « Aucune plongée ».

- Produces (`rules.ts`), pures, reprises et étendues de `collections/rules.ts` et `tag/rules.ts` (dont les tests existants sont déplacés avec elles, valeurs attendues inchangées) :
  - `canGoNext(j, lr): string | null` : l'invite qui grise « Suivant » (« Cochez une ou plusieurs plongées », « Choisissez une plongée », « Sélectionnez des photos dans Lightroom », ou `null`). Étape 1, chemin « sélection » : grisé sans sélection.
  - `writeOptions(j): WriteOptions` : `altitude` faux si `dive` faux ; « Aucune plongée » → `{ dive: false, altitude: false, place, gps, collections: false }` ; `place` et `gps` tous deux faux quand « Détails du site » est décochée.
  - `tagChoice(j, search, offsets, gps): TagChoice` et `otherSite(j, gps): LocationChoice | undefined` (absent quand ni lieu ni GPS ne sont cochés).
  - `reviewBlocker(i): string | null`, dans l'ordre où le plongeur les rencontre : déconnecté ; (sélection) aucune photo sélectionnée ; recherche ou lecture en cours ou en échec ; courbe en cours de lecture ; ensemble de collections introuvable **seulement si** « Créer les collections » est cochée ; « Cochez au moins une option » ; « Choisissez un site » (« Autre site » sans site, lieu ou GPS coché) ; GPS invalide ; rien à écrire.
  - `effects(i): { dive: string; altitude: string; site: string; collections: string }` : les textes d'effet de l'étape 4 (« N photos · N avec profondeur », « Rien », « N photos », « Non », « N photos · lieu et GPS », « N collections », « Aucune »).
  - `goodToKnow(i): string[]` et `barSummary(i): string` (« N photos · N collections », « N photos · pas de collection », « N collections · rien n'est écrit sur les photos »).
  - `summary(i): { head: string; stats: { label: string; n: number }[]; warnings: string[] }` pour l'étape 5, dans les deux chemins.
- Produces (`texts.ts`) : `photosLine(j, …): string` (la ligne « Photos : … » de chaque étape) et `outsideSentence(p: OutsidePhoto): string` (« Prise à 15:45, 7 minutes après la sortie de l'eau. Rattachée quand même… »).

- [ ] **Step 1 :** écrire `state.test.ts` (une règle, un test), le voir échouer, écrire `state.ts`.
- [ ] **Step 2 :** ajouter à `en.ts` et `fr.ts` tous les textes de F5, étape par étape, mot pour mot depuis la spécification ; clés préfixées `journey.` (les textes repris tels quels de l'application gardent leur clé).
- [ ] **Step 3 :** écrire `rules.test.ts` et `texts.test.ts` avec les textes exacts de la spécification, en français et en anglais pour au moins un cas par fonction ; les cinq lignes du « Review Focus » qui relèvent des règles y ont chacune un test (sélection vide, trois options décochées, « GPS » seul et site sans GPS, ensemble introuvable sans collections cochées, « Aucune plongée »). Les voir échouer, écrire `rules.ts` et `texts.ts`.
- [ ] **Step 4 :** `npm test`, `npm run build`. Commit `feat(app): add the journey's state and rules`.

### Task 6 : les deux lectures de photos et l'étape 3

**Files:**
- Create: `app/src/journey/useCatalogPhotos.ts`, `useSelectionPhotos.ts`, `ClocksStep.tsx`
- Modify: `app/src/apply/CameraClock.tsx` (si la jauge « N dans la plongée · N hors » le demande), `app/src/i18n/*.ts`

**Interfaces:**
- Produces:
  - `useCatalogPhotos(log, lr, diveIds: string[], run: number)` → `{ search, period, cameras, offsets, ignored, touched, notes, margin, preview, previewFailed, setOffset, setIgnored, setMargin, suggest, suggestAll, searchAgain, choices }` : le contenu actuel de `CollectionsScreen.tsx` (recherche, conservation des réglages quand la liaison revient, comptes en direct, « dernier arrivé gagne »), déplacé sans changer une règle. Ne cherche que lorsque `diveIds` n'est pas vide.
  - `useSelectionPhotos(log, lr, dive: Dive | null, active: boolean, run: number)` → `{ described, current, reading, readFailure, counts, freshCounts, offsets, touched, notes, profile, needsValues, unreadable, setOffset, suggest, suggestAll, describeAgain, covering }` : le contenu actuel de `TagScreen.tsx`, plus `suggestAll` (appareils non réglés à la main) et la lecture dès que `active` (chemin « sélection », étape ≥ 2), choix signalé n° 3.
  - `ClocksStep({ path, journey, catalog, selection, log, lr, onBack, onNext })` : **un** composant. Commun : bloc « Horloge des appareils » (`CameraClock`), « Suggérer pour tous », aperçu par plongée, barre du bas. Selon `path` : catalogue → « Ignorer », marge, bloc « Hors plongée · N photos » ; sélection → en-tête « Les N photos sont rattachées à… », pas d'« Ignorer », pas de marge, bloc « Hors de l'horaire de la plongée · N photos », lignes « Pas de courbe » / « Courbe illisible », total « N photos rattachées à … · N avec profondeur ».

- [ ] **Step 1 :** extraire les deux crochets des écrans actuels, **qui les utilisent aussitôt** à la place de leur code : aucun comportement ne change, `npm test` et `npm run build` passent, les anciens écrans marchent toujours.
- [ ] **Step 2 :** écrire `ClocksStep.tsx` d'après la spécification (« Étape 3 — Horloges ») et la maquette fournie. Pas encore monté.
- [ ] **Step 3 :** vérifications. Commit `feat(app): add the clocks step`.

### Task 7 : étapes 4 et 5

**Files:**
- Create: `app/src/journey/ReviewStep.tsx`, `SummaryStep.tsx`
- Modify: `app/src/tag/SitePicker.tsx` (déplacé dans `journey/` s'il n'a plus d'autre usage), `app/src/apply/failure.ts`

**Interfaces:**
- Consumes: `rules.ts` (Task 5), les deux crochets (Task 6), `applyCollections(log, search, choices, options, otherSite)`, `tagSelection(log, choice)`, `collectionSets()`.
- Produces:
  - `ReviewStep({ path, journey, dispatch, catalog, selection, log, lr, settings, onOpenSettings, onApplied, onWriting })` : **un** composant, les mêmes options dans les deux chemins ; `onApplied(result: Applied)` avec `type Applied = { path: "catalog"; result: CollectionsResult; outside: number; cameras: Camera[]; ignored: string[] } | { path: "selection"; result: TagApplied; total: number; outside: OutsidePhoto[]; outsideMore: number }`.
  - `SummaryStep({ applied, journey, log, settings, onBackToDives, onHandleSeparately, onDone })`.

- [ ] **Step 1 : `ReviewStep.tsx`** d'après « Étape 4 — Vérifier et appliquer » et la maquette : groupes « Métadonnées » et « Collections », chaque option avec son texte et son effet à droite (`effects`), sous-option altitude grisée sans « Détails de la plongée », choix du site (« Site de chaque plongée » / « Site de la plongée : nom » / « Autre site » avec `SitePicker` et GPS modifiable), « Lieu » et « GPS », emplacement des collections avec le lien « Emplacement dans Réglages », « À savoir avant d'appliquer », barre du bas (`barSummary`, rappel d'annulation, « ← Précédent », « Appliquer à N photos »), progression pendant l'écriture. « Aucune plongée » : « Détails de la plongée » et « Créer les collections » visibles et grisées. « Appliquer » : `applyCollections` ou `tagSelection` selon le chemin, avec `writeOptions(j)`. Échec : avis à l'étape 4 avec les textes actuels (`applyFailure`), relecture de la sélection ou nouvelle recherche dans les mêmes cas qu'aujourd'hui (`describeAgain`, `staleSearch`, `noPhotos`).
- [ ] **Step 2 : `SummaryStep.tsx`** d'après « Étape 5 — Récapitulatif » : rappel du passage, « Écrit dans Lightroom », les trois compteurs, les collections sous leur emplacement avec « créée » / « complétée » (aucune ligne quand aucune collection n'est renvoyée, jamais de nom vide), avertissements, rappel d'annulation, boutons selon le chemin (`ResultView` actuel en est la base).
- [ ] **Step 3 :** tests purs manquants pour ce que ces deux écrans ont ajouté aux règles. Vérifications. Commit `feat(app): add the review and summary steps`.

### Task 8 : étapes 1 et 2, enveloppe du parcours, retrait des anciens écrans

**Files:**
- Create: `app/src/journey/Journey.tsx`, `PhotosStep.tsx`, `DivesStep.tsx` (déplacé de `collections/`)
- Modify: `app/src/ui/Steps.tsx`, `app/src/dives/DiveList.tsx` (ligne « Aucune plongée »), `app/src/App.tsx`, `App.css`, `app/src/settings/SettingsScreen.tsx`, `app/src/types.ts`, `app/src/i18n/*.ts`, `app/src-tauri/src/settings/file.rs`
- Delete: `app/src/collections/CollectionsScreen.tsx`, `collections/DivesStep.tsx`, `collections/rules.ts` (+ test), `app/src/tag/TagScreen.tsx`, `tag/rules.ts` (+ test), une fois leur contenu utile déplacé

**Interfaces:**
- Produces:
  - `Steps({ states, onGo })` : cinq étapes, libellés « Photos › Plongées › Horloges › Vérifier et appliquer › Récapitulatif », états de `stepStates` (« skipped » grisé, non cliquable).
  - `PhotosStep({ journey, dispatch, lr, notice, onDismissNotice })` : les deux cartes séparées par « ou » ; carte « sélection » avec le résumé (appareils de `lr.selectionCameras`, dates), les vignettes en direct (jusqu'à six, « +N »), grisée sans sélection ; barre du bas.
  - `DivesStep({ path, journey, dispatch, log, lr, sort, onSort, selection })` : **un** composant, `DiveList` en mode `"multi"` ou `"single"` ; sélection : en-tête, première ligne « Aucune plongée — lieu seulement », repère « Couvre la sélection » et son explication.
  - `Journey({ log, lr, settings, sort, onSort, notice, onDismissNotice, onOpenSettings, onWriting, menu })` : tient `useReducer(reduce, start)`, monte les deux crochets, affiche l'étape, la ligne « Photos : … » sous le fil.
- `App.tsx` : un seul écran de parcours et les Réglages. `screen: "journey" | "settings"`. Le parcours reste monté pendant les Réglages. Le menu « Ouvrir Dive Lens… » envoie `menu` (ignoré pendant une écriture). Un carnet changé envoie `logChanged`. Retour à l'étape 1 : `refreshCatalogSize()`.

- [ ] **Step 1 :** `Steps.tsx`, `PhotosStep.tsx`, `DivesStep.tsx` d'après la spécification et les maquettes.
- [ ] **Step 2 :** `Journey.tsx` et `App.tsx` ; retirer les onglets provisoires, les deux anciens écrans et leurs fichiers ; déplacer ce qui sert encore.
- [ ] **Step 3 : Réglages.** Retirer la ligne « Profondeur dans l'altitude » et ses textes ; retirer `depthToAltitude` de `types.ts` et `depth_to_altitude` de `Settings`. Tests d'abord, dans `settings/file.rs` : `a_settings_file_with_the_altitude_setting_still_loads` et `an_export_with_the_altitude_setting_still_imports` (un fichier et un export contenant `"depthToAltitude": true` se lisent, les autres valeurs intactes, la version du format inchangée) ; les tests existants qui écrivaient ce champ sont adaptés à un autre champ, sans perdre ce qu'ils vérifiaient.
- [ ] **Step 4 :** supprimer de `en.ts` et `fr.ts` toute clé qui n'est plus utilisée (`nav.*`, textes des anciens écrans) ; le test de parité des clés passe.
- [ ] **Step 5 :** `tests/commands.rs` et les tests de contrat qui nomment un écran : adaptés (`"journey"`).
- [ ] **Step 6 :** lancer l'application (`npm run tauri dev`) sans Lightroom : l'étape 1 s'affiche, carte « sélection » grisée, barre d'état « Lightroom déconnecté », roue dentée puis « Retour », parcours « catalogue » jusqu'à l'étape 3 qui annonce la liaison absente. Noter le résultat dans le rapport.
- [ ] **Step 7 :** toutes les vérifications. Commit `feat(app): replace the two screens with the journey`. Push, pull request ; « Visible choices » : ce qui est décidé dans la spécification, et les choix n° 2 à 6 de ce plan comme proposés.

---

## Phase 4 — Vérifications et documentation

### Task 9 : vérifications Lightroom, guide, README

Branche `docs/unified-journey-guide`, pull request `docs: describe the journey in the guide and the lightroom checks`.

**Files:**
- Modify: `docs/contributing/lightroom-checks.md`, `docs/user/en.md`, `docs/user/fr.md`, `README.md`, `CONTRIBUTING.md`, `CLAUDE.md`

- [ ] **Step 1 : `lightroom-checks.md`.** Même forme (identifiant, Avant, Étapes, Attendu, feuille de résultats). Séries :
  - **JRN** (parcours, cadre) : menu unique actif sans sélection ; étape 1 avec et sans sélection, vignettes ; taille du catalogue égale à « Toutes les photos » et recomptée au retour à l'étape 1 ; Réglages et « Retour » ; carnet changé depuis les Réglages ; menu pendant une écriture ; plug-in d'une autre version → « Redémarrez Lightroom ».
  - **CAT** (chemin « catalogue ») : tout ce que vérifiaient COL-01 à COL-04, joué par le parcours ; plus : une option décochée à la fois, « Autre site », « GPS » seul (un seul « Annuler » défait tout), collections seules.
  - **SEL** (chemin « sélection ») : tout ce que vérifiaient TAG-01 à TAG-08 ; plus : collection créée, puis complétée sans doublon et annoncée « complétée » ; « Aucune plongée — lieu seulement » ; sélection changée avant « Appliquer » ; sélection vidée.
  - **CLK**, **UNIT**, **SET** : repris ; l'altitude se vérifie à l'étape 4 (cochée, décochée), plus dans les Réglages.
  - **UNDO** : « Annuler Dive Lens » défait tout dans les deux chemins, GPS et altitude compris ; première création de l'ensemble « Dive Lens » : étape d'annulation à part, l'ensemble vide reste si l'écriture est ensuite refusée ; Windows : à jouer par un testeur.
  Chaque ancien test qui existe encore y est retrouvé ; une table « ancien identifiant → nouveau » clôt la page.
- [ ] **Step 2 : guide utilisateur**, français et anglais, mêmes sections : ouvrir Dive Lens (un menu), le cadre, les cinq étapes, les deux chemins, les options de l'étape 4 et ce que chacune écrit, l'explication longue de l'altitude (les quatre points de F7), le lieu seul, les photos hors plongée, l'annulation (dont l'ensemble « Dive Lens » créé à part), les Réglages. Exemples avec des sites inventés.
- [ ] **Step 3 :** `README.md`, `CONTRIBUTING.md` (« adds two Library menu items » → un seul ; « Trying the plug-in in Lightroom », étape 3) et `CLAUDE.md` s'ils citent les anciens menus ou écrans.
- [ ] **Step 4 :** relire les liens internes et les ancres. Commit, push, pull request.

---

## Fin du plan

- [ ] `main` contient les trois pull requests, CI verte sur macOS et Windows ; la version est toujours 2.1.1.
- [ ] Remis au propriétaire : la liste des choix signalés, l'état des points « Proposé » de la spécification, et ce qui reste à lui :
  1. essayer le parcours réel dans Lightroom (acceptation), puis dérouler les vérifications sur Mac ;
  2. corrections éventuelles, dans de nouvelles pull requests ;
  3. pré-version bêta pour les testeurs Windows, avec la vérification d'« Annuler » en attente ;
  4. version mineure ;
  5. la page d'accueil du site.
