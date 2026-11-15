# Dive Lens — Spécification de la refonte, V1

Statut : validé par le propriétaire, base de développement · Date : 2026-11-11 · Rédigée en français (document produit, validé par le propriétaire)

Ce document décrit **quoi** fait la V1 de la refonte. Il ne décrit pas l'implémentation détaillée ni la mise en page des écrans (les maquettes viennent après, à partir de l'inventaire des écrans, section 6).

Chaque élément est marqué :

- **[Décidé]** — validé par le propriétaire.
- **[Proposé]** — choix courant fait par défaut, à contester si besoin.
- **[À trancher]** — décision produit ouverte, listée en section 10.

---

## 1. Objectif

Remplacer les fenêtres Lightroom actuelles du plugin par une application dédiée, moderne et réactive, qui devient l'**unique point d'entrée** de Dive Lens, tout en préparant l'ajout progressif d'autres carnets de plongée que MacDive.

La V1 apporte en plus une fonctionnalité nouvelle : la **profondeur, la température et le temps écoulé au moment de chaque photo**.

## 2. Périmètre V1

| Inclus | Exclu (plus tard) |
|---|---|
| Sources : **MacDive et Subsurface** **[Décidé]** | BlueDive, Submersion |
| **macOS (Intel et Apple Silicon) et Windows** **[Décidé]** ; MacDive sur Mac seulement, Subsurface sur les deux | Signature Windows (après la première version publique, section 9) |
| Les trois usages actuels du plugin, repensés | Notion de « voyage » |
| Données par photo (profondeur, température, temps écoulé) | Ajustement visuel sur la courbe de plongée |
| Décalage d'horloge par appareil, avec suggestion | Calage automatique par la profondeur enregistrée par l'appareil |
| **Plugin gratuit** : plus de licence ni de mode d'essai | Nettoyage automatique lors d'une relance |
| | Consultation du carnet de plongée avec photos |
| | Effacer les métadonnées Dive Lens des photos sélectionnées **[Décidé, 2026-11-12]** |

**libdivecomputer est intégré dès la V1 [Décidé, 2026-11-11]** : une vraie base MacDive ne contient pas de courbe lisible, seulement une copie chiffrée propre à MacDive et le téléchargement brut de l'ordinateur de plongée. libdivecomputer (la bibliothèque de Subsurface) décode ce téléchargement. La courbe lisible observée au départ avait été ajoutée à la base de test par un outil externe (DiverCity).

## 3. Architecture

**[Décidé]**

```
Lightroom Classic
 └─ Plugin Dive Lens (Lua, invisible)
     - déclare les champs Dive Lens
     - lit le catalogue (photos, sélection courante) et y écrit
     - lance l'application
            ▲
            │ liaison locale : lecture en direct, écriture sur « Appliquer »
            ▼
Application Dive Lens (Rust + Tauri)
 - seule interface utilisateur
 - lecteurs de carnets de plongée (MacDive et Subsurface en V1) → format commun
```

- **Ouverture** : deux éléments de menu dans Lightroom (Bibliothèque › Modules externes), qui ne sont que des **raccourcis** : chacun ouvre l'application directement sur l'écran correspondant. Aucune fenêtre Lua. Lightroom doit rester ouvert. **[Décidé]**
  - « Créer les collections… » (EN « Create Collections… ») → F5
  - « Tagger la sélection… » (EN « Tag Selection… ») → F8, actif seulement si des photos sont sélectionnées
- **Livraison** : un seul paquet ; l'application est rangée dans le dossier du plugin. **[Décidé]**
  - **Paquet [Décidé, 2026-11-15]** : un zip par système, `DiveLens-<version>-macOS.zip` (application universelle, Intel et Apple Silicon) et `DiveLens-<version>-Windows.zip`. Chacun contient le dossier `DiveLens.lrplugin`, le nom de l'ancien plugin, avec l'application dedans et la licence. Il est fabriqué automatiquement quand un nouveau numéro de version arrive sur `main` et déposé en **brouillon** sur GitHub Releases : le propriétaire le relit et le publie à la main.
  - **Publication [Décidé, 2026-11-15]** : une commande sur l'ordinateur du propriétaire (`npm run release -- patch`, `minor`, `major` ou un numéro) écrit le nouveau numéro partout, puis ouvre la pull request « build: release <numéro> », fusionnée d'elle-même quand les vérifications sont au vert. À la fusion, GitHub fabrique, signe et notarise la version, puis crée le brouillon « Dive Lens <numéro> » avec la liste des changements tirée des pull requests fusionnées. Publier le brouillon crée l'étiquette `v<numéro>`. GitHub ne modifie jamais le code ; une version publiée n'est jamais refabriquée. Une version avec étiquette (`2.1.0-beta.1`) est marquée comme préversion.
  - **Installateurs [Décidé, 2026-11-15]** : en plus du zip, gardé pour installer à la main, un installateur par système : `DiveLens-<version>-macOS.pkg` (signé et notarisé) et `DiveLens-<version>-Windows-Setup.exe` (non signé pour l'instant, section 9). Il installe **pour l'utilisateur seul**, sans mot de passe administrateur, le dossier `DiveLens.lrplugin` dans le dossier « Modules » de Lightroom de cet utilisateur (`~/Library/Application Support/Adobe/Lightroom/Modules` sur Mac, `%APPDATA%\Adobe\Lightroom\Modules` sur Windows), où Lightroom le charge tout seul : rien à ajouter dans le gestionnaire de modules externes. Raison du choix « utilisateur seul » : les fichiers appartiennent à l'utilisateur, la mise à jour automatique (F13) peut donc les remplacer sans mot de passe. L'installateur ne sert qu'à la première installation ; les mises à jour passent par l'application.
    - Une réinstallation part d'un dossier vide : aucun fichier d'une version précédente ne reste. Installer une version plus ancienne par-dessus une plus récente remplace aussi l'application.
    - Écrans de l'installateur **[Décidé, 2026-11-15]** : ceux du système, sans écran de licence, plus un message de fin en anglais ou en français selon le système : « Dive Lens est installé. Si Lightroom Classic est ouvert, quittez-le et relancez-le : Lightroom ne charge Dive Lens qu'à son démarrage. Les menus Dive Lens sont dans Bibliothèque › Modules externes. » (EN « Dive Lens is installed. If Lightroom Classic is open, quit it and start it again: Lightroom loads Dive Lens only when it starts. The Dive Lens menus are in Library › Plug-in Extras. »). Sur Windows, l'installateur suit la langue du système et ajoute une ligne « Dive Lens » dans la liste des applications, pour le désinstaller.
    - **Ancien module 0.x [Décidé, 2026-11-15]** : l'installateur ne cherche pas l'ancien module. Le guide demande de le retirer d'abord (Gestionnaire de modules externes › Supprimer), puis d'installer ; avec le zip, on remplace l'ancien dossier au même endroit, comme avant. Raison : Lightroom note les modules ajoutés à la main dans ses préférences, à un endroit différent sur Mac et Windows et non documenté ; les y chercher serait fragile. Les réglages de l'ancien module restent repris : Lightroom les range sous l'identifiant du module, qui ne change pas. À vérifier dans Lightroom : ce qui se passe si les deux sont présents, et que retirer l'ancien module ne perd pas ses réglages.
  - Windows : l'application a besoin de WebView2, présent sur Windows 11 et sur la plupart des Windows 10.
- **Aucune fenêtre Lua.** Le gestionnaire de modules externes de Lightroom affichera au minimum la version et un bouton pour ouvrir Dive Lens (Lightroom impose cette entrée). **[Décidé, 2026-11-15]**
- **Liaison** : l'application suit en direct la sélection de photos dans Lightroom. Rien n'est écrit dans le catalogue avant que l'utilisateur clique sur **Appliquer** ; ce qu'il écrit est alors **annulable en une seule étape** dans Lightroom. **[Décidé]**
  - **GPS et altitude dans une seconde écriture [Décidé, 2026-11-14]** : « Appliquer » (F5, F8) écrit d'abord, pour toutes les photos, les champs Dive Lens, le pays, la ville, le lieu et les collections ; puis, aussitôt après et sous le même nom d'annulation, une seconde écriture pose le GPS et l'altitude (écriture comme effacement, y compris l'altitude effacée en rattachant une photo à une autre plongée, F8) pour toutes les photos. Sans GPS ni altitude à écrire, il n'y a pas de seconde écriture. Le repère d'altitude (F7), champ Dive Lens, va dans la première écriture avec les autres champs Dive Lens, jamais dans la seconde **[Décidé, 2026-11-15]**. Raison : Lightroom ne défait pas le GPS et l'altitude écrits dans la même écriture qu'un champ de lieu texte (section 9). Sur Mac, les deux écritures forment une seule étape d'annulation : un « Annuler » défait tout, GPS et altitude compris.
  - Rien entre les deux écritures, et rien pendant la seconde, ne rend la main à Lightroom : Mac ne les regroupe que s'il n'y a eu aucune action de l'utilisateur entre elles. La progression affichée s'arrête donc à la fin de la première écriture ; la seconde, courte, n'en donne pas.
  - **Windows** : ce regroupement en une seule étape n'est documenté par Adobe que pour macOS. Les testeurs Windows vérifient qu'un seul « Annuler » défait tout, GPS et altitude compris.
  - Si la seconde écriture n'a pas lieu (Lightroom occupé, erreur) alors que la première a réussi, les photos ont leurs données de plongée et leur lieu, mais pas leur GPS ni leur altitude. Ce n'est pas un succès : le journal du plugin le note et l'application l'affiche **[Décidé, 2026-11-15]**, en nommant l'élément du menu Annuler tel que Lightroom le montre :
    - Lightroom occupé : « Lightroom n'a pas laissé Dive Lens écrire le GPS et l'altitude ; tout le reste a été écrit. Choisissez Édition › Annuler Tagger la sélection (Dive Lens) dans Lightroom, puis appliquez à nouveau. » (EN « Lightroom did not let Dive Lens write the GPS and altitude; everything else was written. Choose Edit › Undo Tag Selection (Dive Lens) in Lightroom, then apply again. ») ;
    - erreur : « Tout a été écrit sauf le GPS et l'altitude : {erreur}. Choisissez Édition › Annuler Tagger la sélection (Dive Lens) dans Lightroom, puis appliquez à nouveau. » (EN « Everything was written except the GPS and altitude: {error}. Choose Edit › Undo Tag Selection (Dive Lens) in Lightroom, then apply again. ») ;
    - pour « Créer les collections », les mêmes textes avec « Créer les collections (Dive Lens) » (EN « Create Collections (Dive Lens) »). Les décalages d'horloge sont mémorisés comme après une écriture complète : ils sont déjà sur les photos.
    - Annuler d'abord : appliquer à nouveau sans annuler ne referait pas l'effacement de l'altitude d'une autre plongée (F8), la photo portant déjà la nouvelle plongée.
    - Le nom de l'élément suit la langue de Lightroom, le texte celle de l'application : quand elles diffèrent, le nom cité ne correspond pas exactement au menu. Laissé tel quel **[Décidé, 2026-11-15]**.
- Si la liaison est perdue (Lightroom fermé, plugin rechargé), l'application l'indique clairement et désactive « Appliquer ». **[Décidé, 2026-11-15]**
- Choisir un menu alors que l'application est déjà ouverte ramène sa fenêtre au premier plan, sur l'écran demandé. **[Décidé, 2026-11-15]**

## 4. Contrat de compatibilité

**[Décidé]** — un seul utilisateur existant, mais on garde la même nomenclature pour que ses catalogues restent valides sans migration.

