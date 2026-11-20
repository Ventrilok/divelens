# Réorganisation du code de Dive Lens — conception

Date : 2026-11-19. Point de départ : le rapport de complexité « Dive Lens — rapport de complexité » (Claude Docs, 2026-11-18), dont les changements sont cités ici par leur numéro (n°1 à n°18). Décisions prises par le propriétaire du produit le 2026-11-19.

## Intention

Avant d'ajouter des fonctionnalités (BlueDive, d'autres applications de plongée), mettre le code sur une base propre. « Tagger la sélection » et « Créer les collections » ont chacune leur copie du circuit « Appliquer » dans le plug-in, le moteur de l'app et l'interface, et ces copies ont commencé à diverger. Le projet est petit (≈ 23 000 lignes) et n'a pas encore de contributeur extérieur : réorganiser ne coûtera jamais moins cher.

## Objectifs

- La structure par fonctionnalité proposée par le rapport, pour le moteur, le plug-in et l'interface (arborescences en annexe).
- Un seul circuit « Appliquer » : une seule enveloppe d'écriture dans le plug-in, une seule gestion des erreurs et des décalages d'horloge dans le moteur, un seul traducteur d'erreurs, une seule barre de progression et un seul bloc « horloge des appareils » dans l'interface.
- Le plus possible de décisions testées automatiquement : les règles de « Tagger la sélection » sorties du moteur, les règles « quand Appliquer est permis » sorties des écrans, et les décisions de l'enveloppe d'écriture du plug-in déplacées vers ses règles testées.
- Corriger en chemin : n°1 (code brut affiché), n°2 (écran bloqué), n°3 (liste des valeurs relues en trois exemplaires), n°4 (unités annoncées différentes des unités écrites), n°13 (réaction unique aux pannes internes). Ajouter les tests n°5 et n°6.
- Finir par une version corrective.

## Non-objectifs

- Aucune nouvelle fonctionnalité. Aucun changement visible, hormis les corrections ci-dessus.
- Pas de préparation de BlueDive (n°7 à 9), pas de description des données produite automatiquement (n°14), pas de commande unique de vérification (n°17), pas de liste des points ouverts dans la spécification (n°18).
- Pas de faux Lightroom (n°15) : refusé (il pourrait lui-même être faux, devrait suivre les versions de Lightroom, trop lourd pour un petit plug-in). Remplacé par la recette et par le déplacement des décisions vers les règles testées (ci-dessous).
- Pas de MCP Lightroom pour l'instant : mis de côté pour plus tard.
- Rien ne bouge dans `core/` (lecteurs de carnets compris) ni dans `libdivecomputer-sys/`. Le fichier d'automatisation des versions n'est pas découpé.
- L'effacement d'altitude dans « Créer les collections » (spécification : « Proposé », 2026-11-15) reste tel qu'il est codé ; la décision est prise à part.

## Anti-objectifs (échec même si « ça marche »)

- Un changement de ce qui est écrit sur les photos, de l'ordre des deux écritures ou du comportement d'« Annuler » (un seul pas).
- Un renommage de ce que le guide des contributeurs interdit de renommer : identifiant du plug-in, identifiants des champs, ensemble « Dive Lens » et noms des collections, identifiant de l'app.
- Un changement des messages entre l'app et le plug-in ou du format du fichier de réglages. Si un message doit changer malgré tout, le numéro de protocole monte des deux côtés, et c'est signalé au propriétaire.
- Une pull request qui mêle déplacements et corrections.
- Un test supprimé, désactivé ou assoupli pour faire passer un déplacement.

## Contraintes

