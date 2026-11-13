# DiveLens — Spécification de la refonte, V1

Statut : validé par le propriétaire, base de développement · Date : 2026-11-11 · Rédigée en français (document produit, validé par le propriétaire)

Ce document décrit **quoi** fait la V1 de la refonte. Il ne décrit pas l'implémentation détaillée ni la mise en page des écrans (les maquettes viennent après, à partir de l'inventaire des écrans, section 6).

Chaque élément est marqué :

- **[Décidé]** — validé par le propriétaire.
- **[Proposé]** — choix courant fait par défaut, à contester si besoin.
- **[À trancher]** — décision produit ouverte, listée en section 10.

---

## 1. Objectif

Remplacer les fenêtres Lightroom actuelles du plugin par une application dédiée, moderne et réactive, qui devient l'**unique point d'entrée** de DiveLens, tout en préparant l'ajout progressif d'autres carnets de plongée que MacDive.

La V1 apporte en plus une fonctionnalité nouvelle : la **profondeur, la température et le temps écoulé au moment de chaque photo**.

## 2. Périmètre V1

| Inclus | Exclu (plus tard) |
|---|---|
| Sources : **MacDive et Subsurface** **[Décidé]** | BlueDive, Submersion |
| **macOS (Intel et Apple Silicon) et Windows** **[Décidé]** ; MacDive sur Mac seulement, Subsurface sur les deux | Signature Apple et Windows (avant ou après la première version publique, à décider) |
| Les trois usages actuels du plugin, repensés | Notion de « voyage » |
| Données par photo (profondeur, température, temps écoulé) | Ajustement visuel sur la courbe de plongée |
| Décalage d'horloge par appareil, avec suggestion | Calage automatique par la profondeur enregistrée par l'appareil |
| **Plugin gratuit** : plus de licence ni de mode d'essai | Nettoyage automatique lors d'une relance |
| | Consultation du carnet de plongée avec photos |
| | Effacer les métadonnées DiveLens des photos sélectionnées **[Décidé, 2026-11-12]** |

**libdivecomputer est intégré dès la V1 [Décidé, 2026-11-11]** : une vraie base MacDive ne contient pas de courbe lisible, seulement une copie chiffrée propre à MacDive et le téléchargement brut de l'ordinateur de plongée. libdivecomputer (la bibliothèque de Subsurface) décode ce téléchargement. La courbe lisible observée au départ avait été ajoutée à la base de test par un outil externe (DiverCity).

## 3. Architecture

**[Décidé]**

```
Lightroom Classic
 └─ Plugin DiveLens (Lua, invisible)
     - déclare les champs DiveLens
     - lit le catalogue (photos, sélection courante) et y écrit
     - lance l'application
            ▲
            │ liaison locale : lecture en direct, écriture sur « Appliquer »
            ▼
Application DiveLens (Rust + Tauri)
 - seule interface utilisateur
 - lecteurs de carnets de plongée (MacDive et Subsurface en V1) → format commun
```

- **Ouverture** : deux éléments de menu dans Lightroom (Bibliothèque › Modules externes), qui ne sont que des **raccourcis** : chacun ouvre l'application directement sur l'écran correspondant. Aucune fenêtre Lua. Lightroom doit rester ouvert. **[Décidé]**
  - « Créer les collections… » (EN « Create Collections… ») → F5
  - « Tagger la sélection… » (EN « Tag Selection… ») → F8, actif seulement si des photos sont sélectionnées
- **Livraison** : un seul paquet ; l'application est rangée dans le dossier du plugin. **[Décidé]**
- **Aucune fenêtre Lua.** Le gestionnaire de modules externes de Lightroom affichera au minimum la version et un bouton pour ouvrir DiveLens (Lightroom impose cette entrée). **[Proposé]**
- **Liaison** : l'application suit en direct la sélection de photos dans Lightroom. Rien n'est écrit dans le catalogue avant que l'utilisateur clique sur **Appliquer** ; tout est alors écrit en **une seule fois**, donc annulable en une seule étape dans Lightroom. **[Décidé]**
- Si la liaison est perdue (Lightroom fermé, plugin rechargé), l'application l'indique clairement et désactive « Appliquer ». **[Proposé]**
- Choisir un menu alors que l'application est déjà ouverte ramène sa fenêtre au premier plan, sur l'écran demandé. **[Proposé]**

## 4. Contrat de compatibilité

**[Décidé]** — un seul utilisateur existant, mais on garde la même nomenclature pour que ses catalogues restent valides sans migration.

À conserver à l'identique :

- Identifiant du plugin : `com.querton.divelens`.
- Les 8 champs DiveLens existants et leur format :

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

