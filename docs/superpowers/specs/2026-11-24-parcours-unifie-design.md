# Parcours unifié de Dive Lens — conception

Date : 2026-11-24. Décisions prises par le propriétaire du produit les 22, 23 et 24 novembre 2026, à partir des maquettes « Dive Lens — parcours unifié » (version 19, dix écrans : cinq par chemin). Ce document remplace, pour les écrans et les menus, ce que la spécification (`docs/spec-v1.md`) dit de F1, F5 et F8 ; la spécification est réécrite en premier (phase 0).

Les maquettes sont privées et ne sont pas liées depuis le dépôt ; le propriétaire les fournit. Elles font foi pour la mise en page et pour les textes que ce document ne cite pas ; en phase 0, ces textes sont recopiés dans la spécification, en français et en anglais, pour que le dépôt se suffise.

## Intention

Aujourd'hui Dive Lens a deux fonctionnalités aux noms techniques, « Créer les collections » et « Tagger la sélection », chacune avec son menu Lightroom, son écran et sa logique. Elles font presque la même chose : rattacher des photos à des plongées et y écrire les données. Le plongeur doit comprendre la différence avant de commencer.

On les remplace par **un seul parcours en cinq étapes**, où la seule vraie question, « sur quelles photos ? », est posée à la première étape. La prochaine version publiée doit montrer ce changement : aucune version ne sort avant la fin de ce chantier.

## Objectifs

- Un parcours unique : Photos › Plongées › Horloges › Vérifier et appliquer › Récapitulatif.
- Une application sans barre latérale : un en-tête (logo, roue dentée), une barre d'état, les Réglages en plein écran.
- Un seul menu Lightroom dans Bibliothèque, un seul nom d'annulation.
- Ce qui est écrit se choisit à l'étape 4, par options, identiques dans les deux chemins.
- Chaque combinaison d'options de l'étape 4 couverte par des tests automatiques des règles d'écriture.

## Non-objectifs

- Aucune nouvelle application de plongée (BlueDive), aucune nouvelle langue.
- Pas d'effacement des données Dive Lens (reporté).
- Rien ne bouge dans `core/` au-delà de ce que les options de l'étape 4 exigent, ni dans `libdivecomputer-sys/`.
- Pas de refonte de l'écran Réglages : seuls son accès, son bouton « Retour » et la ligne d'altitude changent.

## Anti-objectifs (échec même si « ça marche »)