- Une pull request par étape, branche `refactor/<slug>` (ou `fix/`, `test/`, `docs/` selon le contenu), message au format Conventional Commits, CI verte sur macOS et Windows, fusion par squash. Push, ouverture de pull request et fusion : seulement sur accord explicite du propriétaire.
- Le numéro de version ne change pas avant la version finale : sur `main`, un changement de version déclenche une publication.
- Vérifications avant chaque commit : celles du guide des contributeurs pour la partie touchée.
- Plug-in : Lua 5.1, sans compilation. Une erreur dans du code déplacé n'apparaît qu'à l'exécution dans Lightroom ; le test « chaque fichier se compile » et la recette sont les seuls filets pour l'enveloppe. Le plug-in reste sans sous-dossiers (on n'a pas vérifié que Lightroom les charge).
- Plug-in : entre les deux écritures, rien n'est envoyé à l'app et rien ne cède la main ; même nom d'annulation pour les deux ; GPS et altitude dans la seconde.
- Moteur : style de modules Rust 2024 (`settings.rs` à côté de `settings/`, pas de `mod.rs`). Une commande Tauri déplacée devient publique dans son module ; chaque nom de commande reste unique dans l'app.
- Les tests de l'interface tournent à UTC+14 ; les heures de plongée ne bougent jamais avec le fuseau.
- Données de test fictives uniquement ; aucun catalogue Lightroom (`.lrcat`) dans le dépôt.

## Approches considérées

| Approche | Ce qu'elle donne | Ce qu'elle coûte |
|---|---|---|
| A. Défauts d'abord (version corrective), puis réorganisation | Correctifs chez les utilisateurs tout de suite | Le code corrigé est retouché en étant déplacé |
| B. Tout mêlé : chaque défaut corrigé à l'étape où son code bouge | Rien fait deux fois | Si quelque chose casse, on ne sait pas si c'est le déplacement ou la correction |
| **C. Déplacer, vérifier, fusionner, corriger (retenue)** | Chaque étape a une seule cause possible de régression | Les défauts restent chez les utilisateurs pendant la réorganisation (cas rares) |

Le rapport conseillait de réorganiser « au fil de l'eau », à la prochaine retouche de chaque partie. Le propriétaire préfère une base propre avant les nouvelles fonctionnalités. C est retenue parce qu'elle sépare les causes de régression ; B a été écartée par crainte qu'elle n'introduise de nouveaux problèmes.

## Ce qui est décidé

### Les cinq phases, dans l'ordre

**Phase 0 — Recette et référence.** Écrire la recette et préparer son matériel (ci-dessous). Le propriétaire la joue sur la version actuelle (2.1.1) : ce résultat est la référence. Un problème trouvé plus tard est comparé à cette référence avant d'être attribué à la réorganisation.

**Phase 1 — Déplacements purs, sans changement de comportement.**
- Test n°6 d'abord (température basse, note et ville passées au plug-in avec de vraies valeurs) : c'est un filet pour les déplacements, et il doit passer sur le code actuel.
- Moteur (n°10) : découpage selon l'arborescence en annexe ; règles de « Tagger la sélection » sorties dans un module testé, les tests décrivant le comportement actuel.
- Plug-in : `Link.lua` découpé en transport, suivi de la sélection, écriture de « Tagger la sélection » et écriture de « Créer les collections » (deux enveloppes encore, chacune dans son fichier). La liste des valeurs relues est déduite des listes existantes (n°3). Les décisions qui n'ont pas besoin de Lightroom quittent l'enveloppe pour les règles testées : quelle réponse envoyer à l'app selon l'issue des deux écritures, quelles photos passent dans la seconde écriture, les comptes renvoyés, quelles collections sont nouvelles ou existaient déjà. Les tests décrivent le comportement actuel.
- Tests du plug-in découpés selon les nouveaux fichiers (n°16), sans modifier un seul test.
- Interface (n°12) : dossiers par fonctionnalité selon l'annexe ; règles « quand Appliquer est permis » et texte de la barre sortis et testés ; liste des plongées partagée entre l'étape 1 et « Tagger la sélection » si les deux copies sont identiques.
- Règle de la phase : toute différence trouvée entre deux copies censées être identiques n'est pas réglée en phase 1. Elle est notée et traitée en phase 3.

**Phase 2 — Point de contrôle.** CI verte, puis recette rejouée par le propriétaire : le résultat doit être identique à la référence.

**Phase 3 — Fusion des circuits « Appliquer » (n°11).** Pour respecter « jamais déplacement et correction dans la même pull request » : d'abord une pull request de correction qui fait se comporter pareil les deux copies (n°1, n°2, différences notées en phase 1), puis une pull request de fusion qui ne change aucun comportement. *(Choix du constructeur, confirmé par la relecture.)*
- Plug-in : une seule enveloppe « deux écritures, un seul Annuler », utilisée par les deux fonctionnalités, qui répond toujours à l'app, y compris sur une erreur imprévue. C'est ce qui corrige le n°2.
- Moteur : une seule gestion des erreurs d'« Appliquer » et de la mémoire des décalages d'horloge.
- Interface : un seul traducteur d'erreurs (corrige le n°1), une seule barre de progression, un seul bloc « horloge des appareils ».
- Test n°5 : il relie, entre plug-in, moteur et interface, les raisons d'échec, les noms d'écran et le nom de l'étape « Annuler », comme le test qui vérifie déjà le numéro de protocole.
- Les différences notées en phase 1 sont réglées ici.
- Recette rejouée.

**Phase 4 — Corrections restantes et version.** n°4 (revenir à l'ancien réglage à l'écran quand son enregistrement échoue) et n°13 (se rétablir partout après une panne interne), chacun avec ses tests. Puis version corrective, lancée par le propriétaire.

### Recette

- Une liste écrite de scénarios, rangée avec le guide des contributeurs. Chaque scénario donne un point de départ, une action et ce qu'on doit voir.
- Matériel : un catalogue Lightroom de test créé pour l'occasion (jamais le vrai catalogue du propriétaire) ; une poignée de photos fictives rangées dans le dépôt, dont l'heure de prise de vue tombe pendant les plongées du carnet Subsurface fictif déjà utilisé par les tests ; ce carnet.
- Quand : en phase 0 (référence), en phase 2, après la phase 3, puis avant chaque version qui touche au plug-in ou à l'écriture. Le résultat est noté dans la pull request.
- Windows : la référence se joue avec l'installateur 2.1.1 publié. Pour les phases 2 et 3, un lancement manuel du workflow Release sur une branche autre que `main` fabrique un installateur d'essai sans rien publier.
- Le carnet fictif suffit : les plongées 101 et 102 tombent le même jour et ont toutes deux une courbe et un GPS. `core/` n'a pas à changer.
- Si la référence montre sur Windows qu'un seul « Annuler » ne défait pas les deux écritures, on s'arrête et on le signale : ce n'est pas corrigé dans ce chantier, qui interdit de toucher à l'ordre des écritures.
- Scénarios :

| # | Scénario | Ce qu'on doit voir |
|---|---|---|
| 1 | Tagger 3 photos avec une plongée | Champs Dive Lens remplis, profondeur propre à chaque photo, GPS et altitude |
| 2 | Annuler une fois après le 1 | Tout disparaît, GPS et altitude compris |
| 3 | Retagger une photo avec une autre plongée | Les valeurs de l'ancienne plongée ont disparu |
| 4 | Retagger avec la même plongée | Rien d'autre ne bouge |
| 5 | Aucune photo sélectionnée | Message clair, rien d'écrit |
| 6 | Sélection changée juste avant « Appliquer » | Refus, rien d'écrit |
| 7 | Créer les collections pour 2 plongées | Collections dans « Dive Lens », photos rangées, valeurs écrites |
| 8 | Annuler une fois après le 7 | L'écriture est défaite en un seul pas |
| 9 | Relancer « Créer les collections » | Rien en double |
| 10 | Unités en pieds, option altitude activée | Profondeurs en ft, altitude correcte |
| 11 | Windows : scénarios 2 et 8 | Un seul « Annuler » suffit |

### Décisions visibles réservées au propriétaire

- Les textes affichés pour les cas qui montraient du code brut (n°1), à valider dans la pull request de la phase 3. Au même moment : les messages d'échec de recherche de photos qui parlent d'« Annuler » et de « réappliquer » alors que rien n'a été écrit (point ouvert depuis le 2026-11-14).
- Tout autre changement visible découvert en route est signalé, jamais décidé par le constructeur.

## Laissé au constructeur

- Les frontières exactes entre fichiers, à l'intérieur des arborescences de l'annexe, et les noms internes.
- L'ordre des pull requests à l'intérieur d'une phase, et leur taille, à condition qu'aucune ne mêle déplacement et correction.
- La façon de fabriquer les photos fictives (images neutres de petite taille, date de prise de vue dans l'EXIF) et l'emplacement exact de la recette et de son matériel.
- Le regroupement des trois copies de la lecture/écriture sûre des petits fichiers du moteur : en phase 1 si elles sont identiques, sinon en phase 3.

## Choix faits par le constructeur

- Phase 3 en deux pull requests : correction d'abord, fusion ensuite (voir la phase 3).
- Les chemins de fichiers cités dans `CONTRIBUTING.md` (par exemple `CURRENT_VERSION` dans `settings.rs` et `offsets.rs`) sont mis à jour dans la pull request qui déplace le fichier. `CLAUDE.md` (qui cite le fichier de tests du plug-in découpé par le n°16) n'est modifié qu'avec l'accord du propriétaire, qui y a des modifications en cours.
- Accord du propriétaire demandé avant chaque push, ouverture de pull request et fusion.

## Annexe — arborescences visées (reprises du rapport)

Moteur de l'app :

```text
app/src-tauri/src/
├─ main.rs            point d'entrée, inchangé
├─ lib.rs             démarrage et liste des commandes, rien d'autre
├─ settings.rs        Réglages : commandes
├─ settings/
│  ├─ file.rs         fichier de réglages, reprise du plug-in 0.x
│  └─ units.rs        unités lues dans l'application de plongée
├─ dive_log.rs        carnet + verrou « pas de changement pendant une écriture »
├─ tag.rs             Tagger la sélection : commandes
├─ tag/
│  ├─ rules.rs        règles sorties pour être testées
│  └─ selection.rs    description de la sélection
├─ collections.rs     Créer les collections : commandes
├─ collections/
│  └─ plan.rs         répartition des photos, comptes
├─ apply.rs           commun aux deux fonctionnalités
├─ apply/
│  ├─ errors.rs       erreurs communes
│  ├─ searches.rs     recherches numérotées, prises de vue
│  └─ offsets.rs      décalages d'horloge mémorisés après écriture
├─ update.rs          mises à jour : commandes
├─ update/
│  └─ install.rs      remplacement du dossier
├─ lightroom.rs       lien avec le plug-in
├─ lightroom/
│  └─ events.rs       événements reçus de Lightroom
├─ files.rs           lecture/écriture sûre des petits fichiers
└─ applog.rs          journal
```

Plug-in Lightroom (à plat) :

```text
plugin/DiveLens.lrdevplugin/
├─ Info, Init, Shutdown, Disable, Menu…, Launch   inchangés
├─ Metadata, Tagset         champs Dive Lens, inchangés
├─ Link.lua                 le lien seulement : connexions, jeton, aiguillage des messages
├─ Selection.lua            suivi de la sélection et vignettes
├─ CatalogWrite.lua         « deux écritures, un seul Annuler », en un seul exemplaire (phase 3)
├─ ApplyTags.lua            écriture de Tagger la sélection
├─ ApplyCollections.lua     écriture de Créer les collections
├─ Tagging.lua              règles testées (+ heures de prise de vue par appareil)
└─ Json, Framing, Log       outils
```

Interface :

```text
app/src/
├─ main.tsx, App.tsx       démarrage, navigation
├─ api.ts, types.ts, i18n.ts
├─ collections/            Créer les collections
│  ├─ DivesStep.tsx        étape 1 (choix des plongées)
│  ├─ CollectionsScreen.tsx
│  └─ rules.ts + test      quand « Appliquer » est permis, ce que dit la barre
├─ tag/                    Tagger la sélection
│  ├─ TagScreen.tsx, SitePicker.tsx
│  └─ rules.ts + test
├─ settings/               Réglages, écran d'erreur du carnet, sauvegarde
├─ updates/                bandeau de mise à jour
├─ dives/                  commun : liste + filtres + fiche de plongée, profil, carte
├─ apply/                  commun : barre de progression, messages d'erreur (un seul),
│                          décalages par appareil
└─ ui/                     icônes, étoiles, étapes, menu latéral, formats
```