À conserver à l'identique :

- Identifiant du plugin : `com.querton.divelens`.
- Les 8 champs Dive Lens existants et leur format :

| Champ | Contenu | Format actuel |
|---|---|---|
| DiveId | numéro de plongée | texte, ex. `700` |
| StartDate | début de plongée | date et heure locales du lieu de plongée |
| EndDate | fin de plongée | idem |
| Duration | durée | `MM:SS` |
| MaxDepth | profondeur max | nombre en mètres, ex. `31.2` — en V1, écrit avec son unité (F6) |
| TempHigh | température max | `28.0°C` |
| TempLow | température min | `27.0°C` |
| Rating | note de la plongée | étoiles |

- Les champs Lightroom standard remplis : **pays** ← pays du site, **ville** ← lieu du site, **lieu** ← nom du site, **GPS** ← coordonnées du site.
- Le groupe de collections **« Dive Lens »** et le nom des collections **« numéro – site »**. « Dive Lens » (avec une espace) reste l'emplacement **par défaut** ; l'utilisateur peut en choisir un autre dans les réglages (F2). **[Décidé, 2026-11-12]**
- Les réglages existants : emplacement de la base MacDive et ordre d'affichage, repris automatiquement (F2). Le niveau de journal de l'ancien plugin n'est pas repris : il était ignoré (journal toujours au maximum). Le réglage « nombre de plongées affichées » est **supprimé** : c'était un contournement de la lenteur de Lua et Lightroom. **[Décidé]**

Les nouveaux champs (section 7) **s'ajoutent** sans rien renommer.

**Libellés affichés dans Lightroom [Décidé, 2026-11-13]** : seuls les libellés changent, jamais les identifiants. StartDate « Start Dive » (FR « Début de plongée »), EndDate « End Dive » (« Fin de plongée »), PhotoDepth « Depth » (« Profondeur »), PhotoTemp « Temperature » (« Température »). Les libellés français sont **[Décidé, 2026-11-15]**.

## 5. Fonctionnalités

### F1 — Ouvrir Dive Lens

L'utilisateur choisit l'un des menus Dive Lens dans Lightroom ; l'application s'ouvre connectée au catalogue actif, sur l'écran correspondant au menu. Ensuite, la barre de navigation de l'application donne accès à tous les écrans.

Critères d'acceptation :
- L'application affiche l'état de la liaison avec Lightroom et le nombre de photos actuellement sélectionnées, mis à jour en direct.
- Sans liaison, aucune action d'écriture n'est possible.

### F2 — Réglages

Tous les réglages sont gérés **dans l'application**. **[Décidé]** Le gestionnaire de modules externes de Lightroom n'affiche que la version et un bouton pour ouvrir Dive Lens.

Au premier lancement, l'application reprend automatiquement les réglages existants du plugin actuel : emplacement de la base et ordre d'affichage. L'utilisateur n'a rien à ressaisir. **[Décidé]** Le nouveau plugin garde le même identifiant, il lit donc directement les réglages de l'ancien et les transmet à l'application à la connexion. La reprise n'a lieu qu'une fois, et ne remplace jamais un réglage déjà choisi dans l'application. **[Décidé, 2026-11-12]**

**Écran Réglages [Décidé, 2026-11-12]** — sections, dans l'ordre : Carnet de plongée, Affichage, Écriture sur les photos, Collections, Sauvegarde des réglages (F11, depuis le 2026-11-13), Mises à jour (F13, depuis le 2026-11-15), Diagnostic, À propos. Chaque modification est enregistrée immédiatement, sans bouton « Enregistrer ».

