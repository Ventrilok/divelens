# Dive Lens — Guide d'utilisation

Dive Lens relie vos photos sous-marines dans Adobe Lightroom Classic à votre carnet de plongée. Il lit votre carnet **MacDive** (Mac) ou **Subsurface** (Mac et Windows). En un parcours de cinq étapes, vous choisissez les photos et les plongées, réglez l'horloge des appareils, vérifiez ce qui sera écrit, puis appliquez : Dive Lens range les photos dans une collection par plongée et y écrit les infos de plongée : numéro, heures, profondeur, températures, note et site. Quand la plongée a une courbe, chaque photo reçoit en plus **la profondeur, la température de l'eau et le temps écoulé au moment où elle a été prise**.

Dive Lens est gratuit et open source (GPL v3).

> Brouillon à relire. Les exemples (plongées, sites, appareils) sont fictifs.

---

## 1. Installer

Il vous faut Lightroom Classic et MacDive ou Subsurface sur le même ordinateur. Sur Windows, seul Subsurface est proposé.

**Vous utilisiez l'ancien module Dive Lens (0.9 et avant) ?** Avant d'utiliser l'installateur, retirez-le : dans Lightroom, **Fichier › Gestionnaire de modules externes…**, choisissez Dive Lens, **Supprimer**, puis **Terminé**. Si le bouton **Supprimer** est grisé, l'ancien module est déjà là où l'installateur le remplacera : il n'y a rien à faire. Vos photos gardent leurs infos de plongée, et l'application reprend vos réglages (emplacement du carnet et ordre des plongées) à son premier lancement. Dive Lens est désormais gratuit : plus de clé de licence.

### Avec l'installateur

1. Quittez Lightroom.
2. Téléchargez l'installateur de votre système : `DiveLens-<version>-macOS.pkg` ou `DiveLens-<version>-Windows-Setup.exe`.
3. Ouvrez-le et suivez les étapes. Il installe Dive Lens pour votre compte seulement, sans mot de passe administrateur.
4. Lancez Lightroom : Dive Lens est dans **Bibliothèque › Modules externes**. Il n'y a rien à ajouter dans le gestionnaire de modules externes.

Sur Windows, l'installateur n'est pas encore signé : Windows peut afficher **« Windows a protégé votre ordinateur »**. Cliquez sur **Informations complémentaires**, puis **Exécuter quand même**.

### Avec le zip, à la main

1. Téléchargez le zip de votre système : `DiveLens-<version>-macOS.zip` ou `DiveLens-<version>-Windows.zip`.
2. Décompressez-le. Vous obtenez un dossier **DiveLens.lrplugin** (sur Mac, il apparaît comme un seul fichier). L'application Dive Lens est rangée dedans.
3. Placez ce dossier à un endroit où il restera, par exemple dans Documents. Lightroom le charge depuis cet endroit à chaque démarrage. Si vous utilisiez l'ancien module, quittez Lightroom et remplacez plutôt l'ancien dossier DiveLens.lrplugin par le nouveau, au même endroit : l'étape 4 est alors inutile.
4. Dans Lightroom : **Fichier › Gestionnaire de modules externes… › Ajouter**, choisissez le dossier **DiveLens.lrplugin**, puis **Terminé**.

Sur Mac, la première fois que Lightroom ouvre l'application, macOS peut demander de confirmer l'ouverture d'une application téléchargée : cliquez sur **Ouvrir**. Sur Windows, l'application n'est pas encore signée : si Windows affiche **« Windows a protégé votre ordinateur »**, cliquez sur **Informations complémentaires**, puis **Exécuter quand même**. Une seule fois suffit.

### WebView2 sur Windows

L'application a besoin de **WebView2** de Microsoft, déjà présent sur Windows 11 et sur la plupart des Windows 10. Si l'application ne s'ouvre pas, installez « Microsoft Edge WebView2 Runtime » depuis le site de Microsoft.

### Désinstaller

Vos photos gardent leurs infos de plongée et leurs collections dans le catalogue : elles réapparaissent si vous réinstallez Dive Lens.

**Sur Mac**

1. Quittez Lightroom.
2. Dans le Finder, menu **Aller › Aller au dossier…**, collez `~/Library/Application Support/Adobe/Lightroom/Modules`, puis mettez **DiveLens.lrplugin** à la corbeille. Si vous aviez installé Dive Lens avec le zip, mettez à la corbeille le dossier DiveLens.lrplugin là où vous l'aviez placé, après l'avoir retiré dans **Fichier › Gestionnaire de modules externes…** (**Supprimer**).
3. Facultatif, pour effacer aussi les réglages de Dive Lens : mettez à la corbeille le dossier `~/Library/Application Support/com.querton.divelens.desktop`.

**Sur Windows**

1. Quittez Lightroom.
2. **Paramètres › Applications › Applications installées › Dive Lens › Désinstaller**. Si vous aviez installé Dive Lens avec le zip, retirez-le dans **Fichier › Gestionnaire de modules externes…** (**Supprimer**), puis supprimez le dossier DiveLens.lrplugin là où vous l'aviez placé. Une fois mise à jour par Dive Lens, une copie installée avec le zip figure elle aussi dans **Applications installées** : désinstallez-la de là.
3. Facultatif, pour effacer aussi les réglages de Dive Lens : supprimez le dossier `%APPDATA%\com.querton.divelens.desktop`.

---

## 2. Ouvrir Dive Lens

Dive Lens n'a pas de fenêtre dans Lightroom. Tout se passe dans l'application Dive Lens, que Lightroom ouvre pour vous :

- **Bibliothèque › Modules externes › Ouvrir Dive Lens…** ouvre l'application à l'étape 1 du parcours (section 3), que des photos soient sélectionnées ou non.
- **Aide › Modules externes › Réglages de Dive Lens…** l'ouvre sur les **Réglages** (section 9).

L'entrée Dive Lens de **Fichier › Gestionnaire de modules externes…** a aussi les boutons **Ouvrir Dive Lens** et **Ouvrir les réglages**, et les liens vers le site web de Dive Lens et vers son code source sur GitHub.

Si l'application est déjà ouverte, le menu ramène sa fenêtre au premier plan. Choisi pendant le parcours, **Ouvrir Dive Lens…** ramène à l'étape 1 en gardant vos choix ; choisi sur le récapitulatif, il commence un nouveau passage.

Lightroom doit rester ouvert pendant que vous utilisez l'application. La première fois, passez par les **Réglages** (section 9) : l'application de plongée et l'emplacement de votre carnet.

### La fenêtre