## 5. Fonctionnalités

### F1 — Ouvrir DiveLens

L'utilisateur choisit l'un des menus DiveLens dans Lightroom ; l'application s'ouvre connectée au catalogue actif, sur l'écran correspondant au menu. Ensuite, la barre de navigation de l'application donne accès à tous les écrans.

Critères d'acceptation :
- L'application affiche l'état de la liaison avec Lightroom et le nombre de photos actuellement sélectionnées, mis à jour en direct.
- Sans liaison, aucune action d'écriture n'est possible.

### F2 — Réglages

Tous les réglages sont gérés **dans l'application**. **[Décidé]** Le gestionnaire de modules externes de Lightroom n'affiche que la version et un bouton pour ouvrir DiveLens.

Au premier lancement, l'application reprend automatiquement les réglages existants du plugin actuel : emplacement de la base et ordre d'affichage. L'utilisateur n'a rien à ressaisir. **[Décidé]** Le nouveau plugin garde le même identifiant, il lit donc directement les réglages de l'ancien et les transmet à l'application à la connexion. La reprise n'a lieu qu'une fois, et ne remplace jamais un réglage déjà choisi dans l'application. **[Décidé, 2026-11-12]**

**Écran Réglages [Décidé, 2026-11-12]** — sections, dans l'ordre : Carnet de plongée, Affichage, Écriture sur les photos, Collections, Diagnostic, À propos. Chaque modification est enregistrée immédiatement, sans bouton « Enregistrer ». Les sections « Sauvegarde des réglages » (F11) et « Mises à jour » (F13) de la maquette sont masquées tant que ces fonctions n'existent pas.

- **Source de plongée** : MacDive (Mac) ou Subsurface (Mac et Windows). Emplacement pré-rempli avec l'emplacement standard de l'application choisie, modifiable via un sélecteur. Pas de recherche automatique ailleurs sur le disque. **[Décidé]**
  - Sur Windows, seul Subsurface est proposé : MacDive n'apparaît pas dans la liste. **[Décidé, 2026-11-12]**
  - Sous l'emplacement : état de la base (« Base valide · N plongées · N sites », ou le problème rencontré), « Choisir… » et, si l'emplacement n'est pas le standard, « Emplacement standard ». **[Décidé, 2026-11-12]**
  - **Subsurface [Décidé, 2026-11-12]** : un carnet Subsurface est soit un fichier (`.xml` ou `.ssrf`), soit un dossier (stockage git). Deux boutons, « Choisir un fichier… » et « Choisir un dossier… » ; l'état indique le format reconnu (« Fichier XML · N plongées · N sites » ou « Stockage git · N plongées · N sites »). Emplacement pré-rempli : le fichier standard de Subsurface, `<utilisateur>.xml` dans son dossier (`~/Library/Application Support/Subsurface/` sur Mac, `%APPDATA%\Subsurface\` sur Windows). Le « fichier par défaut » réglé dans Subsurface n'est pas lu.
  - **Un emplacement mémorisé par application [Décidé, 2026-11-12]** : passer de MacDive à Subsurface puis revenir retrouve l'emplacement MacDive choisi.