- **Source de plongée** : MacDive (Mac) ou Subsurface (Mac et Windows). Emplacement pré-rempli avec l'emplacement standard de l'application choisie, modifiable via un sélecteur. Pas de recherche automatique ailleurs sur le disque. **[Décidé]**
  - Sur Windows, seul Subsurface est proposé : MacDive n'apparaît pas dans la liste. **[Décidé, 2026-11-12]**
  - Sous l'emplacement : état de la base (« Base valide · N plongées · N sites », ou le problème rencontré), « Choisir… » et, si l'emplacement n'est pas le standard, « Emplacement standard ». **[Décidé, 2026-11-12]**
  - **Subsurface [Décidé, 2026-11-12]** : un carnet Subsurface est soit un fichier (`.xml` ou `.ssrf`), soit un dossier (stockage git). Deux boutons, « Choisir un fichier… » et « Choisir un dossier… » ; l'état indique le format reconnu (« Fichier XML · N plongées · N sites » ou « Stockage git · N plongées · N sites »). Emplacement pré-rempli : le fichier standard de Subsurface, `<utilisateur>.xml` dans son dossier (`~/Library/Application Support/Subsurface/` sur Mac, `%APPDATA%\Subsurface\` sur Windows). Le « fichier par défaut » réglé dans Subsurface n'est pas lu.
  - **Un emplacement mémorisé par application [Décidé, 2026-11-12]** : passer de MacDive à Subsurface puis revenir retrouve l'emplacement MacDive choisi.
  - **Changer d'application [Décidé, 2026-11-14]** : seulement dans Réglages, jamais depuis Créer les collections ni Tagger la sélection. La ligne « <application> · N plongées » reste affichée dans ces écrans, sans être un lien.
  - **Un seul carnet à la fois [Décidé, 2026-11-14]** : changer d'application ou de carnet remet les parcours à zéro : plongées décochées, Créer les collections revient à l'étape 1, Tagger la sélection repart de zéro. Quel que soit le chemin : liste Application, « Choisir… » (ou « Choisir un fichier… » / « Choisir un dossier… »), « Emplacement standard », import des réglages (F11), reprise des réglages de l'ancien plugin à la connexion, boutons de l'écran d'erreur du carnet (« Localiser la base… », etc.). Seul compte le carnet lu (application en service et emplacement), pas le réglage : choisir « Emplacement standard » pour le fichier qui l'était déjà ne remet rien à zéro, et changer seulement l'emplacement de l'autre application (import, reprise) ne relit même pas le carnet : l'écran en cours reste tel quel.
  - **Pendant une écriture dans Lightroom [Décidé, 2026-11-14]** : le carnet ne peut changer par aucun de ces chemins. C'est l'application elle-même qui refuse, les boutons grisés n'en sont que le reflet. Un import qui changerait le carnet est refusé en entier, rien n'est changé. La reprise des réglages de l'ancien plugin, si elle est refusée ainsi, est retentée à la connexion suivante.
    - Présentation **[Décidé, 2026-11-15]** : pendant l'écriture, la liste Application, « Choisir… » (ou « Choisir un fichier… » / « Choisir un dossier… »), « Emplacement standard », « Importer… » et « Remplacer » sont grisés ; sous l'état du carnet et sous « Exporter… / Importer… », une ligne en petits caractères : « Écriture dans Lightroom en cours : le carnet de plongée pourra être changé une fois celle-ci terminée. » (EN « Writing to Lightroom: the dive log can be changed once the write is done. »). Si un changement est quand même refusé, le même texte s'affiche en erreur en haut de l'écran.
  - **Actions sur le carnet affiché [Décidé, 2026-11-14]** : chaque action désigne le carnet que l'écran affiche (application et emplacement) : lecture d'une courbe, « Appliquer » de Tagger la sélection, recherche des photos et « Appliquer » de Créer les collections. Si ce n'est plus le carnet réglé, l'action est refusée et il faut recommencer. La vérification porte sur l'application et l'emplacement, jamais sur la date de modification du fichier : MacDive enregistre souvent sa base, une vérification plus fine bloquerait à tort.
    - Quand deux lectures du carnet se suivent, seule la dernière compte : une lecture lente de l'ancien carnet ne remplace jamais le nouveau. Les courbes déjà affichées sont oubliées à chaque lecture du carnet.
    - Après un refus, le carnet est relu (ce qui remet les parcours à zéro) et l'écran où arrive le plongeur dit pourquoi, une fois.
    - Présentation et textes du refus **[Décidé, 2026-11-15]** : sous le titre de l'étape 1 de Créer les collections ou de Tagger la sélection, un bandeau orange, comme les autres avertissements, avec un bouton « Fermer ». Il disparaît avec « Fermer », « Suivant », « Appliquer », « Annuler » ou en changeant d'écran. Texte : « Le carnet de plongée a changé. Rien n'a été écrit : recommencez. » (EN « The dive log has changed. Nothing was written: start again. ») quand c'était un « Appliquer » ; sinon (recherche des photos, courbe, qui n'écrivent rien) « Le carnet de plongée a changé : recommencez. » (EN « The dive log has changed: start again. »).
- **Ordre d'affichage des plongées** : pas de ligne dans Réglages. La liste mémorise le dernier ordre choisi (récentes, anciennes, profondeur) et le retrouve au lancement suivant ; il vaut pour Créer les collections et Tagger la sélection, qui partagent la même liste. L'ancien réglage (croissant / décroissant) sert de valeur de départ. **[Décidé, 2026-11-12]**
- **Niveau de journal** : « Normal » ou « Détaillé », et bouton « Afficher le journal » (F10). **[Décidé, 2026-11-12]**
- **Profondeur copiée dans l'altitude** (voir F7), désactivée par défaut. **[Décidé]**
- « Inclure les photos prises en surface » (marge) : **pas de ligne dans Réglages**. Elle se règle sur l'écran « Créer les collections » (F5), repart de **0 min** à chaque fois et n'est pas mémorisée : une marge oubliée ferait entrer en silence des photos du bateau dans les collections. **[Décidé, 2026-11-12]**
- **Unités** : profondeur en mètres ou en pieds, température en °C ou en °F. **[Décidé]** Elles s'appliquent à l'affichage dans l'application et aux valeurs écrites dans les champs Dive Lens. L'altitude GPS (F7) reste toujours en mètres, c'est la norme du champ. Les photos déjà taguées gardent l'unité en vigueur au moment de l'application. **[Décidé, 2026-11-15]**
  - **Trois choix par unité [Décidé, 2026-11-12]** : « Comme MacDive » (valeur par défaut, avec l'unité qui en résulte entre parenthèses), ou forcée (mètres / pieds, °C / °F). Deux réglages séparés, car MacDive a un réglage « canadien » (pieds et °C).
  - « Comme l'application de plongée » reprend les **préférences de l'application de plongée** choisie, pas du carnet (qui stocke toujours en métrique). Lecture seule, comme le carnet.
    - MacDive : préférence globale « unités » de MacDive : 0 = m et °C, 1 = ft et °F, 2 (canadien) = ft et °C ; toute autre valeur = m et °C.
    - Subsurface : ses réglages séparés de profondeur et de température (préférences sur Mac, registre sur Windows) ; absents = valeurs par défaut de Subsurface (métrique).
    - Préférences introuvables (application absente, carnet copié d'un autre ordinateur) : m et °C.
- **Langue** de l'interface : anglais et français en V1, ouverte à d'autres langues. Par défaut la langue du système, modifiable dans les réglages. **[Décidé : multilingue, EN + FR, langue du système (2026-11-15)]** Le changement est immédiat, sans redémarrage. Il ne concerne que l'application : les menus Lightroom et les noms des champs Dive Lens suivent la langue de Lightroom. **[Décidé, 2026-11-12]**
- **Emplacement des collections [Décidé, 2026-11-12]** : où « Créer les collections » range les collections, en deux réglages.
  - « Placer dans » : la racine du catalogue, ou un ensemble de collections existant, choisi dans l'arborescence du catalogue (il faut que Lightroom soit connecté pour la parcourir).
  - « Regrouper dans l'ensemble » : un nom, **« Dive Lens »** par défaut (section 4). L'ensemble est créé dans l'emplacement choisi s'il n'existe pas. Vide : les collections vont directement dans l'emplacement choisi.
  - Un aperçu montre le chemin obtenu, par exemple « Voyages › Dive Lens › 700 - Banana Reef ».
  - Changer ce réglage ne déplace pas les collections déjà créées ; une relance crée la collection au nouvel endroit.
  - Si l'ensemble choisi n'existe plus (supprimé ou renommé dans Lightroom, autre catalogue), « Créer les collections » le signale et « Appliquer » reste désactivé jusqu'à un nouveau choix. Aucun repli silencieux vers la racine.

Critères d'acceptation :
- Une base invalide ou introuvable est signalée par un message clair, sans blocage de l'application. L'écran Réglages reste accessible pour la corriger.
- La base est toujours ouverte en lecture seule ; MacDive peut rester ouvert pendant ce temps.

### F3 — Gratuit, open source, dons

**[Décidé]** Dive Lens est **gratuit et open source**. Plus de licence, de clé, de mode d'essai ni d'appel au service de licence : toutes les plongées et tous les sites sont accessibles.

- Code source public ; lien « Code source » dans Réglages › À propos. **[Décidé]**
- Lien **« Soutenir Dive Lens »** (don) dans Réglages › À propos et sur le site de documentation. **[Décidé]**
- Licence open source du code : **GPL v3**. **[Décidé]** libdivecomputer (LGPL 2.1) est compilée dans l'application, ce que sa licence permet pour un programme GPL dont le code est publié.
- **Dépôt [Décidé]** : la V2 démarre dans un dépôt entièrement nouveau, sans l'historique de la version 0.x. Les données de test sont fictives ou anonymisées (pas de nom de plongeur ni de numéro de série réels).
- Plateforme de dons : **Ko-fi**, mise en place dans un second temps. **[Décidé]** Le bouton « Soutenir Dive Lens » est masqué tant que Ko-fi n'existe pas. **[Décidé, 2026-11-12]**
- Réglages › À propos affiche aussi la version installée et la licence (GPL v3). **[Décidé, 2026-11-12]**

### F4 — Liste des plongées

- **Affichage et filtres : redéfinis pendant la phase de design des écrans [Décidé]**, pour tirer parti d'une interface plus riche que les listes Lightroom.
- **Filtres V1 [Décidé, évolutif]** : recherche (n°, site, lieu, pays), période, pays, profondeur, note, « avec courbe » ; plongées regroupées par mois ; panneau de détail de la plongée (courbe, durée, profondeur max, températures, note, GPS, carte, notes).
- Indique si la plongée a une courbe (donc des données par photo possibles) ou non, comme pour une plongée saisie à la main. **[Décidé]**
- Recherche sur le numéro, le site, le lieu et le pays, comme aujourd'hui. **[Décidé : conservé]**
- **Toutes** les plongées sont chargées, sans limite de nombre. **[Décidé]**
- Une plongée marquée « invalide » dans Subsurface reste dans la liste, avec une pastille « Invalide » sur sa ligne et dans sa fiche, pour faire le lien avec Subsurface qui l'affiche barrée. **[Décidé, 2026-11-12]**
- **Fiche de la plongée [Décidé, 2026-11-13]** :
  - Températures : la min en bleu, la max en orange, chacune avec son unité et, au-dessus à gauche, « min. » ou « max. » en petits caractères ; sans flèche entre les deux. Quand les deux sont égales, une seule valeur, sans couleur. **[Décidé, 2026-11-15]** pour ce dernier cas.
  - **Coordonnées GPS** comme dans Lightroom : degrés, minutes, secondes au dixième (environ 3 m) et hémisphère, ex. `20°15'11.2" S 57°30'45.0" E`. Même présentation dans Tagger la sélection. La saisie d'un autre site (F8) reste en degrés décimaux. **[Décidé, 2026-11-13]**
  - Une **carte du site** sous les coordonnées GPS, quand la plongée en a. C'est une carte en ligne, avec les données OpenStreetMap servies gratuitement par OpenFreeMap, sans compte ni clé. Sans Internet, le message « Carte indisponible sans Internet » la remplace. L'application n'envoie à l'hébergeur que la zone de carte affichée, aucune donnée de plongée. Les crédits de la carte sont écrits en petit sur une ligne sous la carte, toujours visibles : les règles d'OpenStreetMap interdisent de les masquer dès l'ouverture. Ils disparaissent avec la carte quand Internet manque. **[Décidé, 2026-11-13]**
  - Détails de la carte **[Décidé, 2026-11-15]** : style sombre bleuté (« Fiord ») ; environ 20 km de large à l'ouverture ; repère cyan ; on la déplace à la souris et on zoome avec les boutons + / −, car la molette fait défiler la fiche ; pas de rotation.
  - Les **notes** saisies par le plongeur dans son carnet (MacDive et Subsurface), sous la carte, retours à la ligne conservés. Elles ne sont pas écrites dans Lightroom.

Critères d'acceptation :
- La liste complète de la base de test (707 plongées) s'affiche et se filtre sans attente perceptible pendant la frappe.

### F5 — Créer les collections (traitement groupé)

C'est l'usage principal, issu de « Create Collection » actuel.

Parcours :
1. L'utilisateur sélectionne une ou plusieurs plongées à la main, comme aujourd'hui. **Pas de notion de voyage.** **[Décidé]**
   - **Parcours en trois étapes, rappelées en haut de chaque écran [Décidé, 2026-11-13]** : choisir les plongées › ajuster et vérifier › récapitulatif. L'étape 1 est la liste des plongées (F4) en sélection multiple, « Suivant » mène à l'étape 2. Le menu Lightroom « Créer les collections… » ouvre toujours l'étape 1, avec les plongées déjà cochées. Plus d'écran « Plongées » séparé : la barre latérale ne contient que Créer les collections, Tagger la sélection et Réglages.
   - **Navigation [Décidé, 2026-11-13]** : chaque étape a ses boutons « ← Précédent » et « Suivant → » (pas de Précédent à l'étape 1). L'indicateur d'étapes, sous le titre, est une ligne qui relie les trois étapes, comme un tunnel d'achat, sans allure de bouton, dans un cadre sur toute la largeur ; cliquer sur une étape déjà faite y revient, comme « Précédent », sauf depuis le récapitulatif, d'où le bandeau ne ramène plus en arrière. Pas de bouton « Modifier les plongées ». Dans la barre du bas, le texte est à gauche et tous les boutons à droite, « Précédent » à côté du bouton d'avancée : le texte ne se décale jamais d'une étape à l'autre. Le récapitulatif prend toute la largeur ; « Retour aux plongées », à droite et sans flèche, ramène à l'étape 1 sans aucune plongée cochée.
   - **Détails [Décidé, 2026-11-15]** : libellés « Choisir les plongées » / « Ajuster et vérifier » / « Récapitulatif » (EN « Select Dives » / « Adjust & Review » / « Summary ») ; à l'étape 2, le bouton d'avancée reste « Appliquer à N photos → », car il écrit dans le catalogue ; ordre « ← Précédent » puis le bouton d'avancée ; « Retour aux plongées » en bouton principal, le dernier à droite ; quitter le récapitulatif par un menu ou la barre latérale décoche aussi les plongées ; « Suivant » grisé, avec une invite, tant qu'aucune plongée n'est cochée ; l'application s'ouvre sur l'étape 1, y compris depuis le bouton « Ouvrir Dive Lens » du gestionnaire de modules externes ; pendant l'écriture, aucun menu ne quitte l'écran, pour que le récapitulatif s'affiche.
2. L'application repère les appareils photo présents parmi les photos de la période couverte. Les photos sont cherchées **dans tout le catalogue**, sur la période des plongées choisies, élargie de la marge et des décalages. **[Décidé, 2026-11-12]** Un appareil est identifié par son modèle **et son numéro de série** quand la photo le contient, sinon par le modèle seul. Deux boîtiers du même modèle ont donc chacun leur décalage. **[Décidé, 2026-11-12]**
3. **Pour chaque appareil**, l'utilisateur règle **un seul décalage d'horloge**, qui remplace les deux curseurs début et fin actuels. **[Décidé]**
   - Plage : ±24 h, au pas de la minute (comme aujourd'hui).
   - **Décalage mémorisé [Décidé, 2026-11-12]** : l'application retient le dernier décalage appliqué à chaque appareil et le pré-remplit la fois suivante. Il sert aussi au repère « Couvre la sélection » de F8. **Partagé avec « Tagger la sélection » [Décidé, 2026-11-14]** : les deux écrans pré-remplissent et mémorisent les mêmes décalages (F8, « Données par photo »). Les décalages sont mémorisés **séparément pour chaque application de plongée** : un même carnet importé de l'une dans l'autre peut avoir des heures de début différentes (constaté : jusqu'à plusieurs heures entières entre MacDive et Subsurface). Après un changement d'application, aucun décalage n'est pré-rempli ; « Suggérer » le retrouve. **[Décidé, 2026-11-12]**
   - Bouton **« Suggérer »** : propose le décalage qui fait tomber le plus de photos dans les plongées choisies. Il teste les écarts au pas de 15 minutes entre −14 h et +14 h, ce qui couvre tous les fuseaux horaires. L'utilisateur accepte ou non. **[Décidé, 2026-11-12]** Le fuseau horaire enregistré dans les photos n'est **pas utilisé en V1** : il ne donne le décalage que si l'on connaît aussi le fuseau de la plongée, que Subsurface n'enregistre pas et que MacDive garde de façon peu fiable (souvent le fuseau du Mac ou « GMT »). **[Décidé, 2026-11-12]**
   - **Affichage des appareils [Décidé, 2026-11-15]** : une carte compacte par appareil, sur deux colonnes, **classées par nom d'appareil** [Décidé] ; les appareils à vérifier (0 photo dans les plongées) sont signalés en orange. Chaque carte : modèle et numéro de série complet **[Décidé, 2026-11-12]**, nombre de photos, décalage réglé par un curseur de −14 h à +14 h au quart d'heure et un champ heures:minutes, à la minute jusqu'à ±24 h (flèches haut/bas : ±1 min) **[Décidé, 2026-11-14]**, « Suggérer », jauge des photos dans les plongées.
   - **Ignorer un appareil [Décidé]** : case par appareil (ex. téléphone resté sur le bateau) ; ses photos ne sont ni comptées ni taguées.
   - **« Suggérer pour tous » [Décidé]** : calcule le décalage de chaque appareil d'un coup, sans toucher aux appareils ignorés ni aux décalages réglés à la main.
4. **« Inclure les photos prises en surface »** (la marge) : « jusqu'à N min avant et après la plongée », optionnel, **par défaut 0**. Elle sert seulement à remplir les collections. **[Décidé]**
5. **Retour en direct** pendant le réglage : nombre de photos trouvées **pour chaque plongée** et au total. Une plongée à 0 photo est mise en évidence, car c'est souvent le signe d'un changement d'horloge en cours de route. **[Décidé]**
6. **Appliquer** : en une seule étape d'annulation (le GPS et l'altitude dans une seconde écriture, juste après la première, section 3),
   - chaque plongée qui a des photos obtient sa collection « numéro – site », à l'emplacement choisi dans les réglages, « Dive Lens » par défaut (F2), créée si besoin. L'écran rappelle cet emplacement avant « Appliquer » ;
   - les photos reçoivent les infos de plongée et le lieu (section 4) ;
   - les photos reçoivent les données par photo (F6) et le décalage appliqué.
7. **Écran de résultat [Décidé, 2026-11-12]** : collections créées, photos écrites, plongées sans photo.

Règles :
- L'heure de référence d'une photo est son **heure de prise de vue d'origine**, ce qui gère les photos revenues de Photoshop en TIFF, comme aujourd'hui.
- Une photo dans la plongée (début ≤ heure corrigée ≤ fin) reçoit tout.
- Une photo seulement dans la marge reçoit les infos de plongée et le lieu, mais **pas** les données par photo. **[Décidé]**
- Une photo qui tomberait dans deux plongées (marges qui se chevauchent) va à la plongée dont elle est le plus proche. **[Décidé, 2026-11-12]**
- Une plongée sans photo ne crée pas de collection : elle est seulement signalée. **[Décidé]**
- **Plongées lues une seule fois [Décidé, 2026-11-14]** : les plongées choisies, leurs courbes et l'application dont elles viennent sont lues au passage de l'étape 1 à l'étape 2, avec la recherche des photos. « Appliquer » écrit celles-là, sans relire le carnet : jamais de mélange entre deux carnets, et le champ Source (« MacDive » / « Subsurface », section 7) est toujours celui du carnet lu. Si le carnet réglé a changé entre-temps, « Appliquer » est refusé et il faut recommencer (F2). Revenir à l'étape 1 puis repasser à l'étape 2 lance une nouvelle recherche ; seule la dernière lancée compte : une recherche plus lente lancée avant (plus de plongées) ne la remplace jamais, ni pour les compteurs, ni pour « Suggérer », ni pour « Appliquer ».
- **Relancer** un groupe ne fait qu'ajouter : les photos mal classées lors d'un passage précédent restent où elles sont, et l'utilisateur les corrige lui-même. Les photos reprises voient leurs valeurs réécrites. **[Décidé]** L'altitude que Dive Lens y avait écrite suit la règle du repère d'altitude (F7) : effacée si la photo n'a plus de profondeur (passée dans la marge, plongée sans courbe) **[Décidé, 2026-11-15]**, et aussi si elle passe dans une autre plongée sans nouvelle altitude (option altitude désactivée), comme dans « Tagger la sélection » **[Proposé, 2026-11-15]**.
- Si l'utilisateur change l'horloge de l'appareil ou de l'ordinateur de plongée en cours de voyage, il traite deux groupes séparés. **[Décidé]**

Critères d'acceptation :
- Le compteur se met à jour pendant que le décalage ou la marge change, sans clic supplémentaire.
- Un « Annuler » dans Lightroom défait l'intégralité d'une application, GPS et altitude compris, comme pour F8 (section 3). Sur Windows, à vérifier par les testeurs.
- Avec des photos prises à l'heure de France et des plongées aux Maldives (+4 h), « Suggérer » propose +4 h.

### F6 — Données par photo

**[Décidé]** Uniquement la **profondeur**, la **température** et le **temps écoulé depuis le début de la plongée**. Pas de pression de bouteille ni de PPO2.

Écrites par « Créer les collections » (F5) et, depuis le 2026-11-14, par « Tagger la sélection » quand la plongée choisie a une courbe (F8, « Données par photo »), avec le même calcul et les mêmes formats. **[Décidé, 2026-11-14]**

- Calcul à partir de la courbe de plongée : heure de la photo + décalage de l'appareil, puis valeur estimée entre les deux points de mesure qui l'entourent. MacDive enregistre un point toutes les 10 secondes. **[Décidé, 2026-11-12 : interpolation linéaire]**
- Plongée sans courbe : les trois champs restent **vides**. **[Décidé]**
- Photo dans la marge, hors de la plongée : champs vides. **[Décidé]**
- Formats :
  - profondeur **avec son unité**, selon le réglage : `18.4 m` ou `60 ft`. **[Décidé]** La profondeur max est écrite de la même façon pour rester cohérente. **[Décidé, 2026-11-12]** Les photos déjà taguées gardent leur ancienne valeur sans unité jusqu'à une nouvelle application ;
  - température comme les températures max et min ;
  - temps écoulé en `MM:SS`, comme la durée.

### F7 — Profondeur dans l'altitude

**[Décidé]**

- La profondeur est **toujours** écrite dans le champ Dive Lens dédié.
- Une option permet de la **copier aussi dans le champ altitude GPS**, en valeur négative, en mètres. Désactivée par défaut. **[Décidé]**
- Comportements **[Décidé, 2026-11-12]** :
  - « Créer les collections » et « Tagger la sélection » écrivent l'altitude, sur les photos qui reçoivent une profondeur : « Tagger la sélection » copie la profondeur des photos prises pendant la plongée, comme « Créer les collections » (F8, « Données par photo »), toujours dans la seconde écriture (section 3) **[Décidé, 2026-11-14]**. Avant le 2026-11-14, seul « Créer les collections » l'écrivait ;
  - « Tagger la sélection » **efface** aussi l'altitude tirée de la profondeur d'une autre plongée, quand il rattache la photo à une nouvelle plongée sans lui écrire de nouvelle altitude (F8) **[Décidé, 2026-11-14]**. « Créer les collections » fait de même quand il range la photo dans une autre plongée sans lui écrire de nouvelle altitude (option désactivée) **[Proposé, 2026-11-15]** ;
  - une photo sans profondeur (prise dans la marge, plongée sans courbe) garde l'altitude qu'elle a déjà, sauf celle que Dive Lens y avait écrite : « Créer les collections » l'efface avec la profondeur dont elle venait, même pour la même plongée (photo passée dans la marge au nouveau passage) **[Décidé, 2026-11-15]** ;
  - désactiver l'option plus tard n'efface pas les altitudes déjà copiées ;
  - « Annuler » dans Lightroom **défait l'altitude** avec le reste, GPS compris : elle est écrite dans une seconde écriture, juste après le reste (section 3) **[Décidé, 2026-11-14]**. Le rappel après écriture le dit : « Pour tout défaire : Édition › Annuler dans Lightroom, en une seule fois, GPS et altitude compris. » (EN « To undo: Edit › Undo in Lightroom, in one step, GPS and altitude included. ») **[Décidé, 2026-11-15]**
- **Repère d'altitude [Décidé, 2026-11-15]** : chaque fois que Dive Lens écrit une altitude sur une photo, il note sa valeur exacte, en mètres, dans un champ Dive Lens caché, le repère d'altitude (`WrittenAltitude`, section 7). Il sait ainsi si l'altitude d'une photo est la sienne, à remplacer ou effacer, ou celle de l'appareil photo, de l'utilisateur ou d'un autre logiciel, à laisser. Avant le 2026-11-15, Dive Lens le devinait d'après la profondeur écrite sur la photo, ce qui manquait les altitudes écrites en pieds.
  - Quand Dive Lens se demande s'il efface une altitude (photo rattachée à une autre plongée, ou dont la profondeur est effacée) :
    - repère présent et altitude égale au repère, au centimètre près : c'est la sienne, effacée avec le repère ;
    - repère présent mais altitude différente (changée depuis à la main ou par un autre logiciel, ou seconde écriture qui n'a pas eu lieu, section 3) : l'altitude reste ; le repère, devenu faux, est effacé ;
    - pas de repère (photos écrites avant le repère, ou jamais par Dive Lens) : règle d'avant, l'altitude n'est effacée que si elle vaut celle que Dive Lens tire de la profondeur écrite sur la photo, au décimètre près (F8).
  - Une altitude écrite remplace celle de la photo, quelle qu'elle soit, comme avant ; le repère prend sa valeur. Une altitude effacée par Dive Lens emporte son repère.
  - Le repère va dans la première écriture, avec les autres champs Dive Lens, l'altitude dans la seconde (section 3). Si la seconde n'a pas lieu, repère et altitude diffèrent et l'altitude est laissée : c'est le côté sûr. Un « Annuler » défait les deux.
  - Si Lightroom refuse d'effacer une altitude, elle reste, sans repère : la fois suivante, la règle d'avant s'applique.
  - Rien ne change à l'écran : le repère n'est visible nulle part dans Lightroom.
- L'option est accompagnée d'une explication, qui dit trois choses :
  - le champ Dive Lens n'existe que dans Lightroom ;
  - l'altitude suit la photo exportée et se voit dans les autres logiciels ;
  - la copie remplace une altitude GPS déjà présente ; « Annuler » dans Lightroom la remet : « La copie remplace une altitude GPS déjà présente sur la photo ; Édition › Annuler dans Lightroom la remet. Les photos sans profondeur gardent la leur. » (EN « The copy replaces a GPS altitude already on the photo; Edit › Undo in Lightroom brings it back. Photos without a depth keep theirs. ») **[Proposé, 2026-11-14]**

### F8 — Tagger la sélection

**[Décidé]** Fusion des actuels « Set Dive Metadata » et « Set Location ». L'utilisateur tague **les photos sélectionnées dans Lightroom**, quelle que soit leur heure, avec :

- les **données de plongée** d'une plongée choisie (case à cocher) ;
- le **lieu** (case à cocher) : soit le site de la plongée choisie, soit un autre site choisi par pays › lieu › site parmi les sites du carnet de plongée, avec ses coordonnées GPS modifiables comme aujourd'hui.

Décocher « Données de plongée » permet de ne poser qu'un lieu, par exemple sur des photos prises hors plongée (bateau, plage, snorkeling). Au moins une des deux cases doit être cochée.

Usages : photos sans heure fiable, rattachement forcé, ou lieu seul.

Sélection **[Décidé]** :
- **La sélection se fait dans Lightroom**, comme aujourd'hui. L'application la suit en direct : nombre de photos et vignettes.
- « Appliquer » écrit sur les photos sélectionnées dans Lightroom **au moment du clic**. Le bouton affiche toujours le nombre réel, par exemple « Appliquer à 24 photos ».
- Sans sélection, le bouton est désactivé avec le message « Sélectionnez des photos dans Lightroom ».

Ce qui est écrit **[Décidé : comportement actuel en V1]** :
- Données de plongée : n° de plongée, début, fin, durée, profondeur max, températures, note, et l'application source (section 7). Pas de collection. Celles d'une autre plongée déjà sur la photo sont effacées (voir « Photo déjà rattachée à une autre plongée » plus bas).
- Données par photo : quand la plongée choisie a une courbe, sur les photos prises pendant la plongée (voir « Données par photo » plus bas) **[Décidé, 2026-11-14]**. Avant le 2026-11-14 : pas de données par photo.
- Lieu : pays, ville, lieu, GPS (section 4).
- Annulation : un « Annuler » dans Lightroom défait toute l'écriture, **GPS et altitude compris** : ils sont écrits dans une seconde écriture, juste après le reste (section 3). **[Décidé, 2026-11-14]** (Avant : « sauf le GPS », constaté le 2026-11-12 quand tout était écrit en une seule écriture, voir section 9.)

Comportements **[Décidé, 2026-11-12]** :
- Une valeur absente de la plongée choisie (température, note, profondeur) **efface** l'ancienne valeur Dive Lens de la photo : les champs décrivent une seule plongée.
- Plongée sans numéro : le champ numéro reste vide (l'ancien plugin écrivait « 0 »).
- Site sans pays, ville ou GPS : les valeurs déjà présentes sur la photo ne sont pas touchées (protège un GPS venu d'un téléphone). Les valeurs « <No Country> » de l'ancien plugin ne sont plus écrites.
- Tant qu'aucune plongée n'est choisie, le lieu part sur « Autre site » ; il passe au site de la plongée dès qu'une plongée est choisie.
- Liste : un clic sur une ligne choisit la plongée ; les flèches ne font que parcourir (la fiche suit la ligne parcourue).
- « Annuler » remet l'écran à zéro, sans rien garder (il n'y a plus d'écran Plongées où revenir). **[Décidé, 2026-11-15]**
- « Appliquer » écrit les données du carnet affiché ; si le carnet réglé a changé entre-temps, rien n'est écrit et il faut recommencer. Changer de carnet dans Réglages remet l'écran à zéro (F2). **[Décidé, 2026-11-14]**
- Après l'écriture : bandeau avec le nombre de photos taguées et le rappel qu'« Annuler » dans Lightroom défait tout en une fois : « Pour annuler : Édition › Annuler dans Lightroom, en une seule fois, GPS et altitude compris. » (EN « To undo: Edit › Undo in Lightroom, in one step, GPS and altitude included. ») **[Décidé, 2026-11-15]**

Données par photo **[Décidé, 2026-11-14]** (remplace l'« évolution envisagée » : calculer les données par photo ; alimenter la collection de la plongée n'en fait pas partie, toujours pas de collection) :
- Quand « Données de plongée » est cochée et que la plongée choisie a une courbe, « Tagger la sélection » écrit aussi, sur les photos sélectionnées dont l'heure corrigée tombe dans la plongée (début ≤ heure corrigée ≤ fin), la profondeur, la température et le temps écoulé, calculés exactement comme dans « Créer les collections » (F6 : interpolation linéaire, mêmes formats et unités).
- **Décalage d'horloge par appareil**, comme à l'étape 2 de « Créer les collections » : sous « Données de plongée », un bloc avec une carte compacte par appareil présent dans la sélection, classées par nom d'appareil (appareil = modèle et numéro de série quand la photo le contient, F5). Chaque carte : nombre de photos, décalage (même curseur et même champ heures:minutes qu'à l'étape 2 de « Créer les collections »), « Suggérer » et une jauge « N dans la plongée ». Au-dessus des cartes, une ligne en direct : « Données par photo : N photos sur M dans la plongée ». Le bloc est **toujours visible** (pas repliable) quand « Données de plongée » est cochée et que la plongée a une courbe. Plongée sans courbe : une seule ligne à la place, « Pas de courbe : pas de données par photo ». Les cartes sont celles de « Créer les collections », sans copie à part.
- « Suggérer » propose le décalage qui fait tomber le plus de photos sélectionnées de l'appareil dans la plongée choisie, avec la même recherche que F5 (pas de 15 minutes entre −14 h et +14 h). Avec des photos prises à l'heure de France et une plongée aux Maldives (+4 h), « Suggérer » propose +4:00.
- **Décalage pré-rempli** avec le dernier décalage appliqué à l'appareil, mémorisé séparément pour chaque application de plongée, au même endroit que pour « Créer les collections » (mêmes fichiers de décalages). Aucun décalage mémorisé : 0:00, et « Suggérer » est proposé. **Après « Appliquer », les décalages utilisés sont mémorisés**, partagés avec « Créer les collections » (F5).
- Les photos hors de la plongée (après décalage) reçoivent les données de plongée sans données par photo, comme les photos de la marge dans F5. **Pas de marge** dans « Tagger la sélection ».
- Le **décalage d'horloge** (ClockOffset, section 7) n'est écrit que sur les photos qui reçoivent des données par photo.
- **Altitude** : quand l'option « Copier la profondeur dans l'altitude GPS » est activée, la profondeur est copiée dans l'altitude, comme dans « Créer les collections » (F7), toujours dans la seconde écriture (section 3).
- Le repère « Couvre la sélection » suit les décalages affichés à l'écran, pas seulement les décalages mémorisés (voir « Suggestion » plus bas).
- Tout le reste de « Tagger la sélection » est inchangé : lieu, GPS, Source, pas de collection. Le lieu seul (« Données de plongée » décochée) est inchangé.
- Photo déjà rattachée (voir plus bas) : **autre plongée** : les photos dans la plongée reçoivent les nouvelles valeurs ; les photos hors de la plongée voient les valeurs par photo de l'ancienne plongée effacées, comme avant ; l'altitude, comme dans « Photo déjà rattachée à une autre plongée » plus bas. **Même plongée** : les photos dans la plongée voient leurs valeurs par photo réécrites avec le décalage affiché ; les photos hors de la plongée gardent leurs valeurs par photo **[Décidé, 2026-11-15]**.
- Détails **[Décidé, 2026-11-15]** :
  - Le bloc s'intitule « Horloge des appareils », avec l'explication de « Créer les collections » : « Écart entre l'heure de chaque appareil et celle du carnet de plongée. Appareils classés par nom ; ceux à vérifier sont en orange. » Il se place sous la liste des plongées et sa fiche, au-dessus de la ligne « Écrit : … ». Sans plongée choisie, ni bloc ni ligne.
  - Les cartes sont exactement celles de « Créer les collections », étiquette « décalage mémorisé » comprise ; carte en orange quand aucune photo de l'appareil n'est dans la plongée ; « Suggérer » affiché sur chaque carte, sans mise en avant particulière quand aucun décalage n'est mémorisé. Ni « Ignorer » ni « Suggérer pour tous ».
  - **Réglage du décalage [Décidé, 2026-11-14]** : pas de boutons −1 h / −1 min / +1 min / +1 h ; la carte garde le curseur et le champ heures:minutes de « Créer les collections », pour que les deux écrans restent identiques.
  - Textes anglais : « Per-photo data: N photos of M in the dive », « No profile: no per-photo data », jauge « N in the dive ». M compte toutes les photos sélectionnées, y compris celles sans heure de prise de vue, qui ne sont jamais dans la plongée.
  - « Suggérer » sans résultat : « Aucun décalage ne place de photo de cet appareil dans la plongée. » (EN « No offset puts a photo of this camera in the dive. »)
  - Ligne sous la liste quand la plongée a une courbe : « Écrit : n° de plongée, début, fin, durée, profondeur max, températures, note ; sur les photos prises pendant la plongée, aussi profondeur · température · temps et le décalage d'horloge. » (EN « Written: dive number, start, end, duration, max depth, temperatures, rating; on the photos taken during the dive, also depth · temperature · time and the clock offset. ») Sinon, la ligne actuelle.
  - **Courbe illisible** (la plongée a une courbe, mais elle ne se décode pas) : une seule ligne à la place du bloc, « Courbe illisible : pas de données par photo » (EN « Profile could not be read: no per-photo data »), et « Appliquer » écrit les données de plongée sans données par photo, comme avant les données par photo : une courbe illisible ne bloque jamais le tag. « Créer les collections » n'est pas changé. Pendant la lecture de la courbe : « Chargement de la courbe… » (texte de la fiche).
  - L'application ne lit la sélection photo par photo (heure, appareil) qu'une fois une plongée avec courbe choisie, puis à chaque changement de sélection. Lightroom n'est interrogé qu'une fois par sélection : la même lecture sert à la sélection en direct. Pendant la lecture, dans le bloc et dans la barre du bas, « Appliquer » grisé : « Lecture de la sélection dans Lightroom… » (EN « Reading the selection in Lightroom… »). En cas d'échec : « La sélection n'a pas pu être lue dans Lightroom : {erreur} » (EN « The selection could not be read in Lightroom: {error} ») et « Réessayer ».
  - Changement de sélection : un appareil encore présent garde le décalage affiché ; un nouvel appareil part du décalage mémorisé. Les cartes et les compteurs précédents restent affichés jusqu'aux nouveaux. Le repère « Couvre la sélection » prend les décalages des cartes dès qu'elles sont affichées ; avant (aucune plongée avec courbe choisie), les décalages mémorisés, comme avant les données par photo.
  - Une fois une plongée choisie, le repère « Couvre la sélection » peut se déplacer pendant le réglage des décalages, mais la fiche affichée ne saute pas sur la plongée qu'il désigne : elle ne le suit que tant qu'aucune plongée n'est choisie.
  - Le message « Aucun décalage ne place… » d'une carte disparaît quand une autre plongée est choisie.
  - « Appliquer » écrit les données par photo calculées pour la sélection affichée. Si, au moment du clic, la sélection dans Lightroom n'est plus exactement celle-là, rien n'est écrit : « La sélection a changé dans Lightroom. Rien n'a été écrit : vérifiez les données par photo, puis appliquez à nouveau. » (EN « The selection changed in Lightroom. Nothing was written: check the per-photo data, then apply again. »), et la sélection est relue. Sans données par photo (plongée sans courbe, lieu seul), « Appliquer » écrit comme avant sur la sélection du moment.
  - Décalages mémorisés après « Appliquer » : ceux des appareils dont au moins une photo est dans la plongée (le décalage leur a été appliqué), comme dans « Créer les collections » ; aussi quand seuls le GPS et l'altitude n'ont pas pu être écrits.
  - Bandeau après écriture inchangé (nombre de photos taguées, sans le nombre de photos avec profondeur).
  - Réglages › Écriture sur les photos : la précision sous « Copier la profondeur dans l'altitude GPS » devient « En valeur négative, en mètres, en plus du champ Dive Lens. Pour « Créer les collections » et « Tagger la sélection ». » (EN « As a negative value, in metres, in addition to the Dive Lens field. Create Collections and Tag Selection. »), au lieu de « … Pour « Créer les collections » seulement. ».

Photo déjà rattachée à une autre plongée **[Décidé, 2026-11-14]** :
- Quand « Données de plongée » est cochée et qu'une photo porte déjà une **autre** plongée, ses valeurs par photo sont effacées : profondeur, température, temps écoulé et décalage d'horloge (F6, section 7). Elles décrivaient l'ancienne plongée.
- L'altitude GPS n'est effacée que si c'est celle que Dive Lens a écrite, d'après le repère d'altitude (F7) **[Décidé, 2026-11-15]**. Sans repère (photo taguée avant le repère), seulement si elle vaut **exactement** celle que Dive Lens a tirée de l'ancienne profondeur, au décimètre près. Sinon elle reste : ce champ Lightroom est partagé avec l'appareil photo et l'utilisateur. Dans le doute, on la garde.
  - Sans repère, en pieds, la profondeur écrite est arrondie au pied : l'altitude tirée de la profondeur exacte ne se retrouve souvent pas, et elle est alors gardée. Avec le repère, elle est reconnue et effacée.
  - Sans repère, profondeur absente ou illisible : l'altitude est gardée.
- Quand la plongée a une courbe, les photos prises pendant la plongée reçoivent leurs nouvelles valeurs par photo au lieu de les voir effacées ; l'altitude que Dive Lens avait écrite est alors remplacée par la nouvelle si l'option altitude est activée, et effacée sinon (voir « Données par photo » plus haut) **[Décidé, 2026-11-14]**.
- **Même plongée** : application source, début et n° de plongée identiques tous les trois. Un n° vide des deux côtés compte comme identique. Rien d'autre n'est alors touché, sauf les valeurs par photo des photos prises pendant la plongée, réécrites avec le décalage affiché (« Données par photo » plus haut).
  - Le début est enregistré sur la photo en texte, au format de date du système. Si ce format change entre deux écritures (changement de région, catalogue ouvert sur Mac puis sur Windows), la même plongée passe pour une autre : ses valeurs par photo sont effacées, ainsi que l'altitude GPS écrite par Dive Lens, dans les conditions ci-dessus (« Annuler » dans Lightroom les remet). Accepté pour la V1. **[Décidé, 2026-11-14]**
- Une photo sans données de plongée n'a rien à effacer. Le lieu seul (« Données de plongée » décochée) ne touche à rien d'autre.
- La vérification se fait au moment de l'écriture, d'après la plongée que porte la photo, dans la première écriture ; l'altitude est effacée dans la seconde, avec le GPS, et son repère dans la première (section 3). Le tout reste une seule étape d'annulation.
- « Annuler » dans Lightroom remet les valeurs par photo effacées et l'altitude, repère compris. Le bandeau après écriture a un seul rappel, qu'une altitude ait été effacée ou non (voir plus haut) ; le journal du plugin note sur combien de photos les valeurs d'une autre plongée et l'altitude ont été effacées. **[Décidé, 2026-11-14]**
- Si Lightroom refuse d'effacer une altitude, elle reste et le reste de l'écriture se fait.
- La photo reste dans la collection de l'ancienne plongée : c'est à l'utilisateur de la déplacer. Pas de changement de collection automatique.
- Détails **[Décidé, 2026-11-15]** : « sans données de plongée » = ni source, ni début, ni n° ; un champ vidé à la main dans Lightroom compte comme vide ; une profondeur sans unité (format d'avant les unités) se lit en mètres.

Liste des plongées **[Décidé]** : **la même que dans F4** (mêmes colonnes, regroupement par mois, recherche et filtres), en choix unique.

Suggestion **[Décidé]** : la ligne de la plongée dont les heures couvrent la sélection porte un repère « Couvre la sélection ». Le calcul applique à chaque boîtier le décalage affiché à l'écran (« Données par photo » plus haut), pré-rempli avec le dernier décalage appliqué au boîtier, dans « Créer les collections » ou « Tagger la sélection », quand il existe ; tant qu'aucun décalage n'est affiché, ce dernier décalage appliqué. **[Décidé, 2026-11-14]** (Avant : seulement le dernier décalage appliqué dans « Créer les collections », 2026-11-12.) Ce n'est qu'une aide : l'utilisateur choisit.


### F9 — (fusionné dans F8)

### F10 — Journal

Un fichier journal est conservé pour le diagnostic, consultable depuis les réglages. **[Décidé, 2026-11-15 : conservé]**

**Emplacement [Décidé, 2026-11-12]** : l'application écrit son journal (`DiveLens App.log`) dans le dossier des journaux de Lightroom, à côté de celui du plugin (`DiveLens.log`) : `~/Library/Logs/Adobe/Lightroom/LrClassicLogs/` sur Mac, `%LOCALAPPDATA%\Adobe\Lightroom\Logs\LrClassicLogs\` sur Windows. « Afficher le journal » ouvre ce dossier, avec les deux fichiers côte à côte. Le niveau choisi (Normal / Détaillé) s'applique aux deux. Le journal de l'application est limité en taille : au-delà, l'ancien est gardé une fois puis remplacé.

**Signaler un problème [Décidé, 2026-11-14]** :
- Les journaux ne sont jamais joints à un ticket GitHub : le dépôt est public, et les journaux contiennent les sites de plongée et des chemins de fichiers. Le problème se décrit dans un ticket ; si les journaux sont nécessaires, ils sont demandés, puis envoyés par e-mail à divelens@querton.com seulement, avec le numéro du ticket. Le guide (F12), le formulaire de signalement de bug, `CONTRIBUTING.md` et `README.md` le disent.
- Prise en charge d'une nouvelle application de plongée : les carnets réels anonymisés envoyés par le public sont acceptés, en plus des carnets fictifs, en privé à la même adresse, jamais joints publiquement. Ils servent aux essais. Le dépôt, lui, ne contient que des données de test fictives ou correctement anonymisées, jamais un carnet réel tel quel.
- Le signalement privé des vulnérabilités de GitHub est activé. Une faille ou un secret exposé se signale par là ou à la même adresse, jamais dans un ticket ou une pull request publics (`SECURITY.md`, en anglais). Seule la dernière version publiée est prise en charge.

### F11 — Exporter et importer les réglages

**[Décidé]** L'utilisateur peut exporter ses réglages dans un fichier pour les reprendre sur un autre ordinateur, ou après une réinstallation.

Chantier à part, fait après Subsurface (2026-11-13). Les réglages tiennent déjà dans un seul fichier lisible. **[Décidé, 2026-11-12]**

- Contenu : tout ce que contient le fichier de réglages de l'application : application de plongée et emplacement du carnet de chacune, unités, langue, option altitude, emplacement des collections, niveau de journal, ordre des plongées, et les réglages ajoutés plus tard. Pas la marge (elle n'est plus mémorisée, F2), ni les décalages d'horloge mémorisés, ni le fait que les réglages de l'ancien plugin ont déjà été repris (propre à cet ordinateur). **[Décidé, 2026-11-15]**
- Format : le fichier de réglages lui-même, en JSON lisible, avec son numéro de version. **[Décidé, 2026-11-15]**
- Import : les réglages du fichier remplacent les réglages en cours, après confirmation. Si la base indiquée est introuvable sur cet ordinateur, l'application le signale et demande de la localiser. **[Décidé, 2026-11-15]**
  - Un fichier qui n'est pas un fichier de réglages Dive Lens (pas de numéro de version) est refusé avec un message ; rien n'est changé.
  - Carnet introuvable : l'emplacement du fichier est **gardé** (jamais de bascule silencieuse vers un autre carnet) ; la confirmation le dit déjà, puis la section affiche le problème avec « Choisir… » (ou « Choisir un fichier… » / « Choisir un dossier… » pour Subsurface).
  - Ensemble de collections absent de ce catalogue : même état que dans F2 (signalé, « Appliquer » désactivé jusqu'à un nouveau choix).
  - Après un import, les réglages de l'ancien plugin ne sont plus jamais repris : le fichier importé est un choix de l'utilisateur. **[Décidé, 2026-11-13]**
- **Écran [Décidé, 2026-11-15]** : section « Sauvegarde des réglages » (EN « Settings backup ») entre Collections et Diagnostic, comme dans la maquette. Une ligne « Exporter ou importer » (« Pour retrouver vos réglages sur un autre ordinateur, ou après une réinstallation. ») avec « Exporter… » et « Importer… ».
  - Export : fenêtre d'enregistrement du système, nom proposé « Réglages Dive Lens.json » (EN « Dive Lens settings.json »), puis « Réglages exportés dans « … » ».
  - Import : fenêtre d'ouverture du système (fichiers `.json`), puis une confirmation dans la section : « Remplacer les réglages actuels par ceux de « … » ? — Tous les réglages de cet écran sont remplacés. », boutons « Remplacer » et « Annuler ».

Critères d'acceptation :
- Un fichier exporté par une version antérieure de Dive Lens reste importable ; les réglages inconnus sont ignorés.

### F12 — Documentation utilisateur (site web)

**[Décidé]** Une documentation utilisateur en ligne accompagne la V1, dans les mêmes langues que l'application (anglais et français).

Contenu **[Décidé, 2026-11-15]** :
- installation et premier lancement (base MacDive, reprise des réglages) ;
- créer les collections : choix des plongées, horloge des appareils, « Suggérer », photos prises en surface ;
- tagger la sélection : données de plongée, lieu, autre site ;
- réglages : unités, langue, profondeur dans l'altitude, export des réglages ;
- dépannage : aucune photo trouvée, base introuvable, Lightroom déconnecté, horloge changée en cours de voyage ;
- confidentialité : la base de plongée est lue sans jamais être modifiée.

Liens depuis l'application vers la page d'aide concernée (un « ? » par écran) **[Décidé, 2026-11-15]**.

Diffusion **[Décidé]** : un seul texte source, publié à deux endroits :
- **embarqué dans l'application**, consultable hors connexion, toujours à jour avec la version installée ;
- **publié sur le site du plugin** (querton.com), pour les futurs utilisateurs et les moteurs de recherche.

**Texte source [2026-11-13]** : premier jet dans `docs/user/en.md` et `docs/user/fr.md`, un fichier par langue, à relire par le propriétaire. Il reprend les libellés exacts de l'application et de Lightroom, et ajoute au contenu ci-dessus le premier lancement d'une version non signée (Mac : « Ouvrir quand même » ; Windows : « Exécuter quand même », WebView2) et les champs Dive Lens dans Lightroom. L'export des réglages n'y figure pas tant que F11 n'est pas livré. Restent à faire : l'embarquer dans l'application, le publier sur querton.com et les liens « ? ».

**Sauvegarde des réglages [2026-11-14]** : F11 étant livré, le guide la décrit dans la section Réglages (contenu, export, import, carnet introuvable, fin de la reprise de l'ancien module après un import).

**Guide des contributeurs [2026-11-14]** : `CONTRIBUTING.md` (en anglais) a une section « Adding a dive application » : ticket « Support a new dive application » d'abord, puis la fiche de l'application, le lecteur copié du lecteur d'exemple, le carnet de référence, les tests communs, la vérification locale `verify_log` (sa sortie est une donnée personnelle, jamais dans le dépôt, un ticket ou une pull request) et la liste de vérification de la pull request, qui demande aussi de citer la nouvelle application dans le README et le guide **[Décidé, 2026-11-15]** ; les étapes restent dans `docs/contributing/reference-log.md`. `CLAUDE.md` importe `CONTRIBUTING.md` au lieu d'en recopier les règles. **[Décidé, 2026-11-14]**

### F13 — Mises à jour automatiques

**[Décidé : oui]** L'application vérifie s'il existe une nouvelle version de Dive Lens et propose de l'installer.

- **Tout le paquet** est remplacé : l'application et la partie Lightroom. **[Décidé, 2026-11-15]**
- **Vérification** au lancement de l'application, au plus une fois par jour, avec un réglage pour la désactiver (activé par défaut) et un bouton « Vérifier maintenant ». Une vérification automatique qui échoue (hors ligne) n'affiche rien ; « Vérifier maintenant » dit pourquoi. **[Décidé, 2026-11-15]**
- **Versions stables seulement** : une pré-version n'est jamais proposée, et il n'y a pas de bêta. **[Décidé, 2026-11-15]**
- **Signalement [Décidé, 2026-11-15]** : un bandeau au-dessus de l'écran en cours, « Dive Lens 2.1.0 est disponible. [Voir] [×] ». [Voir] ouvre Réglages › Mises à jour ; [×] ferme le bandeau jusqu'à la version suivante **[Proposé]**. Le bandeau ne s'affiche pas sur l'écran Réglages, qui le dit dans sa section **[Proposé]**.
- **Réglages › Mises à jour [Décidé, 2026-11-15]**, entre « Sauvegarde des réglages » et « Diagnostic », comme dans la maquette : « Vérifier automatiquement » (« Au lancement, au plus une fois par jour. Rien ne s'installe sans votre accord. ») et « Dive Lens <version> » avec la dernière vérification et « Vérifier maintenant ». Quand une version est trouvée : « Version 2.1.0 disponible », [Installer], un lien « Nouveautés » vers la page de la version sur GitHub, et « Après l'installation, redémarrez Lightroom pour terminer la mise à jour. » Pendant l'installation : une barre de progression.
- **Pas de notes dans l'application**, seulement le lien vers la page de la version : l'application sera traduite dans d'autres langues, aucun texte ne vient du serveur. **[Décidé, 2026-11-15]**
- **Rien ne s'installe sans l'accord de l'utilisateur** (bouton [Installer]), ni pendant une écriture dans Lightroom ; aucune écriture ne commence pendant le remplacement. **[Décidé, 2026-11-15]**
- **Installation [Décidé, 2026-11-15]** : l'application télécharge le fichier de son système, vérifie sa signature, remplace le dossier du plugin là où il est, puis redémarre sur Réglages.
  - Mac : le fichier est le zip du dossier du plugin ; il est décompressé à côté du dossier, vérifié (signature Apple comprise), puis échangé avec l'ancien. L'ancienne version reste à côté, cachée, jusqu'au lancement suivant ; si l'échange échoue, elle est remise en place.
  - Windows : le fichier est l'installateur, lancé sans fenêtre sur le dossier en place (on ne peut pas renommer un dossier dont le programme tourne). La version affichée dans « Applications installées » reste juste. Une copie installée avec le zip ailleurs y apparaît ensuite, à son emplacement.
  - Rien ne reste à côté du dossier sous un nom en `.lrplugin` : Lightroom le chargerait comme un second Dive Lens.
- **Copie qui ne peut pas se mettre à jour** (dossier non modifiable) : « Télécharger depuis GitHub » à la place de [Installer]. Une version de développement (hors d'un dossier de plugin) ne vérifie rien d'elle-même et ne s'installe jamais. **[Décidé, 2026-11-15]**
- **Ensuite** : tant que Lightroom tourne avec l'ancienne partie Dive Lens (autre version, ou autre protocole de liaison), un bandeau dit « Redémarrez Lightroom pour terminer la mise à jour de Dive Lens. », et la barre latérale « Redémarrez Lightroom » au lieu de « Lightroom non connecté ». Lightroom ne recharge sa partie Dive Lens qu'au redémarrage. **[Décidé, 2026-11-15]**
- **Signature** : chaque fichier de mise à jour est signé avec la clé de mise à jour Dive Lens (section 9), et la signature porte le numéro de version ; l'application refuse un fichier dont la signature ne correspond pas à sa clé publique ou nomme une autre version. Signature Apple et Windows : section 9. **[Décidé]**
- **Hébergement** sur **GitHub Releases** : chaque version publie `latest.json`, qui nomme le fichier de chaque système et sa signature ; l'application le lit à l'adresse de la dernière version publiée. Un brouillon n'est jamais proposé : c'est la publication qui propose la version à tous. querton.com garde la présentation, la documentation et le lien de don. **[Décidé]**
- **Numérotation [Décidé]** : la première version publique est la **2.0.0**, pour marquer la rupture avec le plugin 0.x. Versions de développement en 0.x d'ici là.
- **Première version** : une copie installée avant la première version qui contient F13 ne se met jamais à jour seule ; F13 doit donc être dans la 2.0.0. **[Décidé, 2026-11-15]**
- Mises à jour ouvertes à tous (plugin gratuit). **[Décidé]**

## 6. Inventaire des écrans (entrée des maquettes)

Description fonctionnelle seulement. La mise en page est une décision du propriétaire, à faire dans les maquettes.

**Cohérence [Décidé]** : une donnée affichée à plusieurs endroits (ex. la liste des plongées) a partout la même présentation.

**Principe [Décidé]** : les écrans sont repensés **de A à Z**, sans reprendre la structure des fenêtres Lua actuelles. On peut découper ou regrouper les écrans librement. Le seul critère est la facilité d'usage. La liste ci-dessous décrit des **besoins**, pas un découpage imposé.

| Écran | Ce qu'il montre | Actions |
|---|---|---|
| **Accueil / état** | Liaison Lightroom, nombre de photos sélectionnées, source de plongée et nombre de plongées | Aller vers chaque usage, ouvrir les réglages |
| **Liste des plongées** | Plongées (F4), indicateur de courbe ; étape 1 de « Créer les collections », pas d'écran à part | Rechercher, sélectionner une ou plusieurs plongées |
| **Créer les collections** (F5) | Plongées choisies avec compteur de photos chacune, appareils détectés, décalage et marge, total | Régler le décalage par appareil, « Suggérer », régler la marge, Appliquer |
| **Tagger la sélection** (F8) | Sélection Lightroom en direct (nombre, vignettes), plongée choisie, appareils de la sélection avec leur décalage et le nombre de photos dans la plongée quand elle a une courbe, lieu (site de la plongée ou autre site : pays › lieu › site, GPS) | Cocher données de plongée et/ou lieu, choisir, régler le décalage par appareil, « Suggérer », Appliquer |
| **Réglages** (F2, F3, F7, F10, F13) | Source de plongée et état de la base, langue, unités, option altitude et son explication, emplacement des collections, mises à jour, journal, à propos | Modifier, afficher le journal, exporter et importer les réglages (F11), vérifier et installer une mise à jour (F13) |
| **Résultat d'application** | Collections créées, photos écrites, plongées sans photo | Fermer |

États à couvrir dans les maquettes : liaison perdue, base introuvable, plongée sans courbe, aucune photo trouvée, écriture en cours avec progression **[Décidé]**.

**Listes déroulantes [Décidé, 2026-11-13]** : fermées, elles ont l'aspect des autres champs (fond sombre, bord fin, flèche cyan), identique sur Mac et Windows ; ouvertes, c'est le menu du système.

## 7. Champs ajoutés

**[Décidé, 2026-11-12]** — noms définitifs, jamais renommés (section 4).

**Panneau Dive Lens de Lightroom [Décidé, 2026-11-13]** : quatre sections titrées, la photo d'abord. **Photo** (profondeur, température, temps écoulé) ; **Plongée** (n°, début, fin, durée, profondeur max, temp. min, temp. max, note) ; **Site** (lieu, ville, pays, GPS, champs standard de Lightroom) ; **Technique** (décalage d'horloge, source). Titres français « Photo », « Plongée », « Site », « Technique » **[Décidé, 2026-11-15]**. La vue « Toutes les métadonnées de plug-in » de Lightroom garde l'ordre de définition des champs, sans titres.

| Champ | Contenu | Visible dans Lightroom |
|---|---|---|
| PhotoDepth | profondeur au moment de la photo | oui |
| PhotoTemp | température au moment de la photo | oui |
| PhotoElapsed | temps écoulé depuis le début de la plongée | oui |
| ClockOffset | décalage d'horloge appliqué à la photo, `±HH:MM` | oui **[Décidé]** |
| Source | application dont viennent les données de plongée : `MacDive` ou `Subsurface` | oui **[Décidé, 2026-11-12]** |
| WrittenAltitude | repère d'altitude : l'altitude GPS que Dive Lens a écrite sur la photo, en mètres, en texte (F7) | non : champ caché, absent de tout panneau, ni cherché ni filtrable **[Décidé, 2026-11-15]** |

PhotoDepth, PhotoTemp, PhotoElapsed et ClockOffset sont écrits par « Créer les collections » (ClockOffset sur toutes les photos rangées, marge comprise) et par « Tagger la sélection » seulement sur les photos prises pendant la plongée, quand elle a une courbe (F8). **[Décidé, 2026-11-14]**

Source est écrit chaque fois que des données de plongée le sont : par « Créer les collections » sur toutes les photos rangées, et par « Tagger la sélection » quand « Données de plongée » est cochée. Un lieu posé seul ne le touche pas. Les photos déjà taguées le gardent vide jusqu'à une nouvelle application. **[Décidé, 2026-11-12]**

WrittenAltitude est écrit avec chaque altitude que Dive Lens écrit (F7), par « Créer les collections » et « Tagger la sélection » ; il est effacé avec cette altitude, ou quand elle a changé depuis. Champ sans titre pour Lightroom, donc invisible ; ajouté le 2026-11-15, la version du schéma des champs passe de 3 à 4, sans migration (section 9). **[Décidé, 2026-11-15]**

Les champs Dive Lens restent dans le catalogue Lightroom et ne sont pas écrits dans le fichier photo. D'où l'option altitude (F7).

## 8. Sources de données

### Format commun **[Décidé]**

Chaque source passe par un **lecteur** qui traduit ses données dans un format commun. L'interface et le plugin ne connaissent que ce format. Chaque lecteur déclare ce qu'il sait fournir, et l'interface masque ce qui manque (température, note, GPS…).

Contenu du format commun pour une plongée :
- un identifiant stable dans la source ;
- numéro, début, fin, durée, profondeur max, températures max et min, note ;
- site, lieu, pays, latitude et longitude ;
- la courbe de plongée : temps, profondeur, température, si disponible.

**Carnet de référence [Décidé, 2026-11-15]** : chaque lecteur lit le même carnet fictif de dix plongées et passe les mêmes tests communs. Lire un carnet ne modifie jamais la base ni son journal (`-wal`) et ne crée aucun fichier à côté ; seule écriture restante : quand MacDive est ouvert, SQLite met à jour son index partagé (`-shm`), comme pour tout lecteur.

### MacDive (V1)

- Base SQLite lue **en lecture seule**, à l'emplacement donné par l'utilisateur. La base peut se trouver dans iCloud.
- La courbe vient du **téléchargement brut de l'ordinateur** que MacDive conserve, décodé par libdivecomputer d'après le nom de l'ordinateur noté par MacDive (ex. « Shearwater Perdix 2 »). Les données Shearwater sont d'abord décompressées. La copie chiffrée propre à MacDive n'est pas utilisée (format non documenté).
- Une plongée **sans téléchargement brut** n'a pas de courbe : plongée saisie à la main, ou ordinateur ancien importé avant que MacDive ne conserve ce téléchargement (constaté sur d'autres bases : jusqu'à 424 plongées sur 530).
- Vérifié sur la base du propriétaire : 649 plongées sur 756 ont une courbe, toutes décodées (Shearwater Perdix 2, Mares Genius) ; les 107 autres sont saisies à la main. Les courbes sont identiques à celles que produit MacDive, à un détail près : pour Shearwater, libdivecomputer place le premier point à 10 s au lieu de 0 s. Dive Lens suit libdivecomputer, donc le même calage que Subsurface.

### Subsurface (V1) **[Décidé]**

- Mac et Windows. Lecture seule, à l'emplacement donné par l'utilisateur.
- Deux formats de stockage à gérer :
  - **fichier XML** ;
  - **stockage versionné** (dossier de type git), format par défaut avec le stockage en ligne de Subsurface, qui en garde une copie locale.
- La courbe est en clair dans les deux formats.
- Pays et ville d'un site sont facultatifs dans Subsurface : l'interface gère leur absence.
- Données de test : carnets **fictifs** dans les deux formats, avec les mêmes plongées, pour vérifier que les deux lectures donnent le même résultat. Le carnet du propriétaire sert seulement à une vérification locale, jamais versée au dépôt.

**Règles de lecture [Décidé, 2026-11-12]** — principe général : **chaque valeur est reprise telle que l'application de plongée l'affiche**, pour éviter toute incompréhension.

- **Plusieurs ordinateurs sur une plongée** : le premier ordinateur, celui que Subsurface affiche, fournit la courbe, la profondeur max, les températures et les données par photo.
- **Plongée sans ordinateur** : « No Computer » (plongée importée de MacDive) ou « manually added dive » (saisie à la main dans Subsurface), avec ses quelques points saisis à la main : traitée comme une plongée **sans courbe** (données par photo vides, pas d'indicateur de courbe).
- **Nom de site « Pays / Lieu / Site »** : découpé en pays, lieu et site seulement si le pays et la ville du site sont vides dans Subsurface **et** que le nom a exactement trois parties séparées par « / » ; sinon le nom entier reste le site.
- **Durée et fin** : la durée affichée par Subsurface. Elle s'arrête à la sortie de l'eau, alors que MacDive compte aussi les minutes enregistrées en surface : constaté sur le carnet du propriétaire, 74 plongées plus courtes de 2 min 20 en médiane, jusqu'à 7 min 40.
- **Températures** : Subsurface n'affiche qu'une température de l'eau par plongée. Min = température de l'eau, à défaut le point le plus froid de la courbe ; max = point le plus chaud de la courbe, à défaut la température de l'eau (seule exception au principe, sinon le champ resterait toujours vide). Jamais la température de l'air. Constaté : mêmes valeurs que MacDive sur 620 des 648 plongées du propriétaire qui ont une température.
- **Note** : 1 à 5 étoiles, reprise telle quelle ; 0 ou absente = sans note.
- **Plongée sans numéro** : champ numéro vide, collection nommée par la date (règle de F8).
- **Plongée « invalide »** dans Subsurface : lue comme les autres, signalée par une pastille (F4).
- **Voyages** Subsurface : les plongées rangées dans un voyage sont lues comme les autres (pas de notion de voyage en V1).
- **Identifiants** : une plongée est identifiée par son heure de début (Subsurface n'a pas d'identifiant de plongée), avec un suffixe si deux plongées commencent à la même heure ; un site par l'identifiant que Subsurface lui donne, le même dans les deux formats. Aucun de ces identifiants n'est conservé d'une session à l'autre ni écrit sur les photos.
- **Formats** : seul le format actuel de Subsurface est lu ; un fichier très ancien affiche « ouvrez-le et réenregistrez-le dans Subsurface ». `.ssrf` est accepté comme `.xml`.
- **Stockage git** : Dive Lens lit, comme Subsurface, le dernier enregistrement du carnet (une branche), pas les fichiers du dossier. Il prend le carnet actif du dossier, ou le seul qu'il contient ; s'il y en a plusieurs sans carnet actif, l'état affiche « Plusieurs carnets dans ce dossier ». Pas de choix du carnet en V1.

**Passer de MacDive à Subsurface [Décidé, 2026-11-12]** — aucune migration :
- les décalages mémorisés sont propres à chaque application (F5) ;
- une relance retrouve la collection existante tant que le nom « numéro - site » est identique (742 sur 748 plongées chez le propriétaire) ; sinon une nouvelle collection est créée à côté ; rien n'est déplacé ni supprimé ;
- les photos déjà taguées gardent leurs valeurs jusqu'à une nouvelle application, qui écrit alors les heures et la durée de Subsurface ;
- la documentation (F12) le dit en une phrase.

### Sources suivantes **[Décidé]**

BlueDive et Submersion, après examen de leur stockage.

## 9. Points techniques à vérifier avant développement

- **Ordinateurs anciens sans téléchargement brut** dans MacDive : message à afficher (« pas de courbe disponible pour cet ordinateur »), à distinguer d'une plongée saisie à la main.
- **Ajout de champs** — vérifié dans le kit Lightroom 16 (2026-11-12) : ajouter des champs ne demande aucune migration ; les valeurs existantes sont conservées tant que la définition des 8 champs existants ne change pas. Le numéro de version par champ ne sert qu'à un changement incompatible. La version globale du schéma passe de 1 à 2, comme le kit le recommande à chaque évolution, sans routine de mise à jour (puis à 3 avec Source, à 4 avec le repère d'altitude, section 7). Un champ sans titre n'apparaît dans aucun panneau et ne peut être ni cherché ni filtré (kit Lightroom 16) : à vérifier dans Lightroom pour le repère d'altitude. Preuve finale : test dans Lightroom (anciennes valeurs toujours visibles après rechargement).
- **Écriture de l'altitude** — vérifié (2026-11-12) : un plugin peut écrire l'altitude GPS (nombre, en mètres). « Annuler » la remet, avec le GPS, depuis qu'ils sont écrits à part (voir ci-dessous). Sans effet tant que l'option est désactivée.
- **Annulation du GPS et de l'altitude** — **[Constaté, 2026-11-14]**, dans Lightroom Classic sur Mac, avec un plugin de test minimal :
  - le GPS et l'altitude écrits par un plugin **dans la même écriture qu'un champ de lieu texte** (pays, ville, lieu), dans n'importe quel ordre, ne sont **pas** défaits par « Annuler » ; les champs texte et les champs du plugin, eux, le sont ;
  - écrits seuls (le GPS, l'altitude, ou les deux sans champ texte), ils sont défaits ;
  - écrits dans une **seconde écriture**, juste après celle des champs texte, sans action de l'utilisateur entre les deux, ils sont défaits : sur Mac, les deux écritures forment **une seule étape d'annulation**, au nom de la dernière (le kit Lightroom documente ce regroupement pour macOS) ;
  - un GPS saisi à la main dans Lightroom est défait normalement.
  - Le constat du 2026-11-12 (« Annuler ne remet pas le GPS ni l'altitude ») venait de là : Dive Lens écrivait tout en une seule écriture. D'où le GPS et l'altitude dans une seconde écriture (section 3, **[Décidé, 2026-11-14]**).
  - **Windows** : le regroupement des deux écritures en une seule étape n'est pas documenté. À vérifier par les testeurs Windows : un seul « Annuler » doit tout défaire, GPS et altitude compris.
- **Recherche des photos par date** — constaté (2026-11-12) : la recherche du catalogue par date de prise de vue ne fonctionne qu'au jour près, et sur l'heure de prise de vue modifiée dans Lightroom, pas l'heure d'origine. Le plugin cherche donc par jours élargis, puis l'application filtre sur l'heure d'origine.
- **Filtrer par profondeur** : les champs de plugin sont du texte. Un filtre « photos sous 30 m » n'est probablement pas possible dans Lightroom ; seuls la recherche et la navigation par valeur le sont. À vérifier avant de promettre quoi que ce soit.
- **Températures nulles ou négatives** : le plugin actuel ne les écrit pas (plongée sous glace). **[Décidé, 2026-11-15 : corrigé en V1]**
- **Signature [Décidé, 2026-11-13]** : V1 développée et testée **sans signature** (sur Mac, signature « ad hoc » seulement : macOS signale un développeur non identifié au lieu d'une application endommagée).
  - **Apple** : signée (Developer ID) et notarisée **avant la première version publique**. Compte Apple Developer Program au nom du propriétaire (99 USD/an). Certificat et mots de passe dans les secrets GitHub, jamais dans le dépôt.
    - **En place [2026-11-15]** : deux certificats, « Developer ID Application » (l'application) et « Developer ID Installer » (l'installateur `.pkg`), plus une clé d'API App Store Connect pour la notarisation ; les sept valeurs sont dans les secrets GitHub. Chaque version signe et fait notariser l'application, puis l'installateur. Les certificats expirent en décembre 2027 : les renouveler avant ; les versions déjà publiées restent valables après.
  - **Windows** : première version publique **non signée** (alerte SmartScreen, « Exécuter quand même »), puis demande à **SignPath Foundation** : gratuit pour l'open source, mais exige une version déjà publiée et documentée ; le certificat est au nom de SignPath Foundation. Microsoft Artifact Signing n'est pas ouvert aux particuliers en Suisse.
- **Clé de signature des mises à jour** : créée dès le début, conservée hors du dépôt, jamais changée.
- **Windows** : la partie Lightroom du plugin bloque Windows aujourd'hui ; chemins, lancement de l'application et installation à adapter. Tests manuels Windows **[Décidé]** : machine virtuelle Windows 11 sur le Mac (Apple Silicon, donc Windows ARM) + testeurs sur de vrais PC (Intel/AMD). Vérifier que Lightroom Classic et Subsurface tournent dans la machine virtuelle.
- **Réglage « Application »** (F2) : MacDive ou Subsurface, avec l'emplacement du fichier ou du dossier correspondant ; sur Windows, seul Subsurface est proposé.
- **Fichier journal** — tranché (2026-11-12) : le kit Lightroom documente l'emplacement fixe du journal du plugin ; l'application écrit le sien à côté (F10).
- **Réglages de l'ancien plugin** — vérifié dans le kit Lightroom 16 (2026-11-12) : les préférences d'un plugin sont rangées sous son identifiant ; le nouveau plugin, de même identifiant, les lit donc directement.

## 10. À trancher

Aucune question ouverte à ce stade.

### Tranché

- Repère d'altitude (2026-11-15) : un champ Dive Lens caché note l'altitude que Dive Lens écrit ; il décide si l'altitude d'une photo est la sienne, en pieds comme en mètres ; « Créer les collections » efface celle qu'il avait écrite quand la photo perd sa profondeur (marge, plongée sans courbe) ; photos sans repère : règle d'avant (F5, F7, F8, section 7). Reste proposé : l'effacer aussi quand la photo passe dans une autre plongée, option altitude désactivée (F5, F7).
- Tagger la sélection, données par photo (2026-11-14) : profondeur, température et temps écoulé sur les photos prises pendant la plongée, avec un décalage par appareil comme dans « Créer les collections », partagé avec lui ; altitude avec l'option ; repère « Couvre la sélection » d'après les décalages affichés ; toujours pas de collection (F5, F6, F7, F8).
- Annulation du GPS et de l'altitude (2026-11-14) : écrits dans une seconde écriture, juste après le reste, pour qu'un seul « Annuler » défasse tout (section 3, section 9).
- Coordonnées GPS (2026-11-13) : affichées comme dans Lightroom (degrés, minutes, secondes), saisie toujours en décimal (F4).
- Carte du site (2026-11-13) : carte en ligne sous le GPS de la fiche, plutôt qu'une carte hors ligne grossière (F4).
- Nom (2026-11-13) : « Dive Lens » en deux mots partout où il s'affiche (application, textes dans Lightroom, nom de l'application installée, documentation). Les identifiants techniques et les noms des fichiers journaux restent « DiveLens ».
- Parcours « Créer les collections » (2026-11-13) : trois étapes visibles, la liste des plongées en est la première ; plus d'écran « Plongées » séparé ; le menu Lightroom ouvre l'étape 1 (F5).
- Subsurface (2026-11-12) : premier ordinateur, plongées sans ordinateur sans courbe, découpage « Pays / Lieu / Site », valeurs telles que Subsurface les affiche, choix d'un fichier ou d'un dossier, un emplacement et des décalages mémorisés par application, pastille « Invalide », pas de migration au changement d'application (F2, F4, F5, section 8).
- Écran Réglages (2026-11-12) : unités « comme l'application de plongée » ou forcées ; Subsurface grisé « bientôt » ; journal de l'application à côté de celui du plugin ; ordre des plongées mémorisé sur la liste ; marge non mémorisée ; emplacement des collections au choix, « Dive Lens » par défaut ; export / import et mises à jour reportés (F2, F10, F11, F13).
- Fuseau horaire des photos : non utilisé par « Suggérer » en V1 (F5).
- Documentation : un seul texte, embarqué dans l'application et publié sur querton.com (F12).

- Unités : réglage métrique / impérial (F2) ; profondeurs écrites avec leur unité (F6).
- Liste des plongées : affichage et filtres redéfinis pendant le design (F4).
- Identification d'un appareil : modèle + numéro de série quand disponible (F5, proposé).
- Tagger la sélection : fusion des deux usages sur la sélection ; comportement actuel en V1, évolution notée (F8) ; données par photo ajoutées le 2026-11-14.
- Noms : « Créer les collections » et « Tagger la sélection » ; deux menus raccourcis dans Lightroom (section 3).
- Langues : multilingue (F2).
- Emplacement de la base : emplacement standard de MacDive pré-rempli (F2).
- Dive Lens devient gratuit et open source (GPL v3), avec un lien de don Ko-fi plus tard ; plus de licence ni de mode d'essai ; mises à jour ouvertes à tous (F3, F13).
- Décalage d'horloge visible dans le panneau Dive Lens de Lightroom (section 7).