- **S'écarter du comportement d'aujourd'hui sans décision du propriétaire.** Règle du chantier : dans le doute, on garde ce que fait la version 2.1.1. Les seuls écarts permis sont listés dans « Ce qui change pour le plongeur ».
- Changer l'ordre des deux écritures, ce qui va dans chacune, ou le fait qu'un seul « Annuler » défait tout.
- Renommer ce que le guide des contributeurs interdit de renommer (identifiant du plug-in, identifiants des champs, ensemble « Dive Lens », noms des collections, identifiant de l'app).
- Dix écrans au lieu de cinq : chaque étape est **un seul écran**, codé une fois, qui s'adapte au chemin choisi à l'étape 1 (les maquettes dessinent ses deux variantes). Deux copies d'un même écran, une par chemin, sont un échec : c'est ce que la réorganisation de novembre a supprimé.
- Un test supprimé, désactivé ou assoupli pour faire passer un changement.
- Un état intermédiaire de `main` où l'application ne fonctionne plus : entre deux phases, les écrans en place doivent marcher.

## Contraintes

- Celles du guide des contributeurs : branches, commits, vérifications avant commit, CI verte sur macOS et Windows, données fictives.
- Le numéro de version ne change pas avant la fin : sur `main`, un changement de version déclenche une publication.
- Le propriétaire a autorisé, pour tout ce chantier, à pousser les branches et ouvrir les pull requests sans redemander. Les fusions restent à lui, jamais sans CI verte.
- Plug-in : Lua 5.1 tel que Lightroom l'embarque, sans `collectgarbage` ; pas de sous-dossiers ; entre les deux écritures, rien n'est envoyé à l'app et rien ne cède la main.
- Le protocole de liaison passe de 6 à 7 (voir « Barre d'état ») : les deux côtés montent ensemble.
- Textes : chaque texte en anglais et en français, « Dive Lens » en deux mots.

## Approches considérées

| Approche | Ce qu'elle donne | Ce qu'elle coûte |
|---|---|---|
| A. Nouvel habillage seulement : un parcours à l'écran, mais les deux circuits d'écriture actuels derrière, inchangés | Risque minimal sur ce qui est écrit | Les options de l'étape 4 n'existent pas dans les circuits actuels (collection depuis une sélection, autre site pour tout le catalogue, collections sans données de plongée) : il faudrait de toute façon les ajouter deux fois |
| **B. Moteur d'abord, écrans ensuite (retenue)** : les options de l'étape 4 sont ajoutées aux règles d'écriture et testées, les écrans actuels continuant d'envoyer les valeurs d'aujourd'hui ; puis l'habillage commun ; puis le parcours | Chaque phase laisse une application qui marche ; les règles d'écriture sont testées avant qu'un écran ne les utilise | Trois phases au lieu d'une ; le moteur porte un temps des options que personne n'envoie |
| C. Tout sur une seule branche longue | Pas d'état intermédiaire | Une seule pull request énorme, impossible à relire, régressions sans cause identifiable |

B est retenue par le concepteur (le propriétaire a délégué le « comment ») : c'est la méthode de la réorganisation de novembre, qui a bien fonctionné.

## Ce qui change pour le plongeur

Les seuls écarts voulus par rapport à la 2.1.1 :

1. Un seul menu, un seul écran en cinq étapes, plus de barre latérale.
2. Chemin « sélection » : une collection peut être créée pour la plongée choisie (aujourd'hui : jamais de collection).
3. Chemin « catalogue » : on peut décocher les détails de la plongée, le site ou les collections, et choisir un autre site, le même pour toutes les photos (aujourd'hui : tout est toujours écrit, avec le site de chaque plongée).
4. Lieu et GPS se cochent séparément sous « Détails du site ».
5. « Profondeur dans l'altitude GPS » quitte les Réglages : c'est une option de l'étape 4, décochée à chaque passage.
6. Chemin « sélection » : « Suggérer pour tous » apparaît (choix du concepteur, signalé au propriétaire).
7. Chemin « sélection » : un écran Récapitulatif remplace le bandeau après écriture.
8. La barre d'état affiche la taille du catalogue.

## Ce qui est décidé

### Cadre de l'application

- **En-tête** : logo et « Dive Lens » à gauche, roue dentée à droite. Pas de barre latérale.
- **Barre d'état**, en bas. À gauche, le groupe Lightroom : état de la liaison, « Catalogue : nom », « Taille du catalogue : N éléments », « Photos sélectionnées : N » (« aucune » à zéro). À droite, le groupe carnet : « Application de carnet : nom », « Plongées enregistrées : N ». Les états de liaison (connecté, déconnecté, redémarrer Lightroom) sont ceux de la barre latérale actuelle.
- **Taille du catalogue** : nombre de toutes les photos du catalogue, compté d'un seul appel par le plug-in. Mesuré le 2026-11-24 sur un catalogue réel : près de 50 000 photos en 0,10 s au premier comptage, 0,02 à 0,03 s ensuite. Recompté à la connexion et à chaque retour sur l'étape 1. Le comptage dossier par dossier, qui donne un autre chiffre, n'est pas utilisé.
- **Réglages** : la roue dentée les ouvre en plein écran, à la place du parcours, avec « Retour ». Au retour, le plongeur retrouve l'étape où il était et ses choix ; exception : si le carnet lu a changé (autre application ou autre fichier, règle F2 « Un seul carnet à la fois »), le parcours repart à l'étape 1. La roue dentée est inactive pendant une écriture. Les autres réglages (unités, langue…) s'appliquent au retour.
- **Menus Lightroom** : Bibliothèque › Modules externes › « Ouvrir Dive Lens… » (EN « Open Dive Lens… »), seul. Aide › Modules externes › « Réglages de Dive Lens… » reste. Le gestionnaire de modules externes ne change pas.
- **Annulation** : un seul nom, « Dive Lens » ; le menu Édition affiche « Annuler Dive Lens » (EN « Undo Dive Lens »). Les textes d'erreur qui citent l'ancien nom suivent.
- L'application s'ouvre toujours sur l'étape 1, sauf depuis le menu Aide (Réglages).

### Fil d'étapes

Cinq étapes rappelées en haut de chaque écran, même présentation que le fil actuel de « Créer les collections » : une étape déjà faite est cliquable et y ramène, sauf depuis le Récapitulatif. Sous le fil, une ligne « Photos : … » résume les choix déjà faits. Boutons « ← Précédent » et « Suivant → » dans la barre du bas, texte à gauche, boutons à droite.

### Étape 1 — Photos

« Sur quelles photos travailler ? » Deux cartes :

- **Tout le catalogue** (EN « Entire catalog ») : « Vous cochez une ou plusieurs plongées : chaque photo va dans la plongée qui correspond à son heure. » La sélection Lightroom est ignorée.
- **Les N photos sélectionnées dans Lightroom** (EN « Lightroom selection ») : résumé de la sélection (appareils, dates), les vignettes en direct des photos sélectionnées, jusqu'à six, comme dans « Tagger la sélection » aujourd'hui (seulement sur cette carte, pas aux étapes suivantes), « Vous choisissez une plongée : les N photos y sont toutes rattachées, quelle que soit leur heure. »

Sans sélection dans Lightroom, la carte « sélection » est toujours affichée, grisée, avec « Aucune photo sélectionnée » (EN « No photos selected ») ; elle s'allume dès que des photos sont sélectionnées. L'étape n'est jamais sautée.

### Étape 2 — Plongées

La liste des plongées actuelle (colonnes, regroupement par mois, recherche, filtres, tri, fiche).

- **Catalogue** : choix multiple, comme l'étape 1 de « Créer les collections ». « Suivant » grisé tant qu'aucune plongée n'est cochée.
- **Sélection** : choix unique, comme « Tagger la sélection », avec le repère « Couvre la sélection » calculé avec les décalages mémorisés. Première ligne de la liste : **« Aucune plongée · lieu seulement »** (EN « No dive — location only »), pour poser seulement un lieu (bateau, plage…).

### Étape 3 — Horloges

Le bloc « Horloge des appareils » actuel (cartes par appareil, curseur, champ heures:minutes, « Suggérer », étiquette « décalage mémorisé », jauge), puis l'aperçu par plongée.

- **Catalogue** : comme l'étape 2 actuelle de « Créer les collections » : « Ignorer » par appareil, « Suggérer pour tous », « Inclure les photos prises en surface » (marge, repart de 0 min à chaque fois, non mémorisée), aperçu par plongée, bloc « Hors plongée · N photos ». Ce bloc est un simple texte : il explique que, pour rattacher quand même ces photos ou leur poser un lieu, il faut les sélectionner dans Lightroom puis choisir le chemin « sélection » à l'étape 1. Aucun lien, aucune action sur la sélection de Lightroom.
- **Sélection** : pas de case « Ignorer » (pour écarter des photos, on les retire de la sélection dans Lightroom), pas de marge. « Suggérer pour tous » présent. Une note rappelle que toutes les photos sont rattachées et que l'horloge ne sert qu'à leur donner une profondeur ; les photos hors de l'horaire de la plongée sont listées (« rattachée quand même : infos et lieu, sans profondeur »).
- **« Aucune plongée »** : l'étape est sautée. « Suivant » à l'étape 2 mène à l'étape 4, « Précédent » à l'étape 4 ramène à l'étape 2, et « Horloges » est grisée dans le fil.

### Étape 4 — Vérifier et appliquer

« Ce que Dive Lens écrit dans Lightroom ». Les options sont **cochées à chaque passage**, sauf l'altitude, **décochée à chaque passage** ; rien n'est mémorisé d'un passage à l'autre.

```
Métadonnées
  ☑ Détails de la plongée
      ☐ Profondeur dans l'altitude GPS
  ☑ Détails du site
      (•) Site de chaque plongée   ( ) Autre site… pays › lieu › site
      ☑ Lieu      ☑ GPS
Collections
  ☑ Créer les collections           emplacement : lien vers Réglages
À savoir avant d'appliquer : …      Unités : m, °C
                    [← Précédent]  [Appliquer à N photos]
```

- **Détails de la plongée** : n°, début, fin, durée, profondeur max, températures, note, source ; sur les photos prises pendant la plongée, profondeur, température, temps écoulé et décalage d'horloge.
- **Profondeur dans l'altitude GPS** : sous-option de « Détails de la plongée », avec l'explication courte de la maquette. Le réglage du même nom disparaît de l'écran Réglages.
- **Détails du site** : « Site de chaque plongée » (catalogue) ou « Site de la plongée : nom » (sélection), ou « Autre site », le même pour toutes les photos, choisi par pays › lieu › site parmi les sites du carnet. Sous-options « Lieu » (pays, ville, lieu) et « GPS ».
- **Créer les collections** : une collection « numéro - site » par plongée qui a des photos ; dans le chemin « sélection », celle de la plongée choisie, qui reçoit toutes les photos sélectionnées. Emplacement réglé dans Réglages, rappelé ici.
- **À savoir avant d'appliquer** : plongées sans photo, photos hors plongée qui ne reçoivent rien (catalogue), photos sans profondeur (sélection), unités.
- Le bouton affiche le nombre réel de photos. Rappel : « Annulable en une fois dans Lightroom, GPS et altitude compris ».
- **« Aucune plongée »** : « Détails de la plongée » et « Créer les collections » sont visibles mais grisées ; seul « Détails du site » agit, avec « Autre site » obligatoire. « Appliquer » attend qu'un site soit choisi.

### Étape 5 — Récapitulatif

Dans les deux chemins : ce qui a été écrit (photos taguées, avec profondeur, collections créées avec leur nombre de photos), les avertissements (plongée sans photo, photos hors plongée, photos sans profondeur) et le rappel d'annulation actuel. Boutons : catalogue, « Retour aux plongées » et, quand une plongée n'a reçu aucune photo, « Traiter la plongée N à part » ; sélection, « Terminer ».

### Sélection Lightroom : suivie en direct

Dans le chemin « sélection », la sélection est suivie en direct jusqu'à « Appliquer », exactement comme « Tagger la sélection » aujourd'hui : cartes et compteurs recalculés à chaque changement, « Appliquer » écrit sur les photos sélectionnées au moment du clic, refus « La sélection a changé dans Lightroom… » dans les mêmes conditions qu'aujourd'hui. Si la sélection devient vide en cours de parcours, on reste sur l'étape, « Appliquer » grisé avec le message actuel.

### Libellés

| Français | Anglais |
|---|---|
| Photos › Plongées › Horloges › Vérifier et appliquer › Récapitulatif | Photos › Dives › Clocks › Review & Apply › Summary |
| Tout le catalogue | Entire catalog |
| Sélection Lightroom / Aucune photo sélectionnée | Lightroom selection / No photos selected |
| Aucune plongée — lieu seulement | No dive — location only |
| Métadonnées / Détails de la plongée / Détails du site | Metadata / Dive details / Site details |
| Profondeur dans l'altitude GPS | Copy depth to GPS altitude |
| Site de la plongée / Autre site | Dive's site / Other site |
| Lieu / GPS / Créer les collections | Location / GPS / Create collections |
| Réglages / Retour | Settings / Back |

Les textes non listés reprennent ceux de l'application actuelle ou de la maquette. Le propriétaire les affinera pendant ses essais.

## Règles d'écriture

Principe : chaque chemin garde **ses règles actuelles**, y compris là où elles diffèrent de l'autre. Avec toutes les options cochées et l'altitude comme elle était réglée, le chemin « catalogue » écrit exactement ce que « Créer les collections » écrit aujourd'hui ; le chemin « sélection » écrit ce que « Tagger la sélection » écrit avec « Données de plongée » et « Lieu » cochées, plus la collection.

Différences entre chemins, conservées telles quelles :

| Sujet | Catalogue | Sélection |
|---|---|---|
| Photos écrites | celles rangées dans une plongée (dans la plongée ou la marge), désignées une à une ; appareils ignorés et photos hors plongée non touchés | toutes les photos sélectionnées au moment du clic |
| Photo hors de la plongée | (marge) infos de plongée, lieu et décalage d'horloge, sans données par photo | infos de plongée et lieu, sans données par photo ni décalage d'horloge |
| Même plongée, photo désormais sans profondeur | données par photo réécrites, altitude écrite par Dive Lens effacée | rien n'est touché |
| Courbe illisible | « Appliquer » échoue | écrit les infos de plongée sans données par photo |
| Décalages mémorisés après « Appliquer » | appareils ayant une photo rangée | appareils ayant une photo dans la plongée |

Effet des options, dans les deux chemins :

- **« Détails de la plongée » décochée** : aucun champ de plongée, aucune donnée par photo, aucun décalage d'horloge écrit sur la photo, aucune altitude (la sous-option est grisée), et aucune valeur d'une autre plongée effacée : sans cette option, Dive Lens ne touche pas aux champs de plongée. Les photos sont quand même rattachées aux plongées pour le site et les collections. Les décalages restent mémorisés selon la règle du chemin.
- **« Profondeur dans l'altitude GPS »** : cochée ou non, mêmes règles que le réglage actuel activé ou désactivé (F7, repère d'altitude compris).
- **« Détails du site » décochée** : ni pays, ni ville, ni lieu, ni GPS. « Lieu » seul ou « GPS » seul : seulement la moitié cochée. Les deux sous-options décochées équivalent à l'option décochée. Un site sans pays, ville ou GPS ne touche pas aux valeurs déjà présentes, comme aujourd'hui.
- **« Autre site »** : le site choisi remplace celui de la plongée pour toutes les photos écrites. Ses coordonnées GPS restent modifiables avant d'appliquer, comme dans « Tagger la sélection » aujourd'hui, dans les deux chemins.
- **« Créer les collections » décochée** : aucune collection créée ni modifiée. Cochée dans le chemin « sélection » : même nom, même emplacement, même création de l'ensemble et mêmes refus (ensemble introuvable) que dans le chemin « catalogue » ; une photo déjà dans la collection n'y est pas ajoutée en double.
- Il faut au moins une option cochée parmi les trois : sinon « Appliquer » est grisé avec une invite.
- Deux écritures, un seul nom d'annulation, GPS et altitude dans la seconde : inchangé. La création des collections reste dans la première.

## Barre d'état : taille du catalogue

Le plug-in n'envoie aujourd'hui que le nom du catalogue, à la connexion. La taille demande un nouvel échange (l'app demande, le plug-in compte et répond), donc le protocole 7. Un plug-in resté en version 6 face à une app en 7 retombe sur l'état existant « redémarrez Lightroom ».

## Phases

Une pull request par étape cohérente, CI verte, fusion par le propriétaire.

- **Phase 0 — Spécification.** Réécrire `docs/spec-v1.md` : F1, F2 (roue dentée, altitude), F5 devient le parcours, F7 (option par passage), F8 fusionné dans F5, inventaire des écrans (section 6). Chaque décision de ce document y est datée. Relue par le propriétaire.
- **Phase 1 — Moteur et plug-in.** Options de l'étape 4 dans les règles d'écriture des deux chemins, collection depuis une sélection, « autre site » pour le catalogue, lieu seul sans plongée conservé, taille du catalogue et protocole 7. Les écrans actuels continuent de fonctionner en envoyant les valeurs d'aujourd'hui. Tests : toutes les combinaisons d'options pour chaque chemin, dans les tests des règles du plug-in et du moteur.
- **Phase 2 — Cadre.** En-tête, barre d'état, Réglages en plein écran avec « Retour ». Les deux écrans actuels restent accessibles provisoirement (le moyen est laissé au constructeur, par exemple deux onglets temporaires).
- **Phase 3 — Parcours.** Les cinq étapes, le menu unique, le nom d'annulation unique, l'altitude retirée des Réglages, suppression des anciens écrans et de leurs textes.
- **Phase 4 — Recette et documentation.** Essai du parcours réel par le propriétaire, corrections, puis réécriture des vérifications Lightroom (tout ce qui existe encore des séries actuelles, plus le parcours), du guide utilisateur français et anglais et du README. Recette complète sur Mac par le propriétaire, pré-version bêta pour les testeurs Windows (avec la vérification d'« Annuler » en attente sur Windows), puis version mineure.

Essais manuels du propriétaire : seulement en phase 4. La mesure du catalogue, prévue comme premier essai, est faite.

## Laissé au constructeur

- Tout le découpage interne : fichiers, noms, forme des messages entre l'app et le plug-in, fusion ou non des deux demandes d'écriture en une seule, à condition que les tests actuels des règles d'écriture continuent de passer sans être modifiés pour les cas « toutes options cochées ».
- La forme exacte de l'échange « taille du catalogue ».
- Le sort du réglage d'altitude dans le fichier de réglages : les anciens fichiers et les réglages exportés qui le contiennent doivent rester lisibles ; la valeur est ignorée.
- Le moyen provisoire d'atteindre les deux anciens écrans en phase 2.
- L'ordre et la taille des pull requests dans une phase.
- Les tests de contrat à adapter (noms d'écran, nom d'annulation, liste des commandes) : adaptés à la nouvelle réalité, jamais assouplis.

## Choix faits par le concepteur

À vérifier par le propriétaire en lisant ; aucun n'a été décidé par lui explicitement.

- « Suggérer pour tous » dans le chemin « sélection ».
- « Terminer » (récapitulatif du chemin « sélection ») ramène à l'étape 1. « Retour aux plongées » ramène à l'étape 2 sans plongée cochée, comme aujourd'hui. « Traiter la plongée N à part » ramène à l'étape 2 avec cette seule plongée cochée.
- Au moins une option cochée à l'étape 4 pour pouvoir appliquer.
- « Détails de la plongée » décochée : rien n'est effacé des champs de plongée déjà présents.
- « Autre site » : GPS modifiable dans les deux chemins.
- « Aucune plongée » : options sans objet grisées plutôt que masquées (annoncé au propriétaire, sans objection).
- Aide › « Réglages de Dive Lens… » conservé ; nom d'annulation « Dive Lens » (annoncés, sans objection).
- Recomptage du catalogue à la connexion et au retour sur l'étape 1 (annoncé, sans objection).
- Le bandeau « Le carnet de plongée a changé… » s'affiche sous le fil d'étapes de l'étape 1.
- Avec « Autre site », la collection garde le nom « numéro - site » de la plongée, comme aujourd'hui : seuls le lieu et le GPS écrits sur les photos changent.
- État du parcours : le menu « Ouvrir Dive Lens… » choisi pendant un parcours ramène à l'étape 1 en gardant les choix, comme le menu actuel garde les plongées cochées ; pendant une écriture, il ne quitte pas l'écran, comme aujourd'hui. Changer de chemin à l'étape 1 repart de zéro pour les étapes suivantes (plongées, horloges, options) : un seul état de parcours, pas un par chemin.
- Chemin « catalogue », « Détails de la plongée » décochée : une courbe illisible fait toujours échouer « Appliquer », comme aujourd'hui.
- Plusieurs plongées sans photo : le bouton actuel « Traiter les plongées à part » est conservé, avec son comportement.
- L'explication longue de l'option altitude (quatre points, F7) est remplacée à l'étape 4 par le texte court de la maquette ; le texte long passe dans le guide utilisateur.