- **Ordre d'affichage des plongées** : pas de ligne dans Réglages. La liste mémorise le dernier ordre choisi (récentes, anciennes, profondeur) et le retrouve au lancement suivant ; il vaut pour Plongées et Tagger la sélection, qui partagent la même liste. L'ancien réglage (croissant / décroissant) sert de valeur de départ. **[Décidé, 2026-11-12]**
- **Niveau de journal** : « Normal » ou « Détaillé », et bouton « Afficher le journal » (F10). **[Décidé, 2026-11-12]**
- **Profondeur copiée dans l'altitude** (voir F7), désactivée par défaut. **[Décidé]**
- « Inclure les photos prises en surface » (marge) : **pas de ligne dans Réglages**. Elle se règle sur l'écran « Créer les collections » (F5), repart de **0 min** à chaque fois et n'est pas mémorisée : une marge oubliée ferait entrer en silence des photos du bateau dans les collections. **[Décidé, 2026-11-12]**
- **Unités** : profondeur en mètres ou en pieds, température en °C ou en °F. **[Décidé]** Elles s'appliquent à l'affichage dans l'application et aux valeurs écrites dans les champs DiveLens. L'altitude GPS (F7) reste toujours en mètres, c'est la norme du champ. Les photos déjà taguées gardent l'unité en vigueur au moment de l'application. **[Proposé]**
  - **Trois choix par unité [Décidé, 2026-11-12]** : « Comme MacDive » (valeur par défaut, avec l'unité qui en résulte entre parenthèses), ou forcée (mètres / pieds, °C / °F). Deux réglages séparés, car MacDive a un réglage « canadien » (pieds et °C).
  - « Comme l'application de plongée » reprend les **préférences de l'application de plongée** choisie, pas du carnet (qui stocke toujours en métrique). Lecture seule, comme le carnet.
    - MacDive : préférence globale « unités » de MacDive : 0 = m et °C, 1 = ft et °F, 2 (canadien) = ft et °C ; toute autre valeur = m et °C.
    - Subsurface : ses réglages séparés de profondeur et de température (préférences sur Mac, registre sur Windows) ; absents = valeurs par défaut de Subsurface (métrique).
    - Préférences introuvables (application absente, carnet copié d'un autre ordinateur) : m et °C.
- **Langue** de l'interface : anglais et français en V1, ouverte à d'autres langues. Par défaut la langue du système, modifiable dans les réglages. **[Décidé : multilingue ; Proposé : EN + FR, langue du système]** Le changement est immédiat, sans redémarrage. Il ne concerne que l'application : les menus Lightroom et les noms des champs DiveLens suivent la langue de Lightroom. **[Décidé, 2026-11-12]**
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

**[Décidé]** DiveLens est **gratuit et open source**. Plus de licence, de clé, de mode d'essai ni d'appel au service de licence : toutes les plongées et tous les sites sont accessibles.

- Code source public ; lien « Code source » dans Réglages › À propos. **[Décidé]**
- Lien **« Soutenir DiveLens »** (don) dans Réglages › À propos et sur le site de documentation. **[Décidé]**
- Licence open source du code : **GPL v3**. **[Décidé]** libdivecomputer (LGPL 2.1) est compilée dans l'application, ce que sa licence permet pour un programme GPL dont le code est publié.
- **Dépôt [Décidé]** : la V2 démarre dans un dépôt entièrement nouveau, sans l'historique de la version 0.x. Les données de test sont fictives ou anonymisées (pas de nom de plongeur ni de numéro de série réels).
- Plateforme de dons : **Ko-fi**, mise en place dans un second temps. **[Décidé]** Le bouton « Soutenir DiveLens » est masqué tant que Ko-fi n'existe pas. **[Décidé, 2026-11-12]**
- Réglages › À propos affiche aussi la version installée et la licence (GPL v3). **[Décidé, 2026-11-12]**

### F4 — Liste des plongées

- **Affichage et filtres : redéfinis pendant la phase de design des écrans [Décidé]**, pour tirer parti d'une interface plus riche que les listes Lightroom.
- **Filtres V1 [Décidé, évolutif]** : recherche (n°, site, lieu, pays), période, pays, profondeur, note, « avec courbe » ; plongées regroupées par mois ; panneau de détail de la plongée (courbe, durée, profondeur max, températures, note, GPS).
- Indique si la plongée a une courbe (donc des données par photo possibles) ou non, comme pour une plongée saisie à la main. **[Décidé]**
- Recherche sur le numéro, le site, le lieu et le pays, comme aujourd'hui. **[Décidé : conservé]**
- **Toutes** les plongées sont chargées, sans limite de nombre. **[Décidé]**
- Une plongée marquée « invalide » dans Subsurface reste dans la liste, avec une pastille « Invalide » sur sa ligne et dans sa fiche, pour faire le lien avec Subsurface qui l'affiche barrée. **[Décidé, 2026-11-12]**

Critères d'acceptation :
- La liste complète de la base de test (707 plongées) s'affiche et se filtre sans attente perceptible pendant la frappe.

### F5 — Créer les collections (traitement groupé)

C'est l'usage principal, issu de « Create Collection » actuel.

Parcours :
1. L'utilisateur sélectionne une ou plusieurs plongées à la main, comme aujourd'hui. **Pas de notion de voyage.** **[Décidé]**
   - Les plongées se choisissent sur l'écran Plongées (sélection multiple), puis « Créer les collections ». « Modifier les plongées » y revient en gardant la sélection. Ouvert depuis le menu Lightroom sans plongée choisie, l'écran est vide, avec un bouton « Choisir les plongées » vers l'écran Plongées. **[Décidé, 2026-11-12]**
2. L'application repère les appareils photo présents parmi les photos de la période couverte. Les photos sont cherchées **dans tout le catalogue**, sur la période des plongées choisies, élargie de la marge et des décalages. **[Décidé, 2026-11-12]** Un appareil est identifié par son modèle **et son numéro de série** quand la photo le contient, sinon par le modèle seul. Deux boîtiers du même modèle ont donc chacun leur décalage. **[Décidé, 2026-11-12]**
3. **Pour chaque appareil**, l'utilisateur règle **un seul décalage d'horloge**, qui remplace les deux curseurs début et fin actuels. **[Décidé]**
   - Plage : ±24 h, au pas de la minute (comme aujourd'hui).
   - **Décalage mémorisé [Décidé, 2026-11-12]** : l'application retient le dernier décalage appliqué à chaque appareil et le pré-remplit la fois suivante. Il sert aussi au repère « Couvre la sélection » de F8. Les décalages sont mémorisés **séparément pour chaque application de plongée** : un même carnet importé de l'une dans l'autre peut avoir des heures de début différentes (constaté : jusqu'à plusieurs heures entières entre MacDive et Subsurface). Après un changement d'application, aucun décalage n'est pré-rempli ; « Suggérer » le retrouve. **[Décidé, 2026-11-12]**
   - Bouton **« Suggérer »** : propose le décalage qui fait tomber le plus de photos dans les plongées choisies. Il teste les écarts au pas de 15 minutes entre −14 h et +14 h, ce qui couvre tous les fuseaux horaires. L'utilisateur accepte ou non. **[Décidé, 2026-11-12]** Le fuseau horaire enregistré dans les photos n'est **pas utilisé en V1** : il ne donne le décalage que si l'on connaît aussi le fuseau de la plongée, que Subsurface n'enregistre pas et que MacDive garde de façon peu fiable (souvent le fuseau du Mac ou « GMT »). **[Décidé, 2026-11-12]**
   - **Affichage des appareils [Proposé, maquette]** : une carte compacte par appareil, sur deux colonnes, **classées par nom d'appareil** [Décidé] ; les appareils à vérifier (0 photo dans les plongées) sont signalés en orange. Chaque carte : modèle et numéro de série complet **[Décidé, 2026-11-12]**, nombre de photos, décalage avec boutons −1 h / −1 min / +1 min / +1 h, « Suggérer », jauge des photos dans les plongées.
   - **Ignorer un appareil [Décidé]** : case par appareil (ex. téléphone resté sur le bateau) ; ses photos ne sont ni comptées ni taguées.
   - **« Suggérer pour tous » [Décidé]** : calcule le décalage de chaque appareil d'un coup, sans toucher aux appareils ignorés ni aux décalages réglés à la main.
4. **« Inclure les photos prises en surface »** (la marge) : « jusqu'à N min avant et après la plongée », optionnel, **par défaut 0**. Elle sert seulement à remplir les collections. **[Décidé]**
5. **Retour en direct** pendant le réglage : nombre de photos trouvées **pour chaque plongée** et au total. Une plongée à 0 photo est mise en évidence, car c'est souvent le signe d'un changement d'horloge en cours de route. **[Décidé]**
6. **Appliquer** : en une seule écriture,
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
- **Relancer** un groupe ne fait qu'ajouter : les photos mal classées lors d'un passage précédent restent où elles sont, et l'utilisateur les corrige lui-même. Les photos reprises voient leurs valeurs réécrites. **[Décidé]**
- Si l'utilisateur change l'horloge de l'appareil ou de l'ordinateur de plongée en cours de voyage, il traite deux groupes séparés. **[Décidé]**

Critères d'acceptation :
- Le compteur se met à jour pendant que le décalage ou la marge change, sans clic supplémentaire.
- Un « Annuler » dans Lightroom défait l'intégralité d'une application, sauf le GPS, comme pour F8, et sauf l'altitude quand la profondeur y est copiée (F7) (constaté le 2026-11-12).
- Avec des photos prises à l'heure de France et des plongées aux Maldives (+4 h), « Suggérer » propose +4 h.

### F6 — Données par photo

**[Décidé]** Uniquement la **profondeur**, la **température** et le **temps écoulé depuis le début de la plongée**. Pas de pression de bouteille ni de PPO2.

- Calcul à partir de la courbe de plongée : heure de la photo + décalage de l'appareil, puis valeur estimée entre les deux points de mesure qui l'entourent. MacDive enregistre un point toutes les 10 secondes. **[Décidé, 2026-11-12 : interpolation linéaire]**
- Plongée sans courbe : les trois champs restent **vides**. **[Décidé]**
- Photo dans la marge, hors de la plongée : champs vides. **[Décidé]**
- Formats :
  - profondeur **avec son unité**, selon le réglage : `18.4 m` ou `60 ft`. **[Décidé]** La profondeur max est écrite de la même façon pour rester cohérente. **[Décidé, 2026-11-12]** Les photos déjà taguées gardent leur ancienne valeur sans unité jusqu'à une nouvelle application ;
  - température comme les températures max et min ;
  - temps écoulé en `MM:SS`, comme la durée.

### F7 — Profondeur dans l'altitude

**[Décidé]**

- La profondeur est **toujours** écrite dans le champ DiveLens dédié.
- Une option permet de la **copier aussi dans le champ altitude GPS**, en valeur négative, en mètres. Désactivée par défaut. **[Décidé]**
- Comportements **[Décidé, 2026-11-12]** :
  - seul « Créer les collections » écrit l'altitude : « Tagger la sélection » n'écrit pas de profondeur par photo ;
  - une photo sans profondeur (prise dans la marge, plongée sans courbe) garde l'altitude qu'elle a déjà ;
  - désactiver l'option plus tard n'efface pas les altitudes déjà copiées ;
  - « Annuler » dans Lightroom **ne défait pas l'altitude**, comme pour le GPS (constaté le 2026-11-12). Accepté ; l'explication de l'option et le rappel après écriture le disent.
- L'option est accompagnée d'une explication, qui dit trois choses :
  - le champ DiveLens n'existe que dans Lightroom ;
  - l'altitude suit la photo exportée et se voit dans les autres logiciels ;
  - la copie remplace une altitude GPS déjà présente.

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
- Données de plongée : n° de plongée, début, fin, durée, profondeur max, températures, note, et l'application source (section 7). Pas de données par photo, pas de collection.
- Lieu : pays, ville, lieu, GPS (section 4).
- Évolution envisagée : calculer les données par photo et alimenter la collection de la plongée.
- Annulation : « Annuler » dans Lightroom défait l'écriture, **sauf le GPS**, que Lightroom ne remet pas (constaté le 2026-11-12). Accepté tel quel. **[Décidé]**

Comportements **[Décidé, 2026-11-12]** :
- Une valeur absente de la plongée choisie (température, note, profondeur) **efface** l'ancienne valeur DiveLens de la photo : les champs décrivent une seule plongée.
- Plongée sans numéro : le champ numéro reste vide (l'ancien plugin écrivait « 0 »).
- Site sans pays, ville ou GPS : les valeurs déjà présentes sur la photo ne sont pas touchées (protège un GPS venu d'un téléphone). Les valeurs « <No Country> » de l'ancien plugin ne sont plus écrites.
- Tant qu'aucune plongée n'est choisie, le lieu part sur « Autre site » ; il passe au site de la plongée dès qu'une plongée est choisie.
- Liste : un clic sur une ligne choisit la plongée ; les flèches ne font que parcourir (la fiche suit la ligne parcourue).
- « Annuler » revient à l'écran Plongées sans rien garder.
- Après l'écriture : bandeau avec le nombre de photos taguées et le rappel qu'« Annuler » dans Lightroom défait tout en une fois, sauf le GPS.

Liste des plongées **[Décidé]** : **la même que dans F4** (mêmes colonnes, regroupement par mois, recherche et filtres), en choix unique.

Suggestion **[Décidé]** : la ligne de la plongée dont les heures couvrent la sélection porte un repère « Couvre la sélection ». Le calcul applique le dernier décalage appliqué au boîtier dans « Créer les collections », quand il existe. **[Décidé, 2026-11-12]** Ce n'est qu'une aide : l'utilisateur choisit.


### F9 — (fusionné dans F8)

### F10 — Journal

Un fichier journal est conservé pour le diagnostic, consultable depuis les réglages. **[Proposé : conservé]**

**Emplacement [Décidé, 2026-11-12]** : l'application écrit son journal (`DiveLens App.log`) dans le dossier des journaux de Lightroom, à côté de celui du plugin (`DiveLens.log`) : `~/Library/Logs/Adobe/Lightroom/LrClassicLogs/` sur Mac, `%LOCALAPPDATA%\Adobe\Lightroom\Logs\LrClassicLogs\` sur Windows. « Afficher le journal » ouvre ce dossier, avec les deux fichiers côte à côte. Le niveau choisi (Normal / Détaillé) s'applique aux deux. Le journal de l'application est limité en taille : au-delà, l'ancien est gardé une fois puis remplacé.

### F11 — Exporter et importer les réglages

**[Décidé]** L'utilisateur peut exporter ses réglages dans un fichier pour les reprendre sur un autre ordinateur, ou après une réinstallation.

Pas dans le premier écran Réglages : chantier à part, après Subsurface. Les réglages tiennent déjà dans un seul fichier lisible. **[Décidé, 2026-11-12]**

- Contenu : emplacement de la base de plongée, ordre d'affichage, unités, langue, option altitude, marge par défaut, niveau de journal, et les réglages ajoutés plus tard. **[Proposé]**
- Format : un fichier texte lisible. **[Proposé]**
- Import : les réglages du fichier remplacent les réglages en cours, après confirmation. Si la base indiquée est introuvable sur cet ordinateur, l'application le signale et demande de la localiser. **[Proposé]**

Critères d'acceptation :
- Un fichier exporté par une version antérieure de DiveLens reste importable ; les réglages inconnus sont ignorés.

### F12 — Documentation utilisateur (site web)

**[Décidé]** Une documentation utilisateur en ligne accompagne la V1, dans les mêmes langues que l'application (anglais et français).

Contenu **[Proposé]** :
- installation et premier lancement (base MacDive, reprise des réglages) ;
- créer les collections : choix des plongées, horloge des appareils, « Suggérer », photos prises en surface ;
- tagger la sélection : données de plongée, lieu, autre site ;
- réglages : unités, langue, profondeur dans l'altitude, export des réglages ;
- dépannage : aucune photo trouvée, base introuvable, Lightroom déconnecté, horloge changée en cours de voyage ;
- confidentialité : la base de plongée est lue sans jamais être modifiée.

Liens depuis l'application vers la page d'aide concernée (un « ? » par écran) **[Proposé]**.

Diffusion **[Décidé]** : un seul texte source, publié à deux endroits :
- **embarqué dans l'application**, consultable hors connexion, toujours à jour avec la version installée ;
- **publié sur le site du plugin** (querton.com), pour les futurs utilisateurs et les moteurs de recherche.

### F13 — Mises à jour automatiques

**[Décidé : oui]** L'application vérifie s'il existe une nouvelle version de DiveLens et propose de l'installer.

Pas dans le premier écran Réglages : il faut d'abord la clé de signature des mises à jour et le circuit de publication. La section « Mises à jour » apparaîtra avec. **[Décidé, 2026-11-12]**

- Une mise à jour remplace **tout le paquet** : l'application et la partie Lightroom. **[Proposé]**
- Vérification au lancement, au plus une fois par jour, avec un réglage pour la désactiver et un bouton « Vérifier maintenant ». **[Proposé]**
- **Numérotation [Décidé]** : la première version publique est la **2.0.0**, pour marquer la rupture avec le plugin 0.x. Versions de développement en 0.x d'ici là.
- Rien ne s'installe sans l'accord de l'utilisateur : « Version 2.1 disponible — nouveautés — Installer ». **[Proposé]**
- Après installation, un message demande de redémarrer Lightroom, car Lightroom ne recharge sa partie DiveLens qu'au redémarrage. **[Proposé]**
- Paquets signés avec la clé de mise à jour DiveLens, pour qu'une mise à jour falsifiée soit refusée. Signature Apple et Windows : plus tard (section 9). **[Décidé]**
- Fichiers de mise à jour hébergés sur **GitHub Releases**. **[Décidé]** querton.com garde la présentation, la documentation et le lien de don.
- Mises à jour ouvertes à tous (plugin gratuit). **[Décidé]**

## 6. Inventaire des écrans (entrée des maquettes)

Description fonctionnelle seulement. La mise en page est une décision du propriétaire, à faire dans les maquettes.

**Cohérence [Décidé]** : une donnée affichée à plusieurs endroits (ex. la liste des plongées) a partout la même présentation.

**Principe [Décidé]** : les écrans sont repensés **de A à Z**, sans reprendre la structure des fenêtres Lua actuelles. On peut découper ou regrouper les écrans librement. Le seul critère est la facilité d'usage. La liste ci-dessous décrit des **besoins**, pas un découpage imposé.

| Écran | Ce qu'il montre | Actions |
|---|---|---|
| **Accueil / état** | Liaison Lightroom, nombre de photos sélectionnées, source de plongée et nombre de plongées | Aller vers chaque usage, ouvrir les réglages |
| **Liste des plongées** | Plongées (F4), indicateur de courbe | Rechercher, sélectionner une ou plusieurs plongées |
| **Créer les collections** (F5) | Plongées choisies avec compteur de photos chacune, appareils détectés, décalage et marge, total | Régler le décalage par appareil, « Suggérer », régler la marge, Appliquer |
| **Tagger la sélection** (F8) | Sélection Lightroom en direct (nombre, vignettes), plongée choisie, lieu (site de la plongée ou autre site : pays › lieu › site, GPS) | Cocher données de plongée et/ou lieu, choisir, Appliquer |
| **Réglages** (F2, F3, F7, F10) | Source de plongée et état de la base, langue, unités, option altitude et son explication, emplacement des collections, journal, à propos | Modifier, afficher le journal ; F11 et F13 plus tard |
| **Résultat d'application** | Collections créées, photos écrites, plongées sans photo | Fermer |

États à couvrir dans les maquettes : liaison perdue, base introuvable, plongée sans courbe, aucune photo trouvée, écriture en cours avec progression **[Décidé]**.

## 7. Champs ajoutés

**[Décidé, 2026-11-12]** — noms définitifs, jamais renommés (section 4). Ils s'ajoutent au panneau DiveLens de Lightroom après les champs existants.

| Champ | Contenu | Visible dans Lightroom |
|---|---|---|
| PhotoDepth | profondeur au moment de la photo | oui |
| PhotoTemp | température au moment de la photo | oui |
| PhotoElapsed | temps écoulé depuis le début de la plongée | oui |
| ClockOffset | décalage d'horloge appliqué à la photo, `±HH:MM` | oui **[Décidé]** |
| Source | application dont viennent les données de plongée : `MacDive` ou `Subsurface` | oui **[Décidé, 2026-11-12]** |

Source est écrit chaque fois que des données de plongée le sont : par « Créer les collections » sur toutes les photos rangées, et par « Tagger la sélection » quand « Données de plongée » est cochée. Un lieu posé seul ne le touche pas. Les photos déjà taguées le gardent vide jusqu'à une nouvelle application. **[Décidé, 2026-11-12]**

Les champs DiveLens restent dans le catalogue Lightroom et ne sont pas écrits dans le fichier photo. D'où l'option altitude (F7).

## 8. Sources de données

### Format commun **[Décidé]**

Chaque source passe par un **lecteur** qui traduit ses données dans un format commun. L'interface et le plugin ne connaissent que ce format. Chaque lecteur déclare ce qu'il sait fournir, et l'interface masque ce qui manque (température, note, GPS…).

Contenu du format commun pour une plongée :
- un identifiant stable dans la source ;
- numéro, début, fin, durée, profondeur max, températures max et min, note ;
- site, lieu, pays, latitude et longitude ;
- la courbe de plongée : temps, profondeur, température, si disponible.

### MacDive (V1)

- Base SQLite lue **en lecture seule**, à l'emplacement donné par l'utilisateur. La base peut se trouver dans iCloud.
- La courbe vient du **téléchargement brut de l'ordinateur** que MacDive conserve, décodé par libdivecomputer d'après le nom de l'ordinateur noté par MacDive (ex. « Shearwater Perdix 2 »). Les données Shearwater sont d'abord décompressées. La copie chiffrée propre à MacDive n'est pas utilisée (format non documenté).
- Une plongée **sans téléchargement brut** n'a pas de courbe : plongée saisie à la main, ou ordinateur ancien importé avant que MacDive ne conserve ce téléchargement (constaté sur d'autres bases : jusqu'à 424 plongées sur 530).
- Vérifié sur la base du propriétaire : 649 plongées sur 756 ont une courbe, toutes décodées (Shearwater Perdix 2, Mares Genius) ; les 107 autres sont saisies à la main. Les courbes sont identiques à celles que produit MacDive, à un détail près : pour Shearwater, libdivecomputer place le premier point à 10 s au lieu de 0 s. DiveLens suit libdivecomputer, donc le même calage que Subsurface.

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
- **Stockage git** : DiveLens lit, comme Subsurface, le dernier enregistrement du carnet (une branche), pas les fichiers du dossier. Il prend le carnet actif du dossier, ou le seul qu'il contient ; s'il y en a plusieurs sans carnet actif, l'état affiche « Plusieurs carnets dans ce dossier ». Pas de choix du carnet en V1.

**Passer de MacDive à Subsurface [Décidé, 2026-11-12]** — aucune migration :
- les décalages mémorisés sont propres à chaque application (F5) ;
- une relance retrouve la collection existante tant que le nom « numéro - site » est identique (742 sur 748 plongées chez le propriétaire) ; sinon une nouvelle collection est créée à côté ; rien n'est déplacé ni supprimé ;
- les photos déjà taguées gardent leurs valeurs jusqu'à une nouvelle application, qui écrit alors les heures et la durée de Subsurface ;
- la documentation (F12) le dit en une phrase.

### Sources suivantes **[Décidé]**

BlueDive et Submersion, après examen de leur stockage.

## 9. Points techniques à vérifier avant développement

- **Ordinateurs anciens sans téléchargement brut** dans MacDive : message à afficher (« pas de courbe disponible pour cet ordinateur »), à distinguer d'une plongée saisie à la main.
- **Ajout de champs** — vérifié dans le kit Lightroom 16 (2026-11-12) : ajouter des champs ne demande aucune migration ; les valeurs existantes sont conservées tant que la définition des 8 champs existants ne change pas. Le numéro de version par champ ne sert qu'à un changement incompatible. La version globale du schéma passe de 1 à 2, comme le kit le recommande à chaque évolution, sans routine de mise à jour. Preuve finale : test dans Lightroom (anciennes valeurs toujours visibles après rechargement).
- **Écriture de l'altitude** — vérifié (2026-11-12) : un plugin peut écrire l'altitude GPS (nombre, en mètres). « Annuler » ne la remet pas, comme le GPS (constaté le 2026-11-12). Sans effet tant que l'option est désactivée.
- **Recherche des photos par date** — constaté (2026-11-12) : la recherche du catalogue par date de prise de vue ne fonctionne qu'au jour près, et sur l'heure de prise de vue modifiée dans Lightroom, pas l'heure d'origine. Le plugin cherche donc par jours élargis, puis l'application filtre sur l'heure d'origine.
- **Filtrer par profondeur** : les champs de plugin sont du texte. Un filtre « photos sous 30 m » n'est probablement pas possible dans Lightroom ; seuls la recherche et la navigation par valeur le sont. À vérifier avant de promettre quoi que ce soit.
- **Températures nulles ou négatives** : le plugin actuel ne les écrit pas (plongée sous glace). **[Proposé : corrigé en V1]**
- **Signature** : V1 développée et testée **sans signature** **[Décidé]**. Apple (environ 99 $/an, notarisation automatique) recommandé avant la première version publique ; Windows : signature gratuite pour l'open source à viser (ex. SignPath Foundation).
- **Clé de signature des mises à jour** : créée dès le début, conservée hors du dépôt, jamais changée.
- **Windows** : la partie Lightroom du plugin bloque Windows aujourd'hui ; chemins, lancement de l'application et installation à adapter. Tests manuels Windows **[Décidé]** : machine virtuelle Windows 11 sur le Mac (Apple Silicon, donc Windows ARM) + testeurs sur de vrais PC (Intel/AMD). Vérifier que Lightroom Classic et Subsurface tournent dans la machine virtuelle.
- **Réglage « Application »** (F2) : MacDive ou Subsurface, avec l'emplacement du fichier ou du dossier correspondant ; sur Windows, seul Subsurface est proposé.
- **Fichier journal** — tranché (2026-11-12) : le kit Lightroom documente l'emplacement fixe du journal du plugin ; l'application écrit le sien à côté (F10).
- **Réglages de l'ancien plugin** — vérifié dans le kit Lightroom 16 (2026-11-12) : les préférences d'un plugin sont rangées sous son identifiant ; le nouveau plugin, de même identifiant, les lit donc directement.

## 10. À trancher

Aucune question ouverte à ce stade.

### Tranché

- Subsurface (2026-11-12) : premier ordinateur, plongées sans ordinateur sans courbe, découpage « Pays / Lieu / Site », valeurs telles que Subsurface les affiche, choix d'un fichier ou d'un dossier, un emplacement et des décalages mémorisés par application, pastille « Invalide », pas de migration au changement d'application (F2, F4, F5, section 8).
- Écran Réglages (2026-11-12) : unités « comme l'application de plongée » ou forcées ; Subsurface grisé « bientôt » ; journal de l'application à côté de celui du plugin ; ordre des plongées mémorisé sur la liste ; marge non mémorisée ; emplacement des collections au choix, « Dive Lens » par défaut ; export / import et mises à jour reportés (F2, F10, F11, F13).
- Fuseau horaire des photos : non utilisé par « Suggérer » en V1 (F5).
- Documentation : un seul texte, embarqué dans l'application et publié sur querton.com (F12).

- Unités : réglage métrique / impérial (F2) ; profondeurs écrites avec leur unité (F6).
- Liste des plongées : affichage et filtres redéfinis pendant le design (F4).
- Identification d'un appareil : modèle + numéro de série quand disponible (F5, proposé).
- Tagger la sélection : fusion des deux usages sur la sélection ; comportement actuel en V1, évolution notée (F8).
- Noms : « Créer les collections » et « Tagger la sélection » ; deux menus raccourcis dans Lightroom (section 3).
- Langues : multilingue (F2).
- Emplacement de la base : emplacement standard de MacDive pré-rempli (F2).
- DiveLens devient gratuit et open source (GPL v3), avec un lien de don Ko-fi plus tard ; plus de licence ni de mode d'essai ; mises à jour ouvertes à tous (F3, F13).
- Décalage d'horloge visible dans le panneau DiveLens de Lightroom (section 7).