- **En haut** : Dive Lens à gauche ; à droite, la roue dentée ouvre les **Réglages** (section 9). Elle est grisée pendant que Dive Lens écrit dans Lightroom.
- **En bas**, la barre d'état. À gauche, Lightroom :
  - **Lightroom connecté**, **Lightroom déconnecté** ou **Redémarrez Lightroom** ;
  - **Catalogue :** le nom du catalogue ouvert dans Lightroom ;
  - **Taille du catalogue :** le nombre de photos qu'il contient, recompté chaque fois que vous revenez à l'étape 1 ;
  - **Photos sélectionnées :** combien sont sélectionnées dans Lightroom, en direct (**aucune** quand il n'y en a pas).
- À droite de la barre d'état, votre carnet : **Application de carnet :** MacDive ou Subsurface, et **Plongées enregistrées :** combien de plongées il contient.

Si la barre d'état affiche **Lightroom déconnecté**, rien ne peut être écrit : vérifiez que Lightroom est ouvert et que le module est activé dans le gestionnaire de modules externes. Si elle affiche **Redémarrez Lightroom**, Dive Lens a été mis à jour pendant que Lightroom était ouvert : quittez Lightroom et relancez-le.

---

## 3. Le parcours

Cinq étapes, rappelées en haut : **Périmètre › Plongées › Horloges › Vérifier et appliquer › Récapitulatif**. **Suivant** avance et **Précédent** revient en arrière ; un clic sur une étape déjà faite y revient aussi, sauf depuis le récapitulatif. Sous les étapes, une ligne qui commence par **Photos :** rappelle vos choix. Rien n'est écrit dans Lightroom avant que vous cliquiez sur **Appliquer à N photos** à l'étape 4.

La barre de **Précédent** et **Suivant** (ou **Appliquer à N photos**) reste en bas de la fenêtre, juste au-dessus de la barre d'état, et les étapes restent en haut : quand la fenêtre est petite, seul le contenu entre les deux défile. Rien n'est jamais caché sous la barre.

### Deux chemins

L'étape 1 pose la seule vraie question : sur quelles photos ?

- **Tout le catalogue** : vous cochez une ou plusieurs plongées, et Dive Lens cherche dans tout le catalogue les photos prises pendant chacune. Chaque photo va dans la plongée qui correspond à son heure. Pour un voyage entier ou une série de plongées. Les photos sélectionnées dans Lightroom sont ignorées.
- **Les photos sélectionnées dans Lightroom** : celles que vous avez sélectionnées dans Lightroom. Vous choisissez une plongée, et elles y sont toutes rattachées, quelle que soit leur heure. Pour des photos sans heure fiable, pour rattacher des photos à une plongée à la main, ou pour ne leur poser qu'un lieu (section 7).

Choisir l'autre chemin à l'étape 1 fait repartir de zéro les étapes suivantes : plongées, horloges et options.

### Étape 1 — Périmètre

« Sur quelles photos travailler ? » Deux cartes : **Tout le catalogue**, ou les photos sélectionnées dans Lightroom. La carte de la sélection suit Lightroom en direct : les appareils et le jour des photos sélectionnées, et jusqu'à six vignettes. Sans photo sélectionnée, elle est grisée et indique **Aucune photo sélectionnée** : sélectionnez des photos dans Lightroom et elle s'allume. Choisissez une carte, puis **Suivant**.

### Étape 2 — Plongées

La liste montre toutes les plongées de votre carnet, regroupées par mois. Cherchez par numéro, site, lieu ou pays, et affinez avec les filtres (**Période**, **Pays**, **Profondeur**, **Note**, **Avec courbe**, **Avec GPS**, **Avec notes**). **Trier** ordonne la liste (**Plus récentes d'abord**, **Plus anciennes d'abord**, **Profondeur max**) ; Dive Lens retient votre choix. Un clic sur une plongée affiche sa fiche à droite : courbe, durée, profondeur max, températures, note, GPS du site avec une carte, et vos notes.

Chaque ligne se termine par trois repères, en couleur quand la plongée en a, grisés sinon : **Courbe**, **GPS** (son site a une position GPS) et **Notes**. Survolez un repère pour le lire, par exemple « GPS du site disponible » ou « Pas de notes ». La colonne **GPS** et le filtre **Avec GPS** n'apparaissent que si l'application de plongée donne la position des sites, comme MacDive et Subsurface.

- **Tout le catalogue** : cochez une ou plusieurs plongées, puis **Suivant**. La barre indique combien sont cochées et leurs jours ; **Tout désélectionner** les décoche.
- **Les photos sélectionnées dans Lightroom** : cliquez sur une plongée, puis **Suivant**.
  - La plongée dont les heures couvrent la sélection porte le repère **Couvre la sélection**. Il tient compte du dernier décalage appliqué à chaque appareil, puis des décalages réglés à l'étape 3. Ce n'est qu'une aide : c'est vous qui choisissez.
  - La première ligne, **Aucune plongée — lieu seulement**, sert à ne poser qu'un lieu sur les photos (section 7).

Une plongée **sans courbe** (saisie à la main, ou venant d'un ancien ordinateur de plongée dont MacDive n'a pas gardé le téléchargement) fonctionne quand même : ses photos reçoivent les infos de plongée et le lieu, mais ni profondeur, ni température, ni temps.

### Étape 3 — Horloges

**Horloge des appareils.** L'horloge d'un appareil est rarement exactement à l'heure de l'ordinateur de plongée, et souvent encore à l'heure de la maison. Chaque appareil a sa carte (deux boîtiers du même modèle sont distingués par leur numéro de série). Réglez l'écart entre son horloge et celle du carnet :

- **Suggérer** propose le décalage qui place le plus de photos dans les plongées. Il essaie chaque quart d'heure entre −14 h et +14 h, et retrouve donc aussi un fuseau horaire oublié.
- **Suggérer pour tous** le fait pour tous les appareils d'un coup, sans toucher à ceux réglés à la main ni aux appareils ignorés.
- Vous pouvez aussi saisir le décalage (heures:minutes) ou l'ajuster vous-même.
- Chaque carte part du dernier décalage appliqué à cet appareil, dans l'un ou l'autre chemin (**décalage mémorisé**) ; sans décalage connu, elle part de 0:00. Les décalages sont mémorisés séparément pour chaque application de plongée. **Réglages › Décalages d'horloge mémorisés** les liste et oublie celui d'un appareil (section 9).

**Aperçu par plongée.** Les compteurs suivent vos réglages : photos par plongée, appareils, et les données par photo que donne chaque plongée (**Profondeur · temp. · temps**, **Pas de courbe : infos de plongée seules**, **Photos de surface seulement**). Pour une plongée sans photo, son nombre, 0, et **Aucune photo — horloge changée ?** sont écrits en orange : c'est souvent le signe d'une horloge changée pendant le voyage (section 11).

Dans le chemin **Tout le catalogue** :

- Dive Lens cherche dans tout le catalogue les photos prises sur la période des plongées cochées.
- **Ignorer** écarte un appareil, par exemple un téléphone resté sur le bateau : ses photos ne sont ni comptées ni taguées.
- **Inclure les photos prises en surface** : jusqu'à N minutes avant et après chaque plongée, pour les photos prises sur le bateau juste avant la mise à l'eau ou juste après la sortie. 0 minute par défaut, et retour à 0 à chaque passage (sauf **Traiter la plongée N à part** et **Traiter ces plongées à part**, qui la gardent). Ces photos reçoivent les infos de plongée, sans profondeur.
- **Hors plongée** : les photos de la période qui ne tombent dans aucune plongée, même avec la marge (à terre, sur le bateau…). Elles ne reçoivent rien (section 7).

Dans le chemin **Les photos sélectionnées dans Lightroom** :

- Les photos sont rattachées à la plongée choisie, quelle que soit leur heure. L'horloge sert seulement à leur donner une profondeur : seules celles prises pendant la plongée en reçoivent une.
- Une carte par appareil de la sélection. Pas de case **Ignorer** (retirez plutôt la photo de la sélection dans Lightroom), ni de marge de surface.
- **Hors de l'horaire de la plongée** liste les photos prises avant ou après la plongée, par exemple « Prise à 15:45, 7 minutes après la sortie de l'eau. Rattachée quand même à la plongée : elle reçoit ses infos et son lieu, sans profondeur. »
- Une plongée sans courbe affiche « Pas de courbe : pas de données par photo » à la place de l'horloge des appareils, et une plongée dont la courbe ne peut pas être lue « Courbe illisible : pas de données par photo » : les infos de plongée sont alors écrites sans données par photo.
- Avec **Aucune plongée — lieu seulement**, cette étape est sautée : **Horloges** est grisée dans les étapes.

### Étape 4 — Vérifier et appliquer

« Ce que Dive Lens écrit dans Lightroom. » Les mêmes options dans les deux chemins. Elles sont toutes cochées à chaque passage, sauf **Profondeur dans l'altitude GPS**, décochée à chaque passage : rien n'est mémorisé d'un passage à l'autre. À droite de chaque option, son effet sur ce passage, par exemple « 12 photos · 9 avec profondeur ». La [section 4](#4-quelles-options-pour-quel-besoin) montre ce que chaque combinaison d'options écrit sur les photos, et quelles options choisir pour quel besoin.

**Métadonnées**

- **Détails de la plongée** : numéro de plongée, début, fin, durée, profondeur max, températures, note et source (l'application de plongée). Sur les photos prises pendant la plongée, aussi la profondeur, la température et le temps écoulé, et le décalage d'horloge.
  - **Profondeur dans l'altitude GPS**, en dessous, grisée quand **Détails de la plongée** est décochée : écrit aussi la profondeur dans l'altitude GPS, en valeur négative, en mètres, sur les photos qui ont une profondeur (section 6).
- **Détails du site** : le lieu et le GPS du site de la plongée, ou d'un autre site de votre carnet.
  - **Tout le catalogue** : **Site de chaque plongée**, ou **Autre site, le même pour toutes les photos…**.
  - **Les photos sélectionnées dans Lightroom** : **Site de la plongée : <nom>**, ou **Autre site…**.
  - **Autre site** : choisi par pays › lieu › site parmi les sites de votre carnet. Son GPS est modifiable avant d'appliquer, en degrés décimaux (par exemple `12.5000` et `-45.2500`). Les collections gardent le nom du site de leur plongée.
  - **Lieu** : pays, ville et lieu. **GPS** : la position. Décochez l'un pour n'écrire que l'autre. Un site sans GPS laisse en place le GPS déjà présent sur la photo.

**Collections**

- **Créer les collections** (une option de cette étape, et non plus un menu de Lightroom) : dans le chemin **Tout le catalogue**, une collection « numéro - site » par plongée qui a des photos ; dans le chemin **Les photos sélectionnées dans Lightroom**, la collection de la plongée choisie, qui reçoit toutes les photos sélectionnées. Une photo déjà dans la collection n'y est pas ajoutée deux fois. L'emplacement des collections se règle dans les Réglages : le lien **Emplacement dans Réglages** les ouvre (section 9).

**À savoir avant d'appliquer** liste ce qui ne reçoit rien ou moins : les plongées sans photo et les photos hors plongée (**Tout le catalogue**), les photos qui n'auront pas de profondeur (**Les photos sélectionnées dans Lightroom**), et les unités utilisées.

La barre du bas résume ce qui sera écrit (« N photos · N collections ») et rappelle qu'un seul « Annuler » défait tout. Cliquez sur **Appliquer à N photos**. Dans le chemin **Tout le catalogue**, N compte toutes les photos trouvées dans les plongées cochées. Dans le chemin **Les photos sélectionnées dans Lightroom**, ce sont les photos sélectionnées dans Lightroom **au moment du clic** qui sont écrites.

**Appliquer** reste grisé, avec la raison dans la barre, tant que rien ne peut être écrit, par exemple :

- « Cochez au moins une option » ;
- « Choisissez un site », tant qu'aucun site n'est choisi sous **Autre site** ;
- « Sélectionnez des photos dans Lightroom », quand la sélection est vide ;
- « Rien à écrire avec ces options. » ;
- l'ensemble de collections choisi dans les Réglages n'est plus dans le catalogue (seulement avec **Créer les collections** cochée) : **Ouvrir les réglages** permet d'en choisir un autre ;
- Lightroom n'est pas connecté.

### Étape 5 — Récapitulatif

**Écrit dans Lightroom** : **Photos taguées** (les photos écrites, y compris celles seulement rangées dans une collection), **Avec profondeur** et **Collections**, puis chaque collection avec son nombre de photos, **créée** ou **complétée**. Ensuite les avertissements (plongées sans photo, photos hors plongée, photos sans profondeur) et le rappel pour tout défaire (section 8).

- **Tout le catalogue** : **Retour au choix du périmètre** ramène à l'étape 1 et commence un nouveau passage : **Tout le catalogue** reste choisi, aucune plongée n'est cochée et les options reviennent à leur valeur par défaut. Quand une plongée n'a reçu aucune photo, **Traiter la plongée N à part** (ou **Traiter ces plongées à part**) mène directement à l'étape 3 avec ces seules plongées cochées, et relance la recherche de leurs photos : donnez à leur appareil son propre décalage (section 11).
- **Les photos sélectionnées dans Lightroom** : **Terminer** ramène à l'étape 1 et commence de même un nouveau passage, la carte des photos sélectionnées restant choisie.

---

## 4. Quelles options pour quel besoin

Les options de l'étape 4 décident de ce que Dive Lens écrit sur chaque photo. Cette section montre ce qu'écrit chaque combinaison, chemin par chemin, puis quelles options choisir dans des situations que les plongeurs rencontrent souvent. Les règles détaillées sont à la [section 5](#5-ce-qui-est-écrit).

### Comment les options se combinent

- **Détails de la plongée**, **Détails du site** et **Créer les collections** sont indépendantes : ce que l'une écrit ne dépend jamais des autres, et les lignes des tableaux ci-dessous se combinent. Par exemple, **Détails de la plongée** et **Créer les collections** cochées sans **Détails du site** écrivent les infos de plongée et les collections, et laissent le lieu et le GPS tels quels.
- **Profondeur dans l'altitude GPS** ne fonctionne qu'avec **Détails de la plongée** : décocher **Détails de la plongée** la grise.
- Sous **Détails du site**, **Lieu** et **GPS** sont eux aussi indépendants ; les décocher tous les deux revient à décocher **Détails du site**. Tous deux viennent du site de la plongée, ou de l'**Autre site**.
- **Autre site** ne change que le lieu et le GPS écrits sur les photos : chaque collection garde le nom du site de sa plongée.
- Même avec **Détails de la plongée** décochée, les photos sont rattachées à leur plongée d'après leur heure (**Tout le catalogue**), pour le site et les collections.
- **Aucune plongée — lieu seulement** (chemin **Les photos sélectionnées dans Lightroom**) ne laisse que **Détails du site** : **Détails de la plongée** et **Créer les collections** sont grisées, et le site est toujours **Autre site…**.

Les photos écrites :

- **Tout le catalogue** : les photos rangées dans une plongée cochée, prises pendant la plongée ou dans sa marge de surface. Les appareils ignorés et les photos hors plongée ne sont pas touchés.
- **Les photos sélectionnées dans Lightroom** : toutes les photos sélectionnées au moment où vous cliquez sur **Appliquer**, quelle que soit leur heure.

### Ce que reçoit chaque photo

Dans les tableaux, « Gardé » veut dire que Dive Lens ne touche pas au champ : la photo garde ce qu'elle avait, d'un passage précédent, de l'appareil ou de vous. « Aucune » sous **Collection** veut dire qu'aucune collection n'est créée ni modifiée : la photo reste dans les collections où elle était déjà. Un site sans pays, ville, lieu ou GPS laisse cette valeur sur la photo telle quelle. Les champs de plongée écrits décrivent une seule plongée : une valeur que la plongée n'a pas, comme une température ou une note, est effacée.

**Tout le catalogue**

| Coché à l'étape 4 | Champs de plongée | Profondeur, température, temps | Décalage d'horloge | Altitude GPS | Pays, ville, lieu | GPS | Collection |
|---|---|---|---|---|---|---|---|
| Tout (par défaut) | Écrits | Pendant la plongée¹ | Écrit | Gardée² | Site de chaque plongée | Site de chaque plongée | Celle de sa plongée |
| Tout, et **Profondeur dans l'altitude GPS** | Écrits | Pendant la plongée¹ | Écrit | La profondeur³ | Site de chaque plongée | Site de chaque plongée | Celle de sa plongée |
| **Détails de la plongée** seule | Écrits | Pendant la plongée¹ | Écrit | Gardée² | Gardés | Gardé | Aucune |
| **Détails de la plongée** et **Profondeur dans l'altitude GPS** seules | Écrits | Pendant la plongée¹ | Écrit | La profondeur³ | Gardés | Gardé | Aucune |
| **Détails du site** seule (**Lieu** et **GPS**) | Gardés | Gardés | Gardé | Gardée | Site de chaque plongée | Site de chaque plongée | Aucune |
| **Lieu** seul | Gardés | Gardés | Gardé | Gardée | Site de chaque plongée | Gardé | Aucune |
| **GPS** seul | Gardés | Gardés | Gardé | Gardée | Gardés | Site de chaque plongée | Aucune |
| Tout, avec **Autre site, le même pour toutes les photos…** | Écrits | Pendant la plongée¹ | Écrit | Gardée² | L'autre site | L'autre site⁴ | Celle de sa plongée⁵ |
| **Créer les collections** seule | Gardés | Gardés | Gardé | Gardée | Gardés | Gardé | Celle de sa plongée |

**Les photos sélectionnées dans Lightroom**

| Coché à l'étape 4 | Champs de plongée | Profondeur, température, temps | Décalage d'horloge | Altitude GPS | Pays, ville, lieu | GPS | Collection |
|---|---|---|---|---|---|---|---|
| Tout (par défaut) | Écrits | Pendant la plongée¹ | Pendant la plongée¹ | Gardée² | Site de la plongée | Site de la plongée | Celle de la plongée |
| Tout, et **Profondeur dans l'altitude GPS** | Écrits | Pendant la plongée¹ | Pendant la plongée¹ | La profondeur³ | Site de la plongée | Site de la plongée | Celle de la plongée |
| **Détails de la plongée** seule | Écrits | Pendant la plongée¹ | Pendant la plongée¹ | Gardée² | Gardés | Gardé | Aucune |
| **Détails de la plongée** et **Profondeur dans l'altitude GPS** seules | Écrits | Pendant la plongée¹ | Pendant la plongée¹ | La profondeur³ | Gardés | Gardé | Aucune |
| **Détails du site** seule (**Lieu** et **GPS**) | Gardés | Gardés | Gardé | Gardée | Site de la plongée | Site de la plongée | Aucune |
| **Lieu** seul | Gardés | Gardés | Gardé | Gardée | Site de la plongée | Gardé | Aucune |
| **GPS** seul | Gardés | Gardés | Gardé | Gardée | Gardés | Site de la plongée | Aucune |
| Tout, avec **Autre site…** | Écrits | Pendant la plongée¹ | Pendant la plongée¹ | Gardée² | L'autre site | L'autre site⁴ | Celle de la plongée⁵ |
| **Créer les collections** seule | Gardés | Gardés | Gardé | Gardée | Gardés | Gardé | Celle de la plongée |
| **Aucune plongée — lieu seulement**, avec **Lieu** et **GPS** | Gardés | Gardés | Gardé | Gardée | L'autre site | L'autre site⁴ | Aucune |
| **Aucune plongée — lieu seulement**, avec **Lieu** seul | Gardés | Gardés | Gardé | Gardée | L'autre site | Gardé | Aucune |
| **Aucune plongée — lieu seulement**, avec **GPS** seul | Gardés | Gardés | Gardé | Gardée | Gardés | L'autre site⁴ | Aucune |

1. Sur les photos prises pendant une plongée qui a une courbe : la profondeur, la température et le temps écoulé au moment de la photo.
   - **Tout le catalogue** : les autres photos rangées dans une plongée (dans la marge de surface, ou dans une plongée sans courbe) voient ces trois valeurs effacées. Toutes les photos rangées dans une plongée reçoivent le décalage d'horloge.
   - **Les photos sélectionnées dans Lightroom** : le décalage d'horloge ne va que sur les photos prises pendant la plongée, avec leur profondeur. Les autres photos gardent leurs valeurs quand elles portent déjà la même plongée ; quand elles portaient une autre plongée, sa profondeur, sa température, son temps écoulé et son décalage d'horloge sont effacés.
2. Gardée, sauf une altitude que Dive Lens a copiée lui-même : elle est effacée quand la photo est rattachée à une autre plongée sans nouvelle altitude et, dans le chemin **Tout le catalogue**, quand la photo est retrouvée sans profondeur ([section 6](#6-la-profondeur-dans-laltitude-gps)).
3. Sur les photos qui reçoivent une profondeur : cette profondeur, en altitude GPS négative, en mètres. Les autres photos comme à la note 2.
4. Le GPS du site choisi, ou la position que vous avez saisie dessous. Vider les deux coordonnées écrit la position du site lui-même : pour garder le GPS des photos, décochez **GPS**.
5. La collection garde le nom du site de sa plongée.

### Quand « Appliquer » reste grisé

| Situation | Ce que dit la barre |
|---|---|
| **Détails de la plongée**, **Détails du site** et **Créer les collections** toutes décochées ; avec **Aucune plongée — lieu seulement**, **Lieu** et **GPS** tous deux décochés | « Cochez au moins une option » |
| **Autre site** choisi, ou imposé (**Aucune plongée — lieu seulement**, ou une plongée choisie qui n'a pas de site), et aucun site encore choisi | « Choisissez un site » |
| La position saisie sous **Autre site** n'est pas valide, alors que **GPS** est coché | « Latitude entre -90 et 90, longitude entre -180 et 180, ou les deux vides. » |
| Les options cochées ne laissent rien à écrire sur aucune photo : par exemple **GPS** seul sur un site sans GPS, ou **Lieu** seul sur un site sans nom, ni pays, ni ville | « Rien à écrire avec ces options. » |
| **Tout le catalogue** : aucune photo ne tombe dans les plongées cochées | « Aucune photo ne tombe dans les plongées : ajustez les décalages d'horloge. » |
| **Les photos sélectionnées dans Lightroom** : aucune photo sélectionnée | « Sélectionnez des photos dans Lightroom » |
| **Créer les collections** cochée, et l'ensemble de collections choisi dans les réglages n'est plus dans le catalogue | « L'ensemble choisi dans les réglages (<emplacement>) n'est plus dans le catalogue. Choisissez un autre emplacement dans les réglages. », avec **Ouvrir les réglages** |

Dans le chemin **Tout le catalogue**, **GPS** seul (ou **Lieu** seul) n'est pas refusé quand seuls certains sites des plongées le donnent : seules les photos de ces plongées le reçoivent. L'effet affiché à droite de **Détails du site** compte les photos qui reçoivent vraiment quelque chose, tandis que **Appliquer à N photos** compte toujours toutes les photos trouvées.

### Cas d'usage

**Taguer tout un voyage, la première fois.** **Tout le catalogue** ; cochez les plongées du voyage ; à l'étape 3, **Suggérer pour tous**, puis vérifiez la carte de chaque appareil et l'aperçu par plongée ; laissez les options telles quelles ; **Appliquer**. Chaque photo prise pendant une plongée reçoit les infos de plongée, le lieu et le GPS de son site, le décalage d'horloge et, quand la plongée a une courbe, sa profondeur, sa température et son temps écoulé. Chaque plongée qui a des photos reçoit sa collection « numéro - site ». Les photos hors plongée ne reçoivent rien : voir le bateau et la plage, plus bas.

**Ajouter les collections plus tard, sans toucher aux infos des photos.** **Tout le catalogue**, les mêmes plongées ; à l'étape 3, les cartes partent des décalages mémorisés au premier passage ; à l'étape 4, décochez **Détails de la plongée** et **Détails du site** et gardez **Créer les collections**. La barre indique « N collections · rien n'est écrit sur les photos ». Les photos de chaque plongée vont dans sa collection ; une photo qui y est déjà n'est pas ajoutée deux fois. L'inverse marche aussi : décochez seulement **Créer les collections** pour écrire les infos sans aucune collection.

**Des photos prises sur le bateau ou à la plage entre deux plongées.** Dans le chemin **Tout le catalogue**, elles tombent hors plongée et ne reçoivent rien. Sélectionnez-les dans Lightroom, puis choisissez la carte des photos sélectionnées et **Aucune plongée — lieu seulement** ; à l'étape 4, choisissez le site par pays › lieu › site et, si la plage n'est pas à l'endroit du site, saisissez sa position dessous ; **Appliquer**. Elles ne reçoivent que le pays, la ville, le lieu et le GPS : aucun champ de plongée et aucune collection, et les infos de plongée que certaines portent peut-être déjà restent telles quelles. Pour leur donner plutôt les infos d'une plongée, choisissez cette plongée : elles reçoivent ses infos, son site et sa collection, sans profondeur ([section 7](#7-le-lieu-seul-et-les-photos-hors-plongée)).

**Une plongée sans courbe.** Une plongée saisie à la main, ou dont l'application de plongée n'a pas gardé le téléchargement de l'ordinateur, ne demande rien de particulier. Ses photos reçoivent les infos de plongée, le site et la collection, sans profondeur, ni température, ni temps : **Profondeur dans l'altitude GPS** n'a donc rien à copier pour elles. Dans le chemin **Tout le catalogue**, le décalage d'horloge décide toujours quelles photos appartiennent à la plongée, et il est écrit sur elles ; l'aperçu par plongée indique « Pas de courbe : infos de plongée seules ». Dans le chemin **Les photos sélectionnées dans Lightroom**, l'étape 3 indique « Pas de courbe : pas de données par photo » et aucun décalage d'horloge n'est écrit.

**Corriger le site de photos déjà taguées.** Sélectionnez les photos dans Lightroom, choisissez la carte des photos sélectionnées et leur plongée ; à l'étape 4, décochez **Détails de la plongée** et **Créer les collections**, choisissez **Autre site…** et le bon site ; **Appliquer**. Seuls le pays, la ville, le lieu et le GPS changent : les infos de plongée et les collections restent. Une valeur que le nouveau site n'a pas n'est pas effacée : si l'ancien site avait une ville et le nouveau n'en a pas, l'ancienne ville reste ; effacez-la dans Lightroom. Pour les photos de plusieurs plongées d'un coup, faites de même dans le chemin **Tout le catalogue** avec **Autre site, le même pour toutes les photos…**. Si le site est faux dans votre carnet, corrigez-le plutôt là, puis relancez la plongée : les photos reçoivent le nouveau site et, avec **Créer les collections** cochée, une collection au nom du nouveau site est créée à côté de l'ancienne.

**Garder une position GPS venue de l'appareil ou saisie à la main.** Décochez **GPS** sous **Détails du site** : le pays, la ville et le lieu sont écrits, la position reste. Même avec **GPS** coché, un site sans position laisse celle de la photo. Si une position a été remplacée par erreur, **Édition › Annuler Dive Lens** dans Lightroom la remet.

**La profondeur en altitude GPS, pour les cartes et les autres logiciels.** Cochez **Profondeur dans l'altitude GPS**, sous **Détails de la plongée** : chaque photo qui reçoit une profondeur la reçoit aussi en altitude GPS négative. L'option est décochée à chaque passage, pour que vous décidiez chaque fois, en sachant quatre choses : le champ **Profondeur** de Dive Lens reste dans Lightroom et disparaît à l'export ; l'altitude GPS suit les photos exportées et se voit dans les autres logiciels ; elle est toujours en mètres, quelles que soient vos unités ; et elle remplace une altitude déjà présente sur la photo (**Édition › Annuler** la remet). Le détail est à la [section 6](#6-la-profondeur-dans-laltitude-gps).

**Relancer Dive Lens sur des photos déjà taguées.** **Tout le catalogue** : relancer des plongées ne fait qu'ajouter. Rien n'est en double, les photos retrouvées voient leurs valeurs réécrites avec les décalages affichés, et une photo qui tombe maintenant dans une autre plongée reçoit les infos et la collection de celle-ci, mais reste aussi dans son ancienne collection : retirez-l'en à la main. **Les photos sélectionnées dans Lightroom** : avec la plongée que les photos portent déjà, les infos de plongée sont réécrites, de même que la profondeur, la température, le temps écoulé et le décalage d'horloge des photos prises pendant la plongée ; les autres photos gardent les leurs. Avec une autre plongée, ses infos remplacent celles de l'ancienne, et les photos restent dans la collection de l'ancienne plongée.

**Des photos de deux appareils aux horloges différentes.** **Tout le catalogue** les traite en un seul passage : chaque appareil a sa carte à l'étape 3 et son propre décalage ; **Suggérer pour tous** les règle tous d'un coup, sans toucher à une carte réglée à la main ; deux boîtiers du même modèle sont distingués par leur numéro de série. **Ignorer** écarte un appareil dont les photos n'ont rien à voir avec les plongées, comme un téléphone resté sur le bateau. Après **Appliquer**, le décalage de chaque appareil est mémorisé pour la fois suivante (**Réglages › Décalages d'horloge mémorisés**). Une horloge changée en plein voyage est un autre cas : traitez séparément les jours d'avant et d'après le changement, chacun avec son décalage ([section 11](#11-dépannage)).

**Des photos hors de l'horaire de la plongée, dans le chemin des photos sélectionnées.** Elles sont quand même rattachées à la plongée choisie : elles reçoivent ses infos, son site et sa collection, mais ni profondeur, ni température, ni temps, ni décalage d'horloge. L'étape 3 les liste sous **Hors de l'horaire de la plongée**, l'étape 4 sous **À savoir avant d'appliquer**, et le récapitulatif les compte parmi les photos sans profondeur. Elles n'ont rien à faire dans cette plongée ? Retirez-les de la sélection dans Lightroom avant **Appliquer**. Dans le chemin **Tout le catalogue**, **Inclure les photos prises en surface** fait de même pour les photos prises juste avant ou juste après une plongée : elles reçoivent les infos de plongée, le site, le décalage d'horloge et la collection, sans profondeur.

**Annuler.** Quelles que soient les options, un **Appliquer** est une seule étape dans Lightroom : **Édition › Annuler Dive Lens** défait tout ce qu'il a écrit, GPS et altitude compris ([section 8](#8-annuler)).

---

## 5. Ce qui est écrit

- **Créer les collections** : une collection par plongée, nommée **« numéro - site »**, dans l'ensemble de collections **Dive Lens** (l'emplacement se change dans les réglages). Une plongée sans numéro reçoit sa date et son heure de début à la place du numéro.
- **Détails de la plongée** : sur chaque photo, numéro de plongée, début et fin, durée, profondeur max, températures min et max, note et l'application de plongée d'où viennent les données. Sur les photos prises pendant une plongée qui a une courbe, aussi **la profondeur, la température et le temps écoulé** au moment de la photo, estimés entre les deux points de la courbe qui l'entourent. Et le décalage d'horloge appliqué : sur toutes les photos rangées dans une plongée (**Tout le catalogue**), sur les photos prises pendant la plongée, quand elle a une courbe (**Les photos sélectionnées dans Lightroom**).
- **Détails du site** : pays, ville, lieu et GPS, dans les champs standard de Lightroom.

Bon à savoir, dans les deux chemins :

- L'heure d'une photo est son **heure de prise de vue d'origine** : un TIFF revenu de Photoshop est rattaché comme son original.
- Un site sans pays, ville ou GPS ne touche pas aux valeurs déjà présentes sur la photo (une position GPS venue d'un téléphone est conservée).
- Avec **Détails de la plongée** décochée, Dive Lens ne touche pas du tout aux champs de plongée ; les photos sont quand même rattachées aux plongées pour le site et les collections.
- Une valeur absente de la plongée (température, note…) efface l'ancienne valeur Dive Lens de la photo : les champs décrivent toujours une seule plongée.
- Les unités choisies dans les réglages valent pour les valeurs écrites. Les photos déjà taguées gardent l'unité en vigueur au moment où elles ont été écrites.

Dans le chemin **Tout le catalogue** :

- Seules les photos rangées dans une plongée sont écrites, dans la plongée ou dans la marge de surface. Les appareils ignorés et les photos hors plongée ne sont pas touchés.
- Une photo dans la marge de surface reçoit les infos de plongée, le lieu et le décalage d'horloge, sans données par photo.
- Une photo qui tomberait dans deux plongées va à la plus proche.
- Relancer un groupe de plongées ne fait qu'ajouter : les photos déjà rangées restent où elles sont (corrigez-les à la main si besoin), et les photos retrouvées voient leurs valeurs réécrites.
- Une plongée dont la courbe ne peut pas être lue arrête **Appliquer** quand des photos ont été prises pendant celle-ci : rien n'est écrit.

Dans le chemin **Les photos sélectionnées dans Lightroom** :

- Toutes les photos sélectionnées au moment du clic sont écrites.
- Une photo prise hors de l'horaire de la plongée reçoit les infos de plongée et le lieu, sans données par photo ni décalage d'horloge.
- Avec **Détails de la plongée**, une photo qui porte déjà une **autre** plongée perd les valeurs propres à celle-ci : profondeur, température, temps écoulé et décalage d'horloge, remplacés par les nouveaux quand la photo a été prise pendant la nouvelle plongée. Son altitude GPS suit la section 6. La photo reste dans la collection de son ancienne plongée : à vous de la déplacer si besoin.
- Taguer une photo avec la plongée qu'elle porte déjà réécrit ses données par photo si elle a été prise pendant la plongée, avec le décalage affiché ; les autres photos gardent les leurs.
- Si la sélection change dans Lightroom au moment où vous cliquez sur **Appliquer** alors que des données par photo sont écrites, rien n'est écrit : « La sélection a changé dans Lightroom. Rien n'a été écrit : vérifiez les données par photo, puis appliquez à nouveau. »
- Une courbe illisible ne bloque jamais : les infos de plongée sont écrites sans données par photo.

Après **Appliquer**, Dive Lens retient le décalage de chaque appareil pour la fois suivante, dans les deux chemins : dans le chemin **Tout le catalogue**, les appareils qui ont une photo rangée dans une plongée ; dans le chemin **Les photos sélectionnées dans Lightroom**, les appareils qui ont une photo prise pendant la plongée, quand des données par photo sont écrites.

---

## 6. La profondeur dans l'altitude GPS

L'option **Profondeur dans l'altitude GPS** se trouve à l'étape 4, sous **Détails de la plongée**. Elle est décochée à chaque passage : cochez-la chaque fois que vous la voulez. Quatre choses à savoir :

- Le champ **Profondeur** de Dive Lens n'existe que dans Lightroom : il n'est pas écrit dans le fichier photo, et disparaît à l'export. Dive Lens le remplit toujours.
- L'altitude GPS suit la photo exportée et se voit dans les autres logiciels. Avec l'option cochée, Dive Lens y écrit aussi la profondeur de chaque photo, comme une altitude négative.
- Dive Lens écrit toujours l'altitude GPS en mètres, la norme de ce champ, quelles que soient les unités choisies dans les réglages. Avec des profondeurs en pieds, la même photo montre donc 60 ft dans le champ **Profondeur** de Dive Lens, et son altitude GPS est en mètres (18,3 m sous le niveau de la mer) : c'est normal.
- La copie remplace une altitude GPS déjà présente sur la photo ; **Édition › Annuler** dans Lightroom la remet. Les photos sans profondeur gardent la leur.

Et les règles qui en découlent :

- Seules les photos qui reçoivent une profondeur reçoivent une altitude : pas celles de la marge de surface, ni celles prises hors de l'horaire de la plongée, ni celles d'une plongée sans courbe.
- Laisser l'option décochée à un passage suivant n'efface pas les altitudes déjà copiées, à deux exceptions près :
  - dans le chemin **Tout le catalogue**, une photo retrouvée sans profondeur (désormais dans la marge de surface, ou dans une plongée sans courbe) perd l'altitude que Dive Lens y avait copiée ;
  - une photo rattachée à une autre plongée sans nouvelle altitude perd celle que Dive Lens avait tirée de son ancienne profondeur.
- Dive Lens n'efface qu'une altitude qu'il a écrite lui-même : une altitude venue de l'appareil, ou que vous avez saisie ou modifiée vous-même, est conservée.
- Avec **Détails de la plongée** décochée, aucune altitude n'est écrite ni effacée.

---

## 7. Le lieu seul, et les photos hors plongée

Dans le chemin **Tout le catalogue**, les photos prises hors de toute plongée (sur le bateau, à la plage, en snorkeling) ne reçoivent rien : l'étape 3 les montre sous **Hors plongée**, de même que le récapitulatif et **À savoir avant d'appliquer**.

Pour leur poser un lieu, ou les rattacher quand même à une plongée :

1. Sélectionnez-les dans Lightroom, puis **Bibliothèque › Modules externes › Ouvrir Dive Lens…** et choisissez la carte des photos sélectionnées.
2. À l'étape 2 :
   - pour les rattacher quand même à une plongée, choisissez la plongée : elles reçoivent ses infos et son lieu, sans profondeur ;
   - pour ne leur poser qu'un lieu, choisissez **Aucune plongée — lieu seulement**.

Avec **Aucune plongée — lieu seulement**, l'étape 3 est sautée. À l'étape 4, seul **Détails du site** agit : **Détails de la plongée** et **Créer les collections** sont grisées, et le site est **Autre site…**. Choisissez le site par pays › lieu › site, ajustez son GPS si besoin, puis **Appliquer à N photos**. Seuls le pays, la ville, le lieu et le GPS sont écrits : aucun champ de plongée et aucune collection.

---

## 8. Annuler

**Édition › Annuler** dans Lightroom défait tout ce qu'un **Appliquer** a écrit, en une seule fois, GPS et altitude compris, dans les deux chemins. Le menu Édition de Lightroom l'appelle **Annuler Dive Lens**.

- Une collection créée par **Appliquer** disparaît avec lui ; les photos ajoutées à une collection existante en ressortent.
- **La première fois**, quand l'ensemble de collections (**Dive Lens** par défaut) n'existe pas encore, Dive Lens le crée juste avant d'écrire. S'il reste là, vide, après « Annuler », un second **Annuler Dive Lens** le retire, ou supprimez-le dans Lightroom. Si l'écriture est refusée après la création de l'ensemble, l'ensemble vide reste : annulez-le, ou supprimez-le dans Lightroom.
- Si Lightroom n'a pas laissé Dive Lens écrire le GPS et l'altitude, tout le reste est écrit et Dive Lens le signale : choisissez d'abord **Édition › Annuler Dive Lens** dans Lightroom, puis appliquez à nouveau.

---

## 9. Réglages

La roue dentée, en haut, ouvre les Réglages, de même que **Aide › Modules externes › Réglages de Dive Lens…** dans Lightroom. Ils occupent toute la fenêtre. **Retour** ramène au parcours, à l'étape où vous étiez, avec vos choix ; si le carnet de plongée a changé entre-temps, le parcours repart à l'étape 1. La roue dentée est grisée pendant que Dive Lens écrit dans Lightroom.

Chaque modification est enregistrée immédiatement.

**Carnet de plongée**
- **Application** : MacDive ou Subsurface. Seules les applications disponibles sur votre système sont proposées : sur Windows, Subsurface. L'application se change seulement ici, jamais depuis le parcours.
- **Base MacDive** / **Carnet Subsurface** : l'emplacement standard est utilisé, sauf si vous en choisissez un autre. Un carnet Subsurface est soit un fichier (.xml ou .ssrf), soit le dossier de son stockage git (le stockage en ligne en garde une copie sur votre ordinateur). Sous l'emplacement, Dive Lens indique si le carnet est valide et combien il contient de plongées et de sites.
- Chaque application garde son emplacement : passer de l'une à l'autre puis revenir retrouve le vôtre.
- **Un seul carnet à la fois.** Changer d'application ou de carnet fait repartir le parcours à l'étape 1, sans rien de choisi : aucune plongée cochée ni choisie, horloges et options oubliées. C'est vrai quelle que soit la façon dont le carnet change : ici, par un import des réglages, depuis l'écran affiché quand le carnet ne peut pas être lu, ou quand Dive Lens reprend les réglages de l'ancien module en se connectant à Lightroom (section 1).
- Pendant que Dive Lens écrit dans Lightroom, le carnet ne peut pas changer : **Application**, **Choisir…** (ou **Choisir un fichier…** / **Choisir un dossier…**), **Emplacement standard**, **Importer…** et **Remplacer** sont grisés, avec la mention « Écriture dans Lightroom en cours : le carnet de plongée pourra être changé une fois celle-ci terminée. »

**Affichage**
- **Langue** : par défaut, celle du système. Les menus et les champs dans Lightroom suivent la langue de Lightroom.
- **Profondeur** et **Température** : **Comme MacDive** / **Comme Subsurface** (les unités choisies dans cette application), ou forcées en mètres / pieds et °C / °F. Elles valent pour l'application et pour les valeurs écrites sur les photos. L'altitude GPS est toujours en mètres (section 6).

**Collections**
- **Placer dans** : la racine du catalogue ou un ensemble de collections existant (Lightroom doit être connecté pour les parcourir). C'est là que l'option **Créer les collections** de l'étape 4 range les collections.
- **Regrouper dans l'ensemble** : « Dive Lens » par défaut, créé si besoin ; laissez vide pour ranger les collections directement à l'emplacement choisi. Un aperçu montre le chemin obtenu. Les collections déjà créées restent où elles sont.
- Si l'ensemble choisi dans **Placer dans** n'est plus dans le catalogue, l'étape 4 le signale tant que **Créer les collections** est cochée, et **Appliquer** reste grisé jusqu'à ce que vous choisissiez un autre emplacement ici.

**Décalages d'horloge mémorisés**
- Sous le titre, « Gardés pour chaque appareil après « Appliquer » et proposés à nouveau à l'étape Horloges, pour <application>. » La liste montre les appareils dont le décalage est mémorisé pour l'application de plongée choisie plus haut, classés par nom comme à l'étape 3. Chaque ligne donne le nom de l'appareil comme sur sa carte de l'étape 3 (son modèle, ou « Appareil inconnu »), avec dessous « n° <numéro de série> » quand les photos le donnent ; à droite, son décalage comme sur la carte (par exemple « +4:00 », « 0:00 » ou « −0:30 »), et **Oublier**. Sans aucun : « Aucun décalage d'horloge mémorisé. »
- **Oublier** retire aussitôt le décalage de cet appareil, sans demander de confirmation, et sa ligne disparaît. La fois suivante, l'étape 3 part de 0:00 sur sa carte, sans **décalage mémorisé** ; **Suggérer** retrouve le décalage. Les autres appareils, et les décalages gardés pour l'autre application de plongée, restent.
- Rien ne change dans Lightroom : le décalage d'horloge déjà écrit sur les photos reste.
- Oublié pendant un parcours, dans l'un ou l'autre chemin : au retour des Réglages, l'appareil n'est plus présenté comme mémorisé. Si sa carte de l'étape 3 portait encore l'étiquette **décalage mémorisé**, elle repart de 0:00, comme une nouvelle recherche la montrerait ; un décalage réglé à la main ou trouvé par **Suggérer** reste tel quel. L'étiquette disparaît, de même que « avec le décalage mémorisé de… » sous **Couvre la sélection** à l'étape 2. Les compteurs et **Couvre la sélection** suivent le décalage que la carte montre désormais, et **Appliquer** n'écrit et ne mémorise que ce que montrent les cartes.

**Sauvegarde des réglages**
- **Exporter ou importer** : pour retrouver vos réglages sur un autre ordinateur, ou après une réinstallation.
- **Exporter…** les enregistre dans un fichier, nommé « Réglages Dive Lens.json » sauf si vous choisissez un autre nom. Le fichier contient : l'application de plongée et l'emplacement du carnet de chacune, la langue, les unités, l'emplacement des collections, la vérification automatique des mises à jour, le niveau du journal et l'ordre de la liste des plongées. Les décalages d'horloge retenus pour chaque appareil n'y figurent pas : ils restent sur cet ordinateur.
- **Importer…** demande confirmation avant de rien changer : **Remplacer** met les réglages du fichier à la place de tous ceux de cet écran, **Annuler** les laisse tels quels.
- Si le carnet du fichier n'est pas sur cet ordinateur, Dive Lens le signale et garde son emplacement jusqu'à ce que vous indiquiez le bon (**Choisir…**, ou **Choisir un fichier…** / **Choisir un dossier…** pour Subsurface). Si l'ensemble de collections choisi dans **Placer dans** n'est pas dans ce catalogue, l'étape 4 le signale tant que **Créer les collections** est cochée : choisissez un autre emplacement dans les réglages.
- Un fichier venu d'un Mac réglé sur MacDive, importé sur Windows : Dive Lens utilise Subsurface, l'application disponible sur Windows, et garde l'emplacement MacDive : ces réglages, exportés à nouveau puis importés sur un Mac, y utilisent toujours MacDive.
- Un fichier qui ne ressemble pas à un fichier de réglages Dive Lens est refusé, et rien ne change. Un fichier exporté par une version antérieure de Dive Lens reste importable ; son réglage « Copier la profondeur dans l'altitude GPS » est ignoré, l'option étant désormais à l'étape 4.
- Après un import, Dive Lens ne reprend plus les réglages de l'ancien module (section 1) : ce sont ceux du fichier qui sont gardés.

**Mises à jour**
- **Vérifier automatiquement** (activé par défaut) : au lancement de Dive Lens, au plus une fois par jour, Dive Lens regarde sur GitHub s'il existe une nouvelle version. Rien ne s'installe sans votre accord.
- **Vérifier maintenant** le fait tout de suite. Sous **Dive Lens <version>**, la ligne dit si votre version est à jour et quand la dernière vérification a eu lieu.
- Quand une nouvelle version existe, un bandeau le signale en haut de l'écran : **Voir** ouvre cette section, **×** ferme le bandeau jusqu'à la version suivante. Ici, **Nouveautés** ouvre la page de la version sur GitHub, et **Installer** l'installe.
- **Installer** télécharge la nouvelle version, vérifie qu'elle vient bien de Dive Lens (signature), remplace l'application et la partie Lightroom, puis Dive Lens redémarre. Pas pendant une écriture dans Lightroom.
- Ensuite, **redémarrez Lightroom** : il ne charge la nouvelle partie Dive Lens qu'à son démarrage. Jusque-là, Dive Lens affiche « Redémarrez Lightroom pour terminer la mise à jour de Dive Lens. », et la barre d'état **Redémarrez Lightroom**.
- Sur Windows, une copie installée avec le zip apparaît ensuite dans **Paramètres › Applications › Applications installées**, à l'endroit où vous l'aviez placée.
- Si Dive Lens ne peut pas modifier son dossier, il propose **Télécharger depuis GitHub** : installez alors la nouvelle version vous-même (section 1).

**Diagnostic**
- **Niveau du journal** : Normal ou Détaillé. À augmenter seulement pour signaler un problème.
- **Afficher le journal** : ouvre le dossier des deux journaux, celui de l'application (`DiveLens App.log`) et celui du module Lightroom (`DiveLens.log`).

---

## 10. Les champs Dive Lens dans Lightroom

Dans le module **Bibliothèque**, ouvrez le panneau **Métadonnées** et choisissez **Dive Lens** dans son menu. Les champs sont regroupés en quatre parties :

- **Photo** : Profondeur, Température, Temps écoulé ;
- **Plongée** : Plongée, Début de plongée, Fin de plongée, Durée, Profondeur max, Temp. min, Temp. max, Note ;
- **Site** : le lieu, la ville, le pays et le GPS de Lightroom, avec l'altitude GPS sous le GPS ;
- **Technique** : Décalage horloge, Source.

Les champs Dive Lens restent dans le catalogue Lightroom : ils ne sont pas écrits dans les fichiers photo et ne suivent pas les exports. Seuls la position GPS, les champs de lieu et, si vous le choisissez, l'altitude les suivent.

---

## 11. Dépannage

**Une plongée n'a aucune photo.** Vérifiez l'horloge de l'appareil à l'étape 3 : utilisez **Suggérer**, ou réglez l'écart à la main. Si l'horloge d'un appareil ou de l'ordinateur de plongée a été changée pendant le voyage, traitez les jours d'avant et d'après le changement en deux groupes séparés, chacun avec son décalage : **Traiter la plongée N à part**, sur le récapitulatif, le fait pour les plongées restées sans photo.

**« Base MacDive introuvable ».** La base a peut-être été déplacée, par exemple dans iCloud : utilisez **Localiser la base…** ou **Réglages › Choisir…**. Si elle est dans iCloud, vérifiez qu'elle est bien téléchargée sur ce Mac.

**« Le carnet de plongée a changé… ».** Le carnet en service a changé (dans les réglages, par exemple) pendant que Dive Lens travaillait sur le précédent : l'action a donc été refusée. Après **Appliquer**, le bandeau orange indique « Le carnet de plongée a changé. Rien n'a été écrit : recommencez. » Après une action qui n'écrit rien (recherche des photos, affichage d'une courbe), il indique « Le carnet de plongée a changé : recommencez. » Il s'affiche à l'étape 1, sous les étapes, jusqu'à ce que vous cliquiez sur **Fermer** ou **Suivant**. Choisissez à nouveau les plongées : elles viennent maintenant du nouveau carnet.

**Dive Lens est désactivé après l'installation.** Lightroom peut présenter Dive Lens désactivé, notamment quand un module Dive Lens précédent avait été désactivé avant d'être retiré. Dans **Fichier › Gestionnaire de modules externes…**, choisissez Dive Lens, cliquez sur **Activer**, puis **Terminé**. Une seule fois suffit.

**« Lightroom déconnecté ».** Ouvrez Lightroom Classic, vérifiez dans **Fichier › Gestionnaire de modules externes** que Dive Lens est activé, puis choisissez à nouveau **Bibliothèque › Modules externes › Ouvrir Dive Lens…**.

**L'application ne s'ouvre pas (Mac).** Si macOS indique qu'il ne peut pas vérifier l'application, téléchargez à nouveau Dive Lens depuis sa page de publication : les versions publiées sont signées et vérifiées par Apple, et s'ouvrent sans ce message.

**Passer de MacDive à Subsurface.** Rien à migrer. Relancer une plongée retrouve sa collection quand le nom « numéro - site » est identique ; sinon une nouvelle collection est créée à côté. Les décalages d'horloge sont mémorisés séparément pour chaque application.

**Signaler un problème.** Passez **Réglages › Diagnostic › Niveau du journal** sur **Détaillé**, reproduisez le problème, puis **Afficher le journal** : le dossier contient les deux journaux. [Ouvrez ensuite un ticket](https://github.com/Ventrilok/divelens/issues/new/choose) qui décrit le problème, sans y joindre les journaux : le dépôt GitHub est public, et les journaux contiennent vos sites de plongée et des chemins de fichiers. S'ils sont nécessaires, ils vous seront demandés : envoyez-les uniquement par e-mail à [divelens@querton.com](mailto:divelens@querton.com), avec le numéro du ticket.

---

## 12. Confidentialité

- Votre carnet de plongée est seulement lu, jamais modifié. MacDive ou Subsurface peuvent rester ouverts pendant ce temps.
- Dive Lens n'envoie aucune donnée de plongée nulle part. Il n'accède à Internet que pour deux choses :
  - la carte du site : le service de carte (OpenFreeMap, données OpenStreetMap) reçoit la zone de carte affichée, rien d'autre. Sans Internet, la carte est remplacée par le message « Carte indisponible sans Internet. » ;
  - les mises à jour : GitHub reçoit la demande de la dernière version, et le téléchargement si vous installez, rien d'autre. **Réglages › Mises à jour › Vérifier automatiquement** coupe la vérification au lancement.
