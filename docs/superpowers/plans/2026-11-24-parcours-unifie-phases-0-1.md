# Parcours unifié, phases 0 et 1 — plan d'implémentation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Réécrire la spécification pour le parcours unifié, puis donner au moteur et au plug-in tout ce que l'étape 4 du parcours demandera (options d'écriture, collection depuis une sélection, autre site pour le catalogue, taille du catalogue), sans changer un seul écran.

**Architecture:** Les options ne créent pas de nouveau circuit : l'app construit les demandes d'écriture comme aujourd'hui, puis une passe pure dans `core` retire ce qui n'est pas coché (`keep`). Le plug-in apprend seulement trois formes de demande qu'il refusait : un groupe sans plongée, un groupe sans collection, un « apply » avec une collection. Les écrans actuels n'envoient aucune option : les valeurs par défaut reproduisent exactement la 2.1.1. Les phases 2 à 4 (cadre, parcours, recette) auront leur propre plan, écrit une fois la phase 1 fusionnée.

**Tech Stack:** Lua 5.1 (plug-in Lightroom, testé sous mlua par `cargo test -p divelens`), Rust 2024 / Tauri 2 (crates `divelens-core` et `divelens`), TypeScript (types seulement dans ce plan).

**Spec:** `docs/superpowers/specs/2026-11-24-parcours-unifie-design.md` (sections « Règles d'écriture », « Barre d'état », phases 0 et 1, anti-objectifs). Les maquettes sont privées : le propriétaire les fournit pour la Task 1.

## Global Constraints

