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
- **Aide › Modules externes › Réglages de Dive Lens…** l'ouvre sur les **Réglages** (section 8).

L'entrée Dive Lens de **Fichier › Gestionnaire de modules externes…** a aussi les boutons **Ouvrir Dive Lens** et **Ouvrir les réglages**, et les liens vers le site web de Dive Lens et vers son code source sur GitHub.

Si l'application est déjà ouverte, le menu ramène sa fenêtre au premier plan. Choisi pendant le parcours, **Ouvrir Dive Lens…** ramène à l'étape 1 en gardant vos choix ; choisi sur le récapitulatif, il commence un nouveau passage.

Lightroom doit rester ouvert pendant que vous utilisez l'application. La première fois, passez par les **Réglages** (section 8) : l'application de plongée et l'emplacement de votre carnet.

### La fenêtre

- **En haut** : Dive Lens à gauche ; à droite, la roue dentée ouvre les **Réglages** (section 8). Elle est grisée pendant que Dive Lens écrit dans Lightroom.
- **En bas**, la barre d'état. À gauche, Lightroom :
  - **Lightroom connecté**, **Lightroom déconnecté** ou **Redémarrez Lightroom** ;
  - **Catalogue :** le nom du catalogue ouvert dans Lightroom ;
  - **Taille du catalogue :** le nombre de photos qu'il contient, recompté chaque fois que vous revenez à l'étape 1 ;
  - **Photos sélectionnées :** combien sont sélectionnées dans Lightroom, en direct (**aucune** quand il n'y en a pas).
- À droite de la barre d'état, votre carnet : **Application de carnet :** MacDive ou Subsurface, et **Plongées enregistrées :** combien de plongées il contient.

Si la barre d'état affiche **Lightroom déconnecté**, rien ne peut être écrit : vérifiez que Lightroom est ouvert et que le module est activé dans le gestionnaire de modules externes. Si elle affiche **Redémarrez Lightroom**, Dive Lens a été mis à jour pendant que Lightroom était ouvert : quittez Lightroom et relancez-le.

---

## 3. Le parcours

Cinq étapes, rappelées en haut : **Photos › Plongées › Horloges › Vérifier et appliquer › Récapitulatif**. **Suivant** avance et **Précédent** revient en arrière ; un clic sur une étape déjà faite y revient aussi, sauf depuis le récapitulatif. Sous les étapes, une ligne qui commence par **Photos :** rappelle vos choix ; à l'étape 2, son **Modifier** ramène à l'étape 1. Rien n'est écrit dans Lightroom avant que vous cliquiez sur **Appliquer à N photos** à l'étape 4.

### Deux chemins

L'étape 1 pose la seule vraie question : sur quelles photos ?

- **Tout le catalogue** : vous cochez une ou plusieurs plongées, et Dive Lens cherche dans tout le catalogue les photos prises pendant chacune. Chaque photo va dans la plongée qui correspond à son heure. Pour un voyage entier ou une série de plongées. Les photos sélectionnées dans Lightroom sont ignorées.
- **Les photos sélectionnées dans Lightroom** : celles que vous avez sélectionnées dans Lightroom. Vous choisissez une plongée, et elles y sont toutes rattachées, quelle que soit leur heure. Pour des photos sans heure fiable, pour rattacher des photos à une plongée à la main, ou pour ne leur poser qu'un lieu (section 6).

Choisir l'autre chemin à l'étape 1 fait repartir de zéro les étapes suivantes : plongées, horloges et options.

### Étape 1 — Photos

« Sur quelles photos travailler ? » Deux cartes : **Tout le catalogue**, ou les photos sélectionnées dans Lightroom. La carte de la sélection suit Lightroom en direct : les appareils et le jour des photos sélectionnées, et jusqu'à six vignettes. Sans photo sélectionnée, elle est grisée et indique **Aucune photo sélectionnée** : sélectionnez des photos dans Lightroom et elle s'allume. Choisissez une carte, puis **Suivant**.

### Étape 2 — Plongées

La liste montre toutes les plongées de votre carnet, regroupées par mois. Cherchez par numéro, site, lieu ou pays, et affinez avec les filtres (**Période**, **Pays**, **Profondeur**, **Note**, **Avec courbe**). **Trier** ordonne la liste (**Plus récentes d'abord**, **Plus anciennes d'abord**, **Profondeur max**) ; Dive Lens retient votre choix. Un clic sur une plongée affiche sa fiche à droite : courbe, durée, profondeur max, températures, note, GPS du site avec une carte, et vos notes.

- **Tout le catalogue** : cochez une ou plusieurs plongées, puis **Suivant**. La barre indique combien sont cochées et leurs jours ; **Tout désélectionner** les décoche.
- **Les photos sélectionnées dans Lightroom** : cliquez sur une plongée, puis **Suivant**.
  - La plongée dont les heures couvrent la sélection porte le repère **Couvre la sélection**. Il tient compte du dernier décalage appliqué à chaque appareil, puis des décalages réglés à l'étape 3. Ce n'est qu'une aide : c'est vous qui choisissez.
  - La première ligne, **Aucune plongée — lieu seulement**, sert à ne poser qu'un lieu sur les photos (section 6).

Une plongée **sans courbe** (saisie à la main, ou venant d'un ancien ordinateur de plongée dont MacDive n'a pas gardé le téléchargement) fonctionne quand même : ses photos reçoivent les infos de plongée et le lieu, mais ni profondeur, ni température, ni temps.

### Étape 3 — Horloges

**Horloge des appareils.** L'horloge d'un appareil est rarement exactement à l'heure de l'ordinateur de plongée, et souvent encore à l'heure de la maison. Chaque appareil a sa carte (deux boîtiers du même modèle sont distingués par leur numéro de série). Réglez l'écart entre son horloge et celle du carnet :

- **Suggérer** propose le décalage qui place le plus de photos dans les plongées. Il essaie chaque quart d'heure entre −14 h et +14 h, et retrouve donc aussi un fuseau horaire oublié.
- **Suggérer pour tous** le fait pour tous les appareils d'un coup, sans toucher à ceux réglés à la main ni aux appareils ignorés.
- Vous pouvez aussi saisir le décalage (heures:minutes) ou l'ajuster vous-même.
- Chaque carte part du dernier décalage appliqué à cet appareil, dans l'un ou l'autre chemin (**décalage mémorisé**) ; sans décalage connu, elle part de 0:00. Les décalages sont mémorisés séparément pour chaque application de plongée.

**Aperçu par plongée.** Les compteurs suivent vos réglages : photos par plongée, appareils, et les données par photo que donne chaque plongée (**Profondeur · temp. · temps**, **Pas de courbe : infos de plongée seules**, **Photos de surface seulement**). Une plongée sans photo est en orange, **Aucune photo — horloge changée ?** : c'est souvent le signe d'une horloge changée pendant le voyage (section 10).

Dans le chemin **Tout le catalogue** :

- Dive Lens cherche dans tout le catalogue les photos prises sur la période des plongées cochées.
- **Ignorer** écarte un appareil, par exemple un téléphone resté sur le bateau : ses photos ne sont ni comptées ni taguées.
- **Inclure les photos prises en surface** : jusqu'à N minutes avant et après chaque plongée, pour les photos prises sur le bateau juste avant la mise à l'eau ou juste après la sortie. 0 minute par défaut, et retour à 0 à chaque passage. Ces photos reçoivent les infos de plongée, sans profondeur.
- **Hors plongée** : les photos de la période qui ne tombent dans aucune plongée, même avec la marge (à terre, sur le bateau…). Elles ne reçoivent rien (section 6).

Dans le chemin **Les photos sélectionnées dans Lightroom** :

- Les photos sont rattachées à la plongée choisie, quelle que soit leur heure. L'horloge sert seulement à leur donner une profondeur : seules celles prises pendant la plongée en reçoivent une.
- Une carte par appareil de la sélection. Pas de case **Ignorer** (retirez plutôt la photo de la sélection dans Lightroom), ni de marge de surface.
- **Hors de l'horaire de la plongée** liste les photos prises avant ou après la plongée, par exemple « Prise à 15:45, 7 minutes après la sortie de l'eau. Rattachée quand même à la plongée : elle reçoit ses infos et son lieu, sans profondeur. »
- Une plongée sans courbe affiche « Pas de courbe : pas de données par photo » à la place de l'horloge des appareils, et une plongée dont la courbe ne peut pas être lue « Courbe illisible : pas de données par photo » : les infos de plongée sont alors écrites sans données par photo.
- Avec **Aucune plongée — lieu seulement**, cette étape est sautée : **Horloges** est grisée dans les étapes.

### Étape 4 — Vérifier et appliquer

« Ce que Dive Lens écrit dans Lightroom. » Les mêmes options dans les deux chemins. Elles sont toutes cochées à chaque passage, sauf **Profondeur dans l'altitude GPS**, décochée à chaque passage : rien n'est mémorisé d'un passage à l'autre. À droite de chaque option, son effet sur ce passage, par exemple « 12 photos · 9 avec profondeur ».

**Métadonnées**

- **Détails de la plongée** : numéro de plongée, début, fin, durée, profondeur max, températures, note et source (l'application de plongée). Sur les photos prises pendant la plongée, aussi la profondeur, la température et le temps écoulé, et le décalage d'horloge.
  - **Profondeur dans l'altitude GPS**, en dessous, grisée quand **Détails de la plongée** est décochée : écrit aussi la profondeur dans l'altitude GPS, en valeur négative, en mètres, sur les photos qui ont une profondeur (section 5).
- **Détails du site** : le lieu et le GPS du site de la plongée, ou d'un autre site de votre carnet.
  - **Tout le catalogue** : **Site de chaque plongée**, ou **Autre site, le même pour toutes les photos…**.
  - **Les photos sélectionnées dans Lightroom** : **Site de la plongée : <nom>**, ou **Autre site…**.
  - **Autre site** : choisi par pays › lieu › site parmi les sites de votre carnet. Son GPS est modifiable avant d'appliquer, en degrés décimaux (par exemple `12.5000` et `-45.2500`). Les collections gardent le nom du site de leur plongée.
  - **Lieu** : pays, ville et lieu. **GPS** : la position. Décochez l'un pour n'écrire que l'autre. Un site sans GPS laisse en place le GPS déjà présent sur la photo.

**Collections**

- **Créer les collections** (une option de cette étape, plus un menu de Lightroom) : dans le chemin **Tout le catalogue**, une collection « numéro - site » par plongée qui a des photos ; dans le chemin **Les photos sélectionnées dans Lightroom**, la collection de la plongée choisie, qui reçoit toutes les photos sélectionnées. Une photo déjà dans la collection n'y est pas ajoutée deux fois. L'emplacement des collections se règle dans les Réglages : le lien **Emplacement dans Réglages** les ouvre (section 8).

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

**Écrit dans Lightroom** : **Photos taguées** (les photos écrites, y compris celles seulement rangées dans une collection), **Avec profondeur** et **Collections**, puis chaque collection avec son nombre de photos, **créée** ou **complétée**. Ensuite les avertissements (plongées sans photo, photos hors plongée, photos sans profondeur) et le rappel pour tout défaire (section 7).

- **Tout le catalogue** : **Retour aux plongées** ramène à l'étape 2, sans aucune plongée cochée. Quand une plongée n'a reçu aucune photo, **Traiter la plongée N à part** (ou **Traiter ces plongées à part**) mène directement à l'étape 3 avec ces seules plongées cochées, et relance la recherche de leurs photos : donnez à leur appareil son propre décalage (section 10).
- **Les photos sélectionnées dans Lightroom** : **Terminer** ramène à l'étape 1.

---

## 4. Ce qui est écrit

- **Créer les collections** : une collection par plongée, nommée **« numéro - site »**, dans l'ensemble de collections **Dive Lens** (l'emplacement se change dans les réglages). Une plongée sans numéro reçoit sa date et son heure de début à la place du numéro.
- **Détails de la plongée** : sur chaque photo, numéro de plongée, début et fin, durée, profondeur max, températures min et max, note et l'application de plongée d'où viennent les données. Sur les photos prises pendant une plongée qui a une courbe, aussi **la profondeur, la température et le temps écoulé** au moment de la photo, estimés entre les deux points de la courbe qui l'entourent. Et le décalage d'horloge appliqué : sur toutes les photos rangées dans une plongée (**Tout le catalogue**), sur les photos prises pendant la plongée (**Les photos sélectionnées dans Lightroom**).
- **Détails du site** : pays, ville, lieu et GPS, dans les champs standard de Lightroom.

Bon à savoir, dans les deux chemins :

- L'heure d'une photo est son **heure de prise de vue d'origine** : un TIFF revenu de Photoshop est rattaché comme son original.
- Un site sans pays, ville ou GPS ne touche pas aux valeurs déjà présentes sur la photo (une position GPS venue d'un téléphone est conservée).
- Avec **Détails de la plongée** décochée, Dive Lens ne touche pas du tout aux champs de plongée ; les photos sont quand même rattachées aux plongées pour le site et les collections.
- Les unités choisies dans les réglages valent pour les valeurs écrites. Les photos déjà taguées gardent l'unité en vigueur au moment où elles ont été écrites.

Dans le chemin **Tout le catalogue** :

- Seules les photos rangées dans une plongée sont écrites, dans la plongée ou dans la marge de surface. Les appareils ignorés et les photos hors plongée ne sont pas touchés.
- Une photo dans la marge de surface reçoit les infos de plongée, le lieu et le décalage d'horloge, sans données par photo.
- Une photo qui tomberait dans deux plongées va à la plus proche.
- Relancer un groupe de plongées ne fait qu'ajouter : les photos déjà rangées restent où elles sont (corrigez-les à la main si besoin), et les photos retrouvées voient leurs valeurs réécrites.
- Une plongée dont la courbe ne peut pas être lue arrête **Appliquer** : rien n'est écrit.

Dans le chemin **Les photos sélectionnées dans Lightroom** :

- Toutes les photos sélectionnées au moment du clic sont écrites.
- Une photo prise hors de l'horaire de la plongée reçoit les infos de plongée et le lieu, sans données par photo ni décalage d'horloge.
- Une valeur absente de la plongée choisie (température, note…) efface l'ancienne valeur Dive Lens de la photo : les champs décrivent toujours une seule plongée.
- Avec **Détails de la plongée**, une photo qui porte déjà une **autre** plongée perd les valeurs propres à celle-ci : profondeur, température, temps écoulé et décalage d'horloge, remplacés par les nouveaux quand la photo a été prise pendant la nouvelle plongée. Son altitude GPS suit la section 5. La photo reste dans la collection de son ancienne plongée : à vous de la déplacer si besoin.
- Taguer une photo avec la plongée qu'elle porte déjà réécrit ses données par photo si elle a été prise pendant la plongée, avec le décalage affiché ; les autres photos gardent les leurs.
- Si la sélection change dans Lightroom au moment où vous cliquez sur **Appliquer** alors que des données par photo sont écrites, rien n'est écrit : « La sélection a changé dans Lightroom. Rien n'a été écrit : vérifiez les données par photo, puis appliquez à nouveau. »
- Une courbe illisible ne bloque jamais : les infos de plongée sont écrites sans données par photo.

Après **Appliquer**, Dive Lens retient le décalage de chaque appareil pour la fois suivante, dans les deux chemins : dans le chemin **Tout le catalogue**, les appareils qui ont une photo rangée dans une plongée ; dans le chemin **Les photos sélectionnées dans Lightroom**, les appareils qui ont une photo prise pendant la plongée, quand des données par photo sont écrites.

---

## 5. La profondeur dans l'altitude GPS

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

## 6. Le lieu seul, et les photos hors plongée

Dans le chemin **Tout le catalogue**, les photos prises hors de toute plongée (sur le bateau, à la plage, en snorkeling) ne reçoivent rien : l'étape 3 les montre sous **Hors plongée**, de même que le récapitulatif et **À savoir avant d'appliquer**.

Pour leur poser un lieu, ou les rattacher quand même à une plongée :

1. Sélectionnez-les dans Lightroom, puis **Bibliothèque › Modules externes › Ouvrir Dive Lens…** et choisissez la carte des photos sélectionnées.
2. À l'étape 2 :
   - pour les rattacher quand même à une plongée, choisissez la plongée : elles reçoivent ses infos et son lieu, sans profondeur ;
   - pour ne leur poser qu'un lieu, choisissez **Aucune plongée — lieu seulement**.

Avec **Aucune plongée — lieu seulement**, l'étape 3 est sautée. À l'étape 4, seul **Détails du site** agit : **Détails de la plongée** et **Créer les collections** sont grisées, et le site est **Autre site…**. Choisissez le site par pays › lieu › site, ajustez son GPS si besoin, puis **Appliquer à N photos**. Seuls le pays, la ville, le lieu et le GPS sont écrits : aucun champ de plongée et aucune collection.

---

## 7. Annuler

**Édition › Annuler** dans Lightroom défait tout ce qu'un **Appliquer** a écrit, en une seule fois, GPS et altitude compris, dans les deux chemins. Le menu Édition de Lightroom l'appelle **Annuler Dive Lens**.

- Une collection créée par **Appliquer** disparaît avec lui ; les photos ajoutées à une collection existante en ressortent.
- **La première fois**, quand l'ensemble de collections (**Dive Lens** par défaut) n'existe pas encore, Dive Lens le crée à part, juste avant d'écrire. Lightroom en fait une étape séparée, au nom de l'ensemble : le premier « Annuler » défait ce qui a été écrit, un second retire l'ensemble, devenu vide. Si l'écriture est refusée après la création de l'ensemble, l'ensemble vide reste : annulez-le, ou supprimez-le dans Lightroom.
- Si Lightroom n'a pas laissé Dive Lens écrire le GPS et l'altitude, tout le reste est écrit et Dive Lens le signale : choisissez d'abord **Édition › Annuler Dive Lens** dans Lightroom, puis appliquez à nouveau.

---

## 8. Réglages

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
- **Profondeur** et **Température** : **Comme MacDive** / **Comme Subsurface** (les unités choisies dans cette application), ou forcées en mètres / pieds et °C / °F. Elles valent pour l'application et pour les valeurs écrites sur les photos. L'altitude GPS est toujours en mètres (section 5).

**Collections**
- **Placer dans** : la racine du catalogue ou un ensemble de collections existant (Lightroom doit être connecté pour les parcourir). C'est là que l'option **Créer les collections** de l'étape 4 range les collections.
- **Regrouper dans l'ensemble** : « Dive Lens » par défaut, créé si besoin ; laissez vide pour ranger les collections directement à l'emplacement choisi. Un aperçu montre le chemin obtenu. Les collections déjà créées restent où elles sont.
- Si l'ensemble choisi dans **Placer dans** n'est plus dans le catalogue, l'étape 4 le signale tant que **Créer les collections** est cochée, et **Appliquer** reste grisé jusqu'à ce que vous choisissiez un autre emplacement ici.

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

## 9. Les champs Dive Lens dans Lightroom

Dans le module **Bibliothèque**, ouvrez le panneau **Métadonnées** et choisissez **Dive Lens** dans son menu. Les champs sont regroupés en quatre parties :

- **Photo** : Profondeur, Température, Temps écoulé ;
- **Plongée** : Plongée, Début de plongée, Fin de plongée, Durée, Profondeur max, Temp. min, Temp. max, Note ;
- **Site** : le lieu, la ville, le pays et le GPS de Lightroom ;
- **Technique** : Décalage horloge, Source.

Les champs Dive Lens restent dans le catalogue Lightroom : ils ne sont pas écrits dans les fichiers photo et ne suivent pas les exports. Seuls la position GPS, les champs de lieu et, si vous le choisissez, l'altitude les suivent.

---

## 10. Dépannage

**Une plongée n'a aucune photo.** Vérifiez l'horloge de l'appareil à l'étape 3 : utilisez **Suggérer**, ou réglez l'écart à la main. Si l'horloge d'un appareil ou de l'ordinateur de plongée a été changée pendant le voyage, traitez les jours d'avant et d'après le changement en deux groupes séparés, chacun avec son décalage : **Traiter la plongée N à part**, sur le récapitulatif, le fait pour les plongées restées sans photo.

**« Base MacDive introuvable ».** La base a peut-être été déplacée, par exemple dans iCloud : utilisez **Localiser la base…** ou **Réglages › Choisir…**. Si elle est dans iCloud, vérifiez qu'elle est bien téléchargée sur ce Mac.

**« Le carnet de plongée a changé… ».** Le carnet en service a changé (dans les réglages, par exemple) pendant que Dive Lens travaillait sur le précédent : l'action a donc été refusée. Après **Appliquer**, le bandeau orange indique « Le carnet de plongée a changé. Rien n'a été écrit : recommencez. » Après une action qui n'écrit rien (recherche des photos, affichage d'une courbe), il indique « Le carnet de plongée a changé : recommencez. » Il s'affiche à l'étape 1, sous les étapes, jusqu'à ce que vous cliquiez sur **Fermer** ou **Suivant**. Choisissez à nouveau les plongées : elles viennent maintenant du nouveau carnet.

**Dive Lens est désactivé après l'installation.** Lightroom peut présenter Dive Lens désactivé, notamment quand un module Dive Lens précédent avait été désactivé avant d'être retiré. Dans **Fichier › Gestionnaire de modules externes…**, choisissez Dive Lens, cliquez sur **Activer**, puis **Terminé**. Une seule fois suffit.

**« Lightroom déconnecté ».** Ouvrez Lightroom Classic, vérifiez dans **Fichier › Gestionnaire de modules externes** que Dive Lens est activé, puis choisissez à nouveau **Bibliothèque › Modules externes › Ouvrir Dive Lens…**.

**L'application ne s'ouvre pas (Mac).** Si macOS indique qu'il ne peut pas vérifier l'application, téléchargez à nouveau Dive Lens depuis sa page de publication : les versions publiées sont signées et vérifiées par Apple, et s'ouvrent sans ce message.

**Passer de MacDive à Subsurface.** Rien à migrer. Relancer une plongée retrouve sa collection quand le nom « numéro - site » est identique ; sinon une nouvelle collection est créée à côté. Les décalages d'horloge sont mémorisés séparément pour chaque application.

**Signaler un problème.** Passez **Réglages › Diagnostic › Niveau du journal** sur **Détaillé**, reproduisez le problème, puis **Afficher le journal** : le dossier contient les deux journaux. [Ouvrez ensuite un ticket](https://github.com/Ventrilok/divelens/issues/new/choose) qui décrit le problème, sans y joindre les journaux : le dépôt GitHub est public, et les journaux contiennent vos sites de plongée et des chemins de fichiers. S'ils sont nécessaires, ils vous seront demandés : envoyez-les uniquement par e-mail à [divelens@querton.com](mailto:divelens@querton.com), avec le numéro du ticket.

---

## 11. Confidentialité

- Votre carnet de plongée est seulement lu, jamais modifié. MacDive ou Subsurface peuvent rester ouverts pendant ce temps.
- Dive Lens n'envoie aucune donnée de plongée nulle part. Il n'accède à Internet que pour deux choses :
  - la carte du site : le service de carte (OpenFreeMap, données OpenStreetMap) reçoit la zone de carte affichée, rien d'autre. Sans Internet, la carte est remplacée par le message « Carte indisponible sans Internet. » ;
  - les mises à jour : GitHub reçoit la demande de la dernière version, et le téléchargement si vous installez, rien d'autre. **Réglages › Mises à jour › Vérifier automatiquement** coupe la vérification au lancement.