- Dans le doute, on garde ce que fait la version 2.1.1. Tout écart visible non listé dans « Ce qui change pour le plongeur » de la note est signalé au propriétaire, jamais décidé ici.
- Aucun écran ne change dans ce plan. Sans option envoyée, chaque commande écrit exactement ce qu'elle écrit aujourd'hui : **aucun test existant n'est modifié**, sauf ceux que ce plan nomme (protocole, type de `collection`).
- Deux écritures, un seul nom d'annulation, GPS et altitude dans la seconde ; entre les deux, rien n'est envoyé à l'app et rien ne cède la main. La création des collections reste dans la première écriture.
- `PROTOCOL` passe de 6 à 7 **une seule fois**, à la Task 3, dans `app/src-tauri/src/lightroom.rs` et `plugin/DiveLens.lrdevplugin/Link.lua` ensemble. Les Tasks 4 et 7 changent encore des messages sans le remonter : rien n'est publié entre-temps.
- Le numéro de version ne change pas. Le format du fichier de réglages ne change pas (le réglage d'altitude reste lu dans ce plan).
- Plug-in : fichiers à plat, pas de `collectgarbage`, décisions dans `Tagging.lua` (testé), enveloppes minces. Aucun faux Lightroom.
- Rien de ce qui est interdit de renommer ne bouge : identifiants des champs, ensemble « Dive Lens », noms « numéro - site ».
- Données de test fictives uniquement. Aucun lien vers les maquettes dans le dépôt.
- Une pull request par Task, branche `<type>/<slug>` indiquée dans la Task, titre au format Conventional Commits. Avant chaque commit : les vérifications du guide des contributeurs pour la partie touchée. Le propriétaire a autorisé le push et l'ouverture des pull requests pour ce chantier ; si l'outil le refuse, s'arrêter et le lui dire. Fusion : par lui, CI verte.

## Review Focus

1. Plug-in resté en protocole 6 face à une app en 7 : l'app affiche l'état existant « redémarrez Lightroom » et n'écrit rien (Task 3).
2. « Détails de la plongée » décochée sur une photo qui porte déjà une autre plongée : aucun champ de plongée, aucune valeur par photo, aucune altitude n'est effacée (Tasks 3 et 6).
3. « GPS » seul coché et site sans GPS, rien d'autre de coché : refus « rien à écrire » avant d'ouvrir une écriture, jamais une écriture vide qui crée une étape d'annulation (Tasks 2 et 5).
4. Chemin « sélection » avec collection quand l'ensemble de collections choisi n'existe plus : refus `parentNotFound`, aucune photo touchée (Task 4).
5. Chemin « sélection », seconde application sur les mêmes photos : la collection existe déjà, elle est complétée sans doublon et annoncée `created = false` (Task 4).

---

## Fichiers touchés

| Fichier | Rôle dans ce plan |
|---|---|
| `docs/spec-v1.md` | Task 1 : F1, F2, F5, F7, F8, sections 3, 6, 7 |
| `core/src/tags.rs` | Tasks 2, 4 : `Kept`, `keep`, `CollectionPlace`, champs devenus facultatifs |
| `plugin/DiveLens.lrdevplugin/Tagging.lua` | Tasks 3, 4 : groupe sans plongée ou sans collection, collection d'un « apply » |
| `plugin/DiveLens.lrdevplugin/CatalogWrite.lua` | Task 4 : recherche ou création de l'ensemble, partagée |
| `plugin/DiveLens.lrdevplugin/ApplyCollections.lua`, `ApplyTags.lua` | Tasks 3, 4 |
| `plugin/DiveLens.lrdevplugin/Link.lua` | Tasks 3, 7 : protocole 7, `countPhotos` |
| `app/src-tauri/src/lightroom.rs` | Tasks 3, 4, 7 : protocole, réponses, `CountPhotos` |
| `app/src-tauri/src/apply/options.rs` (nouveau) | Task 5 : `WriteOptions` |
| `app/src-tauri/src/tag.rs`, `tag/rules.rs` | Task 5 |
| `app/src-tauri/src/collections.rs`, `collections/plan.rs` | Task 6 |
| `app/src-tauri/tests/plugin_collections.rs`, `plugin_tag_selection.rs`, `plugin_retagging.rs`, `plugin_options.rs` (nouveau) | Tasks 3, 4, 6 |
| `app/src/types.ts`, `app/src/api.ts` | Tasks 5 à 7 : types et appels, sans usage à l'écran |

---

## Phase 0 — Spécification

### Task 1 : réécrire la spécification

Branche `docs/spec-unified-journey`, titre `docs(spec): describe the unified journey`.

**Files:**
- Modify: `docs/spec-v1.md`

**Interfaces:**
- Consumes: la note de conception, les maquettes (dix écrans), fournies par le propriétaire.
- Produces: la spécification qui fait foi pour les phases 1 à 4. Les Tasks suivantes citent ses numéros de fonctionnalité.

- [ ] **Step 1 : F1.** Un seul menu Bibliothèque « Ouvrir Dive Lens… » / « Open Dive Lens… » ; l'app s'ouvre sur l'étape 1 ; en-tête, barre d'état (textes exacts de la note, « Taille du catalogue : N éléments »), roue dentée. Aide › « Réglages de Dive Lens… » conservé.
- [ ] **Step 2 : F2.** Réglages en plein écran avec « Retour » et la règle de retour ; la ligne « Profondeur copiée dans l'altitude » sort de l'écran Réglages ; « Changer d'application », « Un seul carnet à la fois », « Pendant une écriture » et « Actions sur le carnet affiché » réécrits pour « le parcours » au lieu des deux écrans, sans changer une règle.
- [ ] **Step 3 : F5 devient « Le parcours ».** Cinq étapes, les deux chemins, chaque étape avec ses variantes, exactement comme la note. Reprendre **telles quelles** toutes les règles actuelles de F5 (recherche dans tout le catalogue, appareils, marge, « Suggérer », plongées lues une seule fois, relance) et de F8 (sélection en direct, refus « La sélection a changé », photo déjà rattachée à une autre plongée, « Couvre la sélection », lieu seul), chacune rattachée à son chemin. Le tableau « Différences entre chemins » de la note y figure.
- [ ] **Step 4 : étape 4.** Les cinq options, leur état à chaque passage, l'effet de chacune (section « Effet des options » de la note), la règle « au moins une option », le cas « Aucune plongée ».
- [ ] **Step 5 : F7.** L'option devient une case de l'étape 4, décochée à chaque passage ; le repère d'altitude et les règles d'effacement ne changent pas ; texte court de la maquette à l'étape 4, les quatre points de l'explication longue renvoyés au guide utilisateur (F12).
- [ ] **Step 6 : F8** remplacé par « (fusionné dans F5) », comme F9. Section 3 : menus, nom d'annulation unique « Dive Lens », protocole 7. Section 6 : inventaire des écrans. Section 7 : phrases qui citent « Créer les collections » et « Tagger la sélection » réécrites par chemin.
- [ ] **Step 7 : textes.** Recopier depuis les maquettes, en français et en anglais, tous les textes nouveaux que la note ne cite pas (cartes de l'étape 1, notes de l'étape 3, descriptions des options, « À savoir avant d'appliquer », récapitulatif). Le tableau des libellés de la note fait foi pour ceux qu'il donne.
- [ ] **Step 8 : étiquettes.** Chaque décision porte **[Décidé, 2026-11-23]** ou **[Décidé, 2026-11-24]** ; chaque choix de la section « Choix faits par le concepteur » porte **[Proposé, 2026-11-24]**. Les décisions remplacées gardent une mention « Avant le 2026-11-24 : … », comme le reste du document.
- [ ] **Step 9 : vérifier.** `grep -n "Tagger la sélection\|Créer les collections…\|barre latérale" docs/spec-v1.md` : chaque ligne restante est une mention historique voulue (« Avant… ») ou le nom de l'option « Créer les collections ». `cargo fmt --all --check`.
- [ ] **Step 10 : commit, push, pull request.** Dans la PR, section « Visible choices » : la liste des huit écarts de la note et des choix « Proposé ». **Arrêt : le propriétaire relit et fusionne avant la Task 2.**

---

## Phase 1 — Moteur et plug-in

### Task 2 : la passe « garder ce qui est coché » dans `core`

Branche `feat/write-options-core`, titre `feat(core): keep only the ticked parts of a write request`.

**Files:**
- Modify: `core/src/tags.rs`
- Test: `core/src/tags.rs` (module `tests` existant)

**Interfaces:**
- Produces:
  - `pub struct Kept { pub dive: bool, pub place: bool, pub gps: bool, pub collections: bool }` (`Clone, Copy, Debug, PartialEq`), `Kept::ALL` (tout vrai).
  - `impl LocationTags { pub fn keep(self, place: bool, gps: bool) -> Option<LocationTags> }` : retire pays, ville et lieu si `!place`, latitude et longitude si `!gps` ; `None` s'il ne reste aucune valeur.
  - `CollectionGroup` : `collection: Option<String>`, `dive: Option<DiveTags>`, `location: Option<LocationTags>` ; les trois sérialisés sans le champ quand `None` (`skip_serializing_if`).
  - `impl CollectionsRequest { pub fn keep(&mut self, kept: Kept, other_site: Option<&LocationTags>) ; pub fn is_empty(&self) -> bool }`.
  - `impl TagRequest { pub fn keep(&mut self, kept: Kept) ; pub fn is_empty(&self) -> bool }`.

Règles de `CollectionsRequest::keep`, par groupe :
- `other_site` donné : il remplace `location` avant tout le reste (le nom de la collection ne change pas).
- `location = location.and_then(|l| l.keep(kept.place, kept.gps))`.
- `!kept.dive` : `dive = None` ; sur chaque photo, `photo_depth`, `photo_temp`, `photo_elapsed`, `gps_altitude`, `depth_m` à `None` (l'identifiant et `clock_offset` restent, le plug-in les ignore).
- `!kept.collections` : `collection = None`.
- Un groupe sans plongée, sans lieu et sans collection est retiré. `is_empty` : plus aucun groupe.

Règles de `TagRequest::keep` : `location` comme ci-dessus ; `!kept.dive` : `dive = None`, `selection = None`, `photos` vidé ; `!kept.collections` : `collection = None` (champ ajouté à la Task 4 ; dans cette Task, `kept.collections` est ignoré pour `TagRequest`). `is_empty` : ni plongée, ni lieu (ni collection, Task 4).

- [ ] **Step 1 : tests qui échouent.**

```rust
#[test]
fn location_keeps_only_the_ticked_half() {
    let l = LocationTags { country: Some("Maldives".into()), city: Some("Ari".into()),
        location: Some("Maaya Thila".into()), lat: Some(4.09), lon: Some(72.86) };
    let place = l.clone().keep(true, false).unwrap();
    assert_eq!((place.country.as_deref(), place.lat), (Some("Maldives"), None));
    let gps = l.clone().keep(false, true).unwrap();
    assert_eq!((gps.country, gps.city, gps.location, gps.lat), (None, None, None, Some(4.09)));
    assert!(l.clone().keep(false, false).is_none());
    let no_gps = LocationTags { lat: None, lon: None, ..l };
    assert!(no_gps.keep(false, true).is_none()); // Review Focus 3
}

#[test]
fn keep_all_changes_nothing() {
    let mut r = sample_collections_request(); // deux groupes, photos avec profondeur et altitude
    let before = serde_json::to_value(&r).unwrap();
    r.keep(Kept::ALL, None);
    assert_eq!(serde_json::to_value(&r).unwrap(), before);
}

#[test]
fn every_combination_keeps_only_what_is_ticked() {
    for bits in 0u8..16 {
        let kept = Kept { dive: bits & 1 != 0, place: bits & 2 != 0, gps: bits & 4 != 0, collections: bits & 8 != 0 };
        let mut r = sample_collections_request();
        r.keep(kept, None);
        if bits == 0 { assert!(r.is_empty()); continue; }
        for g in &r.groups {
            assert_eq!(g.dive.is_some(), kept.dive);
            assert_eq!(g.collection.is_some(), kept.collections);
            assert_eq!(g.location.as_ref().is_some_and(|l| l.country.is_some()), kept.place);
            assert_eq!(g.location.as_ref().is_some_and(|l| l.lat.is_some()), kept.gps);
            assert_eq!(g.photos.iter().any(|p| p.photo_depth.is_some() || p.gps_altitude.is_some()), kept.dive);
        }
    }
}

#[test]
fn another_site_replaces_every_groups_location_not_its_collection() { /* other_site = Some(site « Plage ») :
    chaque groupe a location.location == Some("Plage"), collection inchangé */ }
```

Ajouter les mêmes trois derniers tests pour `TagRequest` (8 combinaisons de `dive`, `place`, `gps`).

- [ ] **Step 2 :** `cargo test -p divelens-core tags::` → échec de compilation (`Kept` inconnu).
- [ ] **Step 3 : implémenter** les signatures ci-dessus. `collections/plan.rs` de l'app et les tests existants s'adaptent seulement en enveloppant les trois champs dans `Some(...)`.
- [ ] **Step 4 :** `cargo test -p divelens-core`, `cargo clippy -p divelens-core -p libdivecomputer-sys --all-targets -- -D warnings`, puis `cargo test -p divelens` (le JSON envoyé au plug-in est identique quand tout est `Some` : les tests du plug-in passent sans modification).
- [ ] **Step 5 : commit, push, pull request.**

### Task 3 : le plug-in accepte un groupe sans plongée ou sans collection (protocole 7)

Branche `feat/plugin-partial-groups`, titre `feat(plugin): write groups without a dive or a collection`.

**Files:**
- Modify: `plugin/DiveLens.lrdevplugin/Tagging.lua` (`planGroup`, `planCollections`, `collectionWrites`), `ApplyCollections.lua`, `Link.lua` (`PROTOCOL = 7`)
- Modify: `app/src-tauri/src/lightroom.rs` (`PROTOCOL = 7`, `WrittenCollection.collection: Option<String>`), `app/src-tauri/src/collections.rs` et `app/src/types.ts` (le type suit)
- Test: `app/src-tauri/tests/plugin_collections.rs`, `plugin_retagging.rs`

**Interfaces:**
- Consumes: la forme JSON de `CollectionGroup` de la Task 2 (champs absents quand `None`).
- Produces, côté Lua :
  - `planGroup` rend `{ collection = nom ou nil, common = changements, photos = { { id, changes = changements ou nil } }, writesDive = booléen }`. Sans `dive` : `changes = nil` pour chaque photo (ses valeurs propres sont ignorées), seul `id` est exigé. Sans plongée, sans lieu et sans collection : `nil, "nothing to write"`.
  - `Tagging.collectionWrites` : pour une photo sans `changes`, `photoWrites(g.common, nil, nil)` : rien n'est lu ni effacé. `results[gi] = { collection = nom ou nil, count, created }`, `created` toujours faux sans nom.
  - `ApplyCollections.run` : `before` ne lit que les photos des groupes où `writesDive` ; ne crée de collection que pour un groupe qui a un nom ; ne cherche ni ne crée l'ensemble (`place.group`) que si au moins un groupe a un nom.

- [ ] **Step 1 : tests qui échouent** dans `plugin_collections.rs`, sur le modèle de `collections_plan_writes_dive_data_and_per_photo_values` :
  - `a_group_without_a_dive_writes_only_the_location` : groupe `{ collection, location, photos = { { id = "1", clockOffset = "+04:00", photoDepth = "12.0 m" } } }` → les changements de la photo ne contiennent que `country`, `city`, `location`, `gps` ; aucun champ `kind = "plugin"`.
  - `a_group_without_a_collection_creates_none` : `results[1].collection == nil`, `created == false`, `count == 1`.
  - `a_group_with_nothing_is_refused` : `planCollections` rend `nil, "nothing to write"`.
  - dans `plugin_retagging.rs`, `a_group_without_a_dive_clears_nothing_of_another_dive` (Review Focus 2) : photo `before` portant une autre plongée, `PhotoDepth`, `WrittenAltitude` et l'altitude correspondante → `collectionWrites` ne rend pour elle aucun changement à valeur `nil`, et `cleared == 0`.
- [ ] **Step 2 :** `cargo test -p divelens --test plugin_collections --test plugin_retagging` → les quatre échouent (« bad dive »).
- [ ] **Step 3 : implémenter** côté Lua, puis `PROTOCOL` à 7 des deux côtés et `WrittenCollection.collection` en `Option<String>` (`CollectionsResult` et `types.ts` suivent ; l'écran actuel reçoit toujours un nom).
- [ ] **Step 4 : protocole (Review Focus 1).** Le test existant `the_plugin_speaks_the_same_protocol` passe avec 7. Vérifier qu'un test de `lightroom.rs` couvre déjà « hello avec un autre protocole → état redémarrer, aucune commande envoyée » ; sinon l'ajouter avec le faux plug-in du module : `a_plugin_with_an_older_protocol_is_refused` (hello `protocol: 6` → `Status` en état « protocol », `Command::Apply` répond `NotConnected`).
- [ ] **Step 5 :** `cargo test -p divelens`, `cargo clippy -p divelens --all-targets -- -D warnings`, depuis `app/` : `npm test`, `npm run build`.
- [ ] **Step 6 : commit, push, pull request.**

### Task 4 : une collection depuis la sélection

Branche `feat/plugin-selection-collection`, titre `feat(plugin): put the selection in the dive's collection`.

**Files:**
- Modify: `core/src/tags.rs` (`CollectionPlace`, `TagRequest.collection`), `plugin/DiveLens.lrdevplugin/Tagging.lua`, `CatalogWrite.lua`, `ApplyTags.lua`, `ApplyCollections.lua`, `app/src-tauri/src/lightroom.rs`
- Test: `app/src-tauri/tests/plugin_tag_selection.rs`, `core/src/tags.rs`

**Interfaces:**
- Produces :
  - `core` : `pub struct CollectionPlace { pub name: String, pub parent: Vec<String>, pub group: Option<String> }` (camelCase) ; `TagRequest.collection: Option<CollectionPlace>` (absent quand `None`). `TagRequest::keep` et `is_empty` en tiennent compte (Task 2).
  - Lua : `Tagging.plan(request, formatDate)` accepte une demande sans plongée ni lieu si `request.collection` est présent (rend `{}`) ; « nothing to write » seulement quand les trois manquent.
  - Lua : `Tagging.planCollection(request)` → `nil` sans collection ; `{ name, parent = { ... }, group = nom ou nil }` ; ou `nil, "bad collection"` (nom vide, parent non textuel), mêmes contrôles que `planCollections`.
  - Lua : `CatalogWrite.holder(catalog, place, fail)` → `holder, container` ou `nil` après avoir appelé `fail` : le code de recherche et de création de l'ensemble sorti tel quel d'`ApplyCollections.run`, utilisé par les deux.
  - Lua : `Tagging.selectionCollection(name, count, existing)` → `{ collection = name, count = count, created = not existing[name] and count > 0 }`.
  - Message `applied` : `{ type = "applied", id, count, collection = { collection, count, created } ou absent }`. Rust : `FromPlugin::Applied { id, count, collection: Option<WrittenCollection> }` ; la réponse de `Command::Apply` devient `TagWritten { count: usize, collection: Option<WrittenCollection> }`.
- `ApplyTags.run`, ordre : `plan` → `planPhotos` → `planCollection` → sélection (`noSelection`) → `selectionRefusal` → `CatalogWrite.holder` (refus `parentNotFound` avant toute écriture de photo, Review Focus 4) → `CatalogWrite.apply`. Dans `writeFirst`, après les photos : `createCollection(name, container, true)` puis `addPhotos(photos)` (toutes les photos sélectionnées ; Lightroom ignore celles déjà présentes, Review Focus 5).

- [ ] **Step 1 : tests qui échouent** dans `plugin_tag_selection.rs` :
  - `a_request_with_only_a_collection_is_planned` : `Tagging.plan({ collection = { name = "707 - Maaya Thila", parent = {} } }, f)` rend une liste vide, sans raison.
  - `a_request_with_nothing_is_still_refused` : `nil, "nothing to write"`.
  - `the_collection_of_a_request_is_checked` : nom vide → `nil, "bad collection"` ; `parent = { "Voyages" }, group = "Dive Lens"` rendu tel quel ; `group = ""` → `nil`.
  - `a_collection_already_there_is_completed_not_created` : `selectionCollection("707 - Maaya Thila", 5, { ["707 - Maaya Thila"] = true }).created == false` ; sans elle, `true` ; `count = 0` → `false`.
  - `core` : `tag_request_serialises_its_collection_only_when_present`.
- [ ] **Step 2 :** `cargo test -p divelens --test plugin_tag_selection` → échecs.
- [ ] **Step 3 : implémenter.** `ApplyCollections.run` appelle `CatalogWrite.holder` sans autre changement (ses tests ne bougent pas). Côté app, `tag_selection` lit `written.count` et ignore encore `written.collection` (Task 5).
- [ ] **Step 4 :** étendre le test du faux plug-in de `lightroom.rs` qui couvre `applied` : une réponse avec `collection` arrive en `TagWritten { count, collection: Some(..) }`, une réponse sans, en `None`.
- [ ] **Step 5 :** `cargo test -p divelens-core`, `cargo test -p divelens`, les deux clippy, `npm test`, `npm run build`.
- [ ] **Step 6 : commit, push, pull request.** La PR dit que la création de collection depuis une sélection n'est vérifiable dans Lightroom qu'en phase 3 (aucun écran ne l'envoie encore).

### Task 5 : options du chemin « sélection » dans le moteur

Branche `feat/selection-write-options`, titre `feat(app): take write options when tagging the selection`.

**Files:**
- Create: `app/src-tauri/src/apply/options.rs`
- Modify: `app/src-tauri/src/apply.rs` (`pub(crate) mod options;`), `app/src-tauri/src/tag.rs`, `tag/rules.rs`, `app/src-tauri/src/apply/errors.rs` (aucune nouvelle erreur : `NothingChosen` sert), `app/src/types.ts`, `app/src/api.ts`
- Test: `app/src-tauri/src/apply/options.rs`, `tag/rules.rs`

**Interfaces:**
- Consumes: `Kept`, `TagRequest::keep`, `is_empty`, `CollectionPlace` (Tasks 2 et 4), `divelens_core::collections::collection_name`.
- Produces :

```rust
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WriteOptions {
    pub dive: bool,
    /// `None` : le réglage « Profondeur copiée dans l'altitude » décide (écrans actuels).
    pub altitude: Option<bool>,
    pub place: bool,
    pub gps: bool,
    pub collections: bool,
}
impl WriteOptions {
    pub(crate) fn kept(&self) -> Kept;
    /// Vrai seulement si `dive` ; `setting` n'est lu que si `altitude` est `None`.
    pub(crate) fn copies_altitude(&self, setting: impl FnOnce() -> bool) -> bool;
}
```

  - `TagChoice` gagne `#[serde(default)] options: Option<WriteOptions>`. `None` (écrans actuels) : comportement d'aujourd'hui, chemin de code inchangé.
  - Avec des options : `dive_id` désigne la plongée choisie même quand `options.dive` est faux (elle sert au site et au nom de la collection) ; `dive_id = None` est « Aucune plongée ».
  - `rules::build_request` gagne deux paramètres en fin de liste : `kept: Kept` et `collection: Option<CollectionPlace>`. Il construit la demande comme aujourd'hui, pose `collection`, appelle `request.keep(kept)`, rend `Err(ApplyError::NothingChosen)` si `request.is_empty()`, puis `with_altitude`. Le profil n'est lu que si `kept.dive`.
  - `tag_request` : `collection = Some(CollectionPlace { name: collection_name(&dive), parent, group })` quand `options.collections` et qu'une plongée est choisie, `parent` et `group` lus dans `Settings.collections` comme `apply_collections` le fait.
  - `TagApplied` gagne `collection: Option<WrittenCollection>` (camelCase, `null` aujourd'hui).
  - `types.ts` : `WriteOptions`, champ facultatif `options` de `TagChoice`, `collection` de `TagApplied`. Aucun écran ne les utilise.

- [ ] **Step 1 : tests qui échouent.**
  - `options.rs` : `the_altitude_follows_the_setting_only_when_unset` (`altitude: None` + réglage vrai → vrai ; `Some(false)` + réglage vrai → faux, et la closure du réglage n'est pas appelée ; `dive: false` → toujours faux).
  - `tag/rules.rs`, avec les aides `dive()` et `site()` existantes :
    - `without_options_the_request_is_todays` : `Kept::ALL` et `collection = None` → demande identique, champ par champ, à celle du test existant le plus proche.
    - `dive_details_unticked_keep_the_dives_site_and_collection` : `kept.dive = false`, lieu `DiveSite`, collection donnée → `request.dive.is_none()`, `request.photos.is_empty()`, `request.location` est le site de la plongée, `request.collection` présent ; la closure `profile` n'est jamais appelée.
    - `gps_alone_on_a_site_without_gps_is_refused` (Review Focus 3) : `kept = { dive: false, place: false, gps: true, collections: false }`, site sans GPS → `Err(ApplyError::NothingChosen)`.
    - `no_dive_and_another_site_writes_the_place_only` : `dive = None`, lieu `Site`, `kept` avec `collections: true` → pas de collection dans la demande (aucune plongée), lieu écrit.
- [ ] **Step 2 :** `cargo test -p divelens options:: tag::rules::` → échecs.
- [ ] **Step 3 : implémenter.**
- [ ] **Step 4 :** `cargo test -p divelens` (dont `tests/commands.rs`, qui compare `api.ts` à `lib.rs`), clippy de l'app, `npm test`, `npm run build`.
- [ ] **Step 5 : commit, push, pull request.**

### Task 6 : options du chemin « catalogue » dans le moteur, et toutes les combinaisons de bout en bout

Branche `feat/catalogue-write-options`, titre `feat(app): take write options when creating collections`.

**Files:**
- Modify: `app/src-tauri/src/collections.rs`, `app/src/types.ts`, `app/src/api.ts`
- Create: `app/src-tauri/tests/plugin_options.rs`
- Test: `app/src-tauri/tests/plugin_options.rs`, `app/src-tauri/src/collections.rs`

**Interfaces:**
- Consumes: `WriteOptions` (Task 5), `CollectionsRequest::keep` / `is_empty` (Task 2), `tag::LocationChoice::Site` et `tag::rules::{check_gps, find_site}` (rendus `pub(crate)`), les formes de groupe de la Task 3.
- Produces :
  - `apply_collections(app, log, search, choices, options: Option<WriteOptions>, other_site: Option<LocationChoice>)`. `None, None` : chemin d'aujourd'hui.
  - Ordre avec options : demande construite comme aujourd'hui (profils lus comme aujourd'hui : une courbe illisible fait toujours échouer, même « Détails de la plongée » décochée) → `parent` et `group` → autre site (`LocationChoice::DiveSite` ou `None` : site de chaque plongée ; `Site` : `check_gps`, lecture des sites du carnet, `find_site`, `LocationTags::new(&site, lat, lon)`) → `request.keep(options.kept(), other)` → `NothingToWrite` si `request.is_empty()` → altitude si `options.copies_altitude(...)`.
  - `CollectionsResult` : `collections` ne liste que les groupes qui ont un nom ; `photos` reste la somme des photos écrites de tous les groupes.

- [ ] **Step 1 : test de bout en bout qui échoue**, `tests/plugin_options.rs`, avec `common::lua()`, `json_of` et `sample_dive()` :

```rust
/// Chaque combinaison d'options de l'étape 4, chemin « catalogue » : la demande
/// que `keep` produit, planifiée par le plug-in, n'écrit que ce qui est coché.
#[test]
fn every_option_combination_writes_only_what_is_ticked_catalogue() {
    for bits in 1u8..32 {
        let (dive, altitude, place, gps, collections) =
            (bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0, bits & 16 != 0);
        // demande de référence : 1 groupe, 2 photos (une avec profondeur 12 m, une en surface)
        // altitude && dive → copy_depth_to_altitude() ; puis keep(Kept{..}, None)
        // si is_empty() : continue (cas « altitude seule », refusé par le moteur)
        // Lua : groups = Tagging.planCollections(req, f) ; plan = Tagging.collectionWrites(groups résolus, {}, {})
        // assertions sur l'union des champs de plan.groups[1].steps et plan.later :
        //   StartDate présent            <=> dive
        //   PhotoDepth présent (photo 1) <=> dive
        //   gpsAltitude et WrittenAltitude présents (photo 1) <=> dive && altitude
        //   country présent              <=> place
        //   gps présent                  <=> gps
        //   plan.results[1].collection ~= nil <=> collections
    }
}
```

  Même test pour le chemin « sélection » (`every_option_combination_writes_only_what_is_ticked_selection`) avec `TagRequest`, `Tagging.plan`, `planPhotos`, `tagWrites` et `planCollection`, sur les 31 combinaisons ; la photo hors de la plongée ne reçoit jamais `PhotoDepth`.
- [ ] **Step 2 :** `cargo test -p divelens --test plugin_options` → compile et passe déjà pour ce que les Tasks 2 à 4 ont livré ; tout écart trouvé ici est corrigé dans la Task d'origine (nouveau commit sur cette branche, signalé dans la PR). Les tests d'`apply_collections` ci-dessous, eux, échouent.
- [ ] **Step 3 : tests unitaires** dans `collections.rs` (extraire la suite « options » de la commande dans une fonction pure `fn with_options(request: &mut CollectionsRequest, options: &WriteOptions, other: Option<&LocationTags>, setting: impl FnOnce() -> bool) -> Result<(), ApplyError>` pour la tester) :
  - `all_options_and_the_setting_give_todays_request`.
  - `another_site_goes_on_every_group`.
  - `nothing_ticked_is_refused` → `Err(ApplyError::NothingToWrite)`.
- [ ] **Step 4 : implémenter**, `types.ts` et `api.ts` (paramètres facultatifs, non utilisés par l'écran).
- [ ] **Step 5 :** `cargo test -p divelens`, clippy de l'app, `npm test`, `npm run build`.
- [ ] **Step 6 : commit, push, pull request.**

### Task 7 : taille du catalogue

Branche `feat/catalogue-size`, titre `feat(link): report the number of photos in the catalogue`.

**Files:**
- Modify: `plugin/DiveLens.lrdevplugin/Link.lua`, `Selection.lua`, `app/src-tauri/src/lightroom.rs`, `app/src-tauri/src/lib.rs`, `app/src/types.ts`, `app/src/api.ts`
- Test: `app/src-tauri/src/lightroom.rs` (faux plug-in), `app/src-tauri/tests/plugin_tools.rs`

**Interfaces:**
- Produces :
  - App → plug-in : `{ type = "countPhotos", id }`. Plug-in → app : `{ type = "catalogSize", id, count }`, `count = #catalog:getAllPhotos()` (jamais le comptage par dossier). Traité dans la même table de gestionnaires que `describeSelection` : `Selection.count(s, message)`.
  - Rust : `Command::CountPhotos { reply }`, `ToPlugin::CountPhotos { id }`, `FromPlugin::CatalogSize { id, count: u64 }`. `Status` gagne `catalog_photos: Option<u64>` (camelCase `catalogPhotos`), remis à `None` à chaque connexion et rempli par la réponse.
  - L'app envoie `countPhotos` d'elle-même juste après un `hello` accepté.
  - Commande Tauri `refresh_catalog_size() -> Result<(), ApplyError>` : renvoie `countPhotos` (le parcours l'appellera au retour sur l'étape 1, phase 3) ; `NotConnected` sans liaison. Ajoutée à `lib.rs` et `api.ts`.
  - `types.ts` : `catalogPhotos: number | null` dans le statut Lightroom. Aucun écran ne l'affiche encore.

- [ ] **Step 1 : tests qui échouent** dans `lightroom.rs`, avec le faux plug-in du module :
  - `the_catalogue_size_is_asked_after_hello` : après le hello, le faux plug-in reçoit `countPhotos` ; il répond `catalogSize` avec `count: 48000` → `status.catalog_photos == Some(48000)`.
  - `the_catalogue_size_is_forgotten_when_the_link_drops` → `None` après `bye`.
  - `an_empty_catalogue_counts_zero` : `count: 0` → `Some(0)`, pas `None`.
  - `a_plugin_that_never_answers_leaves_the_size_unknown` : pas de réponse → `None`, et `Command::Apply` suivant est servi normalement (le comptage ne bloque rien).
- [ ] **Step 2 :** `cargo test -p divelens lightroom::` → échecs de compilation.
- [ ] **Step 3 : implémenter** des deux côtés. Dans `plugin_tools.rs`, le test existant « chaque fichier se compile » couvre `Selection.lua` ; ajouter `the_link_knows_the_count_message` : la table des gestionnaires de `Link.lua` contient `countPhotos` (même vérification que pour `describeSelection` si elle existe, sinon recherche de la ligne dans le fichier, comme `the_plugin_speaks_the_same_protocol`).
- [ ] **Step 4 :** `cargo test -p divelens`, clippy de l'app, `npm test`, `npm run build`.
- [ ] **Step 5 : essai dans Lightroom par le constructeur n'est pas possible** : le noter dans la PR. Le comptage a été mesuré à part (près de 50 000 photos en 0,10 s) ; son affichage sera vu par le propriétaire en phase 4.
- [ ] **Step 6 : commit, push, pull request.**

---

## Fin de la phase 1

- [ ] `main` contient les Tasks 1 à 7, CI verte sur macOS et Windows.
- [ ] `git diff <main avant la Task 2>..main -- app/src/**/*.tsx` est vide : aucun écran n'a changé.
- [ ] Noter dans une annexe à ce fichier (« Notes pour les phases 2 à 4 ») tout ce que l'exécution a appris : écarts trouvés par `plugin_options.rs`, noms définitifs, limites rencontrées. Le plan des phases 2 à 4 s'écrit à partir de la note de conception et de cette annexe.

---

## Notes pour les phases 2 à 4

Écrites à la fin de la phase 1 (2026-11-24), à partir de l'exécution et de ses relectures. Le plan des phases 2 à 4 part de la note de conception, de la spécification réécrite et de cette annexe.

### Ce que la phase 1 a livré, avec les noms définitifs

- `core/src/tags.rs` : `Kept { dive, place, gps, collections }`, `LocationTags::keep`, `CollectionsRequest::keep(kept, other_site)`, `TagRequest::keep(kept)`, `is_empty` sur les deux, `CollectionPlace { name, parent, group }`, `TagRequest.collection`, `CollectionGroup` aux trois champs facultatifs.
- `app/src-tauri/src/apply/options.rs` : `WriteOptions { dive, altitude: Option<bool>, place, gps, collections }` (`altitude: None` = le réglage décide ; le parcours enverra toujours `Some`).
- `app/src-tauri/src/apply/site.rs` : `another_site`, `check_gps`, `find_site`, pour les deux chemins.
- Commandes : `tag_selection` prend `TagChoice.options` ; `apply_collections` prend `options` et `otherSite` ; `refresh_catalog_size` (sans attente) ; le statut Lightroom porte `catalogPhotos`.
- Réponses : `TagApplied.collection` (la collection de la sélection) ; `CollectionsResult.collections` ne liste que les groupes nommés.
- Plug-in : `Tagging.planCollection`, `selectionCollection`, `namesCollections`, `divePhotos` ; `CatalogWrite.holder` ; `Selection.count`. Protocole 7.
- `tests/plugin_options.rs` : toutes les combinaisons d'options, pour chaque chemin, comparées à la liste exacte de ce qui est écrit et effacé.

### Décisions prises en route

- Sans aucune collection demandée, le plug-in ne cherche aucun ensemble de collections : un ensemble introuvable ne bloque « Appliquer » que si « Créer les collections » est cochée.
- Chemin « sélection », « Détails de la plongée » décochée : les décalages des appareils qui ont une photo dans la plongée restent mémorisés ; la courbe n'est pas lue. Une description de sélection que l'app n'a plus fait refuser « Appliquer », comme avec la case cochée.
- Chemin « catalogue », avec des options : les courbes sont lues comme aujourd'hui (une courbe illisible fait échouer « Appliquer », même « Détails de la plongée » décochée).
- `otherSite` sans `options` est appliqué, tout coché.
- Sans option envoyée, les deux commandes prennent exactement le chemin d'avant la phase 1. La phase 3 supprime ce chemin avec les anciens écrans, et avec lui la lecture du réglage d'altitude.

### À traiter dans le plan des phases 2 à 4

- **Récapitulatif (étape 5)** : avec des options, un groupe auquel il ne reste rien à écrire est retiré de la demande sans apparaître ni dans `collections` ni dans `empty_dives`. Le récapitulatif en a besoin. Avec « Créer les collections » décochée, aucune collection n'est renvoyée : l'écran ne doit pas afficher de nom vide.
- **`gpsNotWritten`** : la réponse du plug-in perd alors le résultat des collections (les deux chemins). À corriger avant que le récapitulatif n'en dépende.
- **Chemin « sélection », plongée décochée** : la demande ne porte plus le numéro de la sélection décrite ; le plug-in ne vérifie donc plus que la sélection est celle dont viennent les décalages mémorisés. À décider avec l'écran de l'étape 4.
- **Sites lus pour rien** : avec « Autre site » et ni lieu ni GPS cochés, le carnet est quand même lu et peut refuser « site inconnu ». L'écran grisera sans doute le choix du site dans ce cas.
- **Textes** : le refus « La sélection a changé… vérifiez les données par photo » s'affichera à l'étape 4, où ces données ne sont pas visibles ; « Appliquer grisé pendant la lecture de la sélection » concerne maintenant l'étape 3. Deux textes à soumettre au propriétaire.
- **Rangement** : `lightroom.rs` a grossi (environ 400 lignes, surtout des tests) : le découper avant d'y ajouter ; `tag/rules.rs` et `collections.rs` deviendront un seul module « parcours ».
- **Petits tests à ajouter** : un groupe retiré pendant qu'un autre reste ; une réponse `collectionsApplied` avec un groupe sans nom lue côté Rust ; la commande `refresh_catalog_size` elle-même ; renommer `tag_request_another_site_is_not_applied_here` et `unknown_dive_or_site_is_refused` ; reformuler l'ancienne assertion `{ collection = "x" }` de `plugin_collections.rs`, qui passe désormais pour une autre raison ; `another_site_alone_means_everything_ticked` avec le réglage d'altitude désactivé.
- **Faire des décisions testées de deux gardes d'enveloppe** : `collectionWrites` pourrait dire quels groupes créent une collection, au lieu du test `g.collection and #group.photos > 0` dans `ApplyCollections.lua` et de son équivalent dans `ApplyTags.lua`.

### À vérifier dans Lightroom (phase 4, vérifications et recette)

- Chemin « sélection » avec collection : la collection existante est complétée, sans doublon, et annoncée « complétée » ; les noms sont comparés exactement.
- Première utilisation de l'ensemble « Dive Lens » dans un catalogue, sur les deux chemins : sa création est une étape d'annulation à part ; si l'écriture est ensuite refusée, l'ensemble vide reste. À dire dans le guide.
- « GPS » seul coché : la première écriture ne change rien, puis l'écriture du GPS ; un seul « Annuler » doit tout défaire.
- Taille du catalogue : affichée, égale à « Toutes les photos » de Lightroom, recomptée au retour sur l'étape 1.
- Un plug-in resté en protocole 6 face à l'app en 7 : état « redémarrez Lightroom ».
- Windows : « Annuler » unique, toujours en attente.

### Données d'exemple

Les tests ajoutés en phase 1 utilisent un site inventé. La spécification et ce plan citent encore l'exemple des maquettes (« 707 - Maaya Thila ») : le propriétaire dira s'il faut le remplacer.
