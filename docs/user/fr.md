# Dive Lens — Guide d'utilisation

Dive Lens relie vos photos sous-marines dans Adobe Lightroom Classic à votre carnet de plongée. Il lit votre carnet **MacDive** (Mac) ou **Subsurface** (Mac et Windows), retrouve les photos prises pendant chaque plongée, les range dans une collection par plongée et y écrit les infos de plongée : numéro, heures, profondeur, températures, note et site. Quand la plongée a une courbe, chaque photo reçoit en plus **la profondeur, la température de l'eau et le temps écoulé au moment où elle a été prise**.

Dive Lens est gratuit et open source (GPL v3).

> Brouillon à relire. Les exemples (plongées, sites, appareils) sont fictifs.

---

## 1. Installer

Il vous faut Lightroom Classic et MacDive ou Subsurface sur le même ordinateur. Sur Windows, seul Subsurface est proposé.

1. Téléchargez le zip de votre système : `DiveLens-<version>-macOS.zip` ou `DiveLens-<version>-Windows.zip`.
2. Décompressez-le. Vous obtenez un dossier **DiveLens.lrplugin** (sur Mac, il apparaît comme un seul fichier). L'application Dive Lens est rangée dedans.
3. Placez ce dossier à un endroit où il restera, par exemple dans Documents. Lightroom le charge depuis cet endroit à chaque démarrage.
4. Dans Lightroom : **Fichier › Gestionnaire de modules externes… › Ajouter**, choisissez le dossier **DiveLens.lrplugin**, puis **Terminé**.

**Vous utilisiez l'ancien module Dive Lens (0.9 et avant) ?** Quittez Lightroom, remplacez l'ancien dossier DiveLens.lrplugin par le nouveau au même endroit, puis relancez Lightroom. Vos photos gardent leurs infos de plongée, et l'application reprend vos réglages (emplacement du carnet et ordre des plongées) à son premier lancement. Dive Lens est désormais gratuit : plus de clé de licence.

### Premier lancement sur Mac

Cette version n'est pas encore signée par Apple. La première fois que Lightroom ouvre l'application, macOS refuse de l'ouvrir, faute de pouvoir vérifier le développeur.

1. Fermez le message (**OK**, pas **Placer dans la corbeille**).
2. Ouvrez **Réglages Système › Confidentialité et sécurité**. Vers le bas, une ligne indique que Dive Lens a été bloquée : cliquez sur **Ouvrir quand même** et confirmez avec votre mot de passe.
3. Choisissez à nouveau le menu Dive Lens dans Lightroom.

Une seule fois suffit.

### Premier lancement sur Windows

Cette version n'est pas encore signée. Quand Lightroom ouvre l'application la première fois, Windows peut afficher **« Windows a protégé votre ordinateur »** : cliquez sur **Informations complémentaires**, puis **Exécuter quand même**. Une seule fois suffit.

L'application a besoin de **WebView2** de Microsoft, déjà présent sur Windows 11 et sur la plupart des Windows 10. Si l'application ne s'ouvre pas, installez « Microsoft Edge WebView2 Runtime » depuis le site de Microsoft.

---

## 2. Ouvrir Dive Lens

Dive Lens n'a pas de fenêtre dans Lightroom. Tout se passe dans l'application Dive Lens, que Lightroom ouvre pour vous :

- **Bibliothèque › Modules externes › Créer les collections…**
- **Bibliothèque › Modules externes › Tagger la sélection…** (seulement quand des photos sont sélectionnées)

Lightroom doit rester ouvert pendant que vous utilisez l'application. La barre latérale de l'application indique si Lightroom est connecté et combien de photos sont sélectionnées. Si elle affiche **Lightroom déconnecté**, rien ne peut être écrit : vérifiez que Lightroom est ouvert et que le module est activé dans le gestionnaire de modules externes.

La première fois, passez par les **Réglages** (section 5) : l'application de plongée et l'emplacement de votre carnet.

---

## 3. Créer les collections

L'usage principal : vous choisissez des plongées, Dive Lens range les photos prises pendant celles-ci.

### Étape 1 — Choisir les plongées

La liste montre toutes les plongées de votre carnet, regroupées par mois. Cherchez par numéro, site, lieu ou pays, et affinez avec les filtres (période, pays, profondeur, note, **Avec courbe**). Un clic sur une plongée affiche sa fiche à droite : courbe, durée, profondeur max, températures, note, GPS du site avec une carte, et vos notes.

Cochez une ou plusieurs plongées, puis **Suivant**.

Une plongée **sans courbe** (saisie à la main, ou venant d'un ancien ordinateur de plongée dont MacDive n'a pas gardé le téléchargement) fonctionne quand même : ses photos reçoivent les infos de plongée et le lieu, mais ni profondeur, ni température, ni temps.

### Étape 2 — Ajuster et vérifier

Dive Lens cherche dans tout le catalogue les photos prises sur la période des plongées choisies, et liste les appareils trouvés (deux boîtiers du même modèle sont distingués par leur numéro de série).

**Horloge des appareils.** L'horloge d'un appareil est rarement exactement à l'heure de l'ordinateur de plongée, et souvent encore à l'heure de la maison. Pour chaque appareil, réglez l'écart entre son horloge et celle du carnet :

- **Suggérer** propose le décalage qui place le plus de photos dans les plongées. Il essaie chaque quart d'heure entre −14 h et +14 h, et retrouve donc aussi un fuseau horaire oublié.
- **Suggérer pour tous** le fait pour tous les appareils d'un coup, sans toucher à ceux réglés à la main ni aux appareils ignorés.
- Vous pouvez aussi saisir le décalage (heures:minutes) ou l'ajuster vous-même.
- **Ignorer** écarte un appareil, par exemple un téléphone resté sur le bateau : ses photos ne sont ni comptées ni taguées.

Dive Lens retient le dernier décalage utilisé pour chaque appareil et le propose la fois suivante.

**Inclure les photos prises en surface.** Facultatif, 0 minute par défaut : jusqu'à N minutes avant et après chaque plongée, pour les photos prises sur le bateau juste avant la mise à l'eau ou juste après la sortie. Elles rejoignent la collection de la plongée et reçoivent les infos de plongée, sans profondeur.

**Aperçu par plongée.** Les compteurs suivent vos réglages : photos par plongée, appareils, et si les données par photo seront écrites. Une plongée **sans photo** est mise en évidence : c'est souvent le signe d'une horloge changée pendant le voyage (voir section 7).

La barre du bas rappelle ce qui sera écrit et où iront les collections. Cliquez sur **Appliquer à N photos**.

### Étape 3 — Récapitulatif

Vous voyez les collections créées ou complétées, les photos taguées, combien ont reçu une profondeur, et les plongées sans photo (aucune collection n'est créée pour elles).

### Ce qui est écrit

- Une collection par plongée, nommée **« numéro - site »**, dans l'ensemble de collections **Dive Lens** (l'emplacement se change dans les réglages).
- Sur chaque photo : numéro de plongée, début et fin, durée, profondeur max, températures min et max, note, l'application de plongée d'où viennent les données, et le décalage d'horloge appliqué.
- Le lieu : pays, ville, lieu et GPS, dans les champs standard de Lightroom.
- Quand la plongée a une courbe : **la profondeur, la température et le temps écoulé** au moment de la photo, estimés entre les deux points de la courbe qui l'entourent.

Bon à savoir :

- L'heure d'une photo est son **heure de prise de vue d'origine** : un TIFF revenu de Photoshop est rattaché comme son original.
- Une photo qui tomberait dans deux plongées va à la plus proche.
- Relancer un groupe de plongées ne fait qu'ajouter : les photos déjà rangées restent où elles sont (corrigez-les à la main si besoin), et les photos retrouvées voient leurs valeurs réécrites.

**Annuler.** **Édition › Annuler** dans Lightroom défait tout en une seule fois, sauf le GPS (et l'altitude, si vous y copiez la profondeur) : Lightroom ne les remet pas.

---

## 4. Tagger la sélection

Pour des photos sans heure fiable, pour rattacher des photos à une plongée à la main, ou pour ne poser qu'un lieu (photos sur le bateau, à la plage, en snorkeling).

1. Dans Lightroom, sélectionnez les photos. L'application suit la sélection en direct et en montre les vignettes.
2. Cochez ce qu'il faut écrire :
   - **Données de plongée** : cliquez ensuite sur une plongée dans la liste. La plongée dont les heures couvrent la sélection porte le repère **Couvre la sélection** (il tient compte du dernier décalage de l'appareil). Ce n'est qu'une aide : c'est vous qui choisissez.
   - **Lieu** : le **Site de la plongée** choisie, ou un **Autre site** choisi par pays › lieu › site parmi les sites de votre carnet. Son GPS est modifiable (en degrés décimaux, par exemple `-20.2531` et `57.5125`).
3. Cliquez sur **Appliquer à N photos**. Ce sont les photos sélectionnées dans Lightroom **à ce moment-là** qui sont écrites.

Ce qui est écrit : avec **Données de plongée**, le n° de plongée, le début, la fin, la durée, la profondeur max, les températures et la note (pas de profondeur par photo ni de collection) ; avec **Lieu**, le pays, la ville, le lieu et le GPS.

- Une valeur absente de la plongée choisie (température, note…) efface l'ancienne valeur Dive Lens de la photo : les champs décrivent toujours une seule plongée.
- Un site sans pays, ville ou GPS ne touche pas aux valeurs déjà présentes sur la photo (une position GPS venue d'un téléphone est conservée).
- Avec **Données de plongée**, une photo qui porte déjà une **autre** plongée perd les valeurs propres à celle-ci : profondeur, température, temps écoulé et décalage d'horloge. Son altitude GPS n'est effacée que si c'est celle que Dive Lens a copiée depuis l'ancienne profondeur ; toute autre altitude est conservée. La photo reste dans sa collection : à vous de la déplacer si besoin.
- **Édition › Annuler** dans Lightroom défait tout en une fois, sauf le GPS (et l'altitude, quand elle a été effacée) : Lightroom ne les remet pas.

---

## 5. Réglages

Chaque modification est enregistrée immédiatement.

**Carnet de plongée**
- **Application** : MacDive ou Subsurface. Seules les applications disponibles sur votre système sont proposées : sur Windows, Subsurface. L'application se change seulement ici, jamais depuis Créer les collections ni Tagger la sélection.
- **Base MacDive** / **Carnet Subsurface** : l'emplacement standard est utilisé, sauf si vous en choisissez un autre. Un carnet Subsurface est soit un fichier (.xml ou .ssrf), soit le dossier de son stockage git (le stockage en ligne en garde une copie sur votre ordinateur). Sous l'emplacement, Dive Lens indique si le carnet est valide et combien il contient de plongées et de sites.
- Chaque application garde son emplacement : passer de l'une à l'autre puis revenir retrouve le vôtre.
- **Un seul carnet à la fois.** Changer d'application ou de carnet remet tout à zéro : Créer les collections revient à l'étape 1, sans aucune plongée cochée, et Tagger la sélection repart de zéro. C'est vrai quelle que soit la façon dont le carnet change : ici, par un import des réglages, depuis l'écran affiché quand le carnet ne peut pas être lu, ou quand Dive Lens reprend les réglages de l'ancien module en se connectant à Lightroom (section 1).
- Pendant que Dive Lens écrit dans Lightroom, le carnet ne peut pas changer : **Application**, **Choisir…** (ou **Choisir un fichier…** / **Choisir un dossier…**), **Emplacement standard**, **Importer…** et **Remplacer** sont grisés, avec la mention « Écriture dans Lightroom en cours : le carnet de plongée pourra être changé une fois celle-ci terminée. »

**Affichage**
- **Langue** : par défaut, celle du système. Les menus et les champs dans Lightroom suivent la langue de Lightroom.
- **Profondeur** et **Température** : **Comme MacDive** / **Comme Subsurface** (les unités choisies dans cette application), ou forcées en mètres / pieds et °C / °F. Elles valent pour l'application et pour les valeurs écrites sur les photos.

**Écriture sur les photos**
- **Copier la profondeur dans l'altitude GPS** (désactivé par défaut) : écrit aussi la profondeur de chaque photo comme une altitude négative, en mètres.
  - Le champ Profondeur de Dive Lens n'existe que dans Lightroom : il disparaît à l'export.
  - L'altitude GPS suit la photo exportée et se voit dans les autres logiciels.
  - La copie remplace une altitude GPS déjà présente sur la photo, et « Annuler » dans Lightroom ne la remet pas. Les photos sans profondeur gardent la leur.

**Collections**
- **Placer dans** : la racine du catalogue ou un ensemble de collections existant (Lightroom doit être connecté pour les parcourir).
- **Regrouper dans l'ensemble** : « Dive Lens » par défaut, créé si besoin ; laissez vide pour ranger les collections directement à l'emplacement choisi. Un aperçu montre le chemin obtenu. Les collections déjà créées restent où elles sont.

**Sauvegarde des réglages**
- **Exporter ou importer** : pour retrouver vos réglages sur un autre ordinateur, ou après une réinstallation.
- **Exporter…** les enregistre dans un fichier, nommé « Réglages Dive Lens.json » sauf si vous choisissez un autre nom. Le fichier contient : l'application de plongée et l'emplacement du carnet de chacune, la langue, les unités, l'option altitude GPS, l'emplacement des collections, le niveau du journal et l'ordre de la liste des plongées. Les décalages d'horloge retenus pour chaque appareil n'y figurent pas : ils restent sur cet ordinateur.
- **Importer…** demande confirmation avant de rien changer : **Remplacer** met les réglages du fichier à la place de tous ceux de cet écran, **Annuler** les laisse tels quels.
- Si le carnet du fichier n'est pas sur cet ordinateur, Dive Lens le signale et garde son emplacement jusqu'à ce que vous indiquiez le bon (**Choisir…**, ou **Choisir un fichier…** / **Choisir un dossier…** pour Subsurface). Si l'ensemble de collections choisi dans **Placer dans** n'est pas dans ce catalogue, Créer les collections le signale : choisissez un autre emplacement dans les réglages.
- Un fichier venu d'un Mac réglé sur MacDive, importé sur Windows : Dive Lens utilise Subsurface, l'application disponible sur Windows, et garde l'emplacement MacDive : ces réglages, exportés à nouveau puis importés sur un Mac, y utilisent toujours MacDive.
- Un fichier qui ne ressemble pas à un fichier de réglages Dive Lens est refusé, et rien ne change. Un fichier exporté par une version antérieure de Dive Lens reste importable.
- Après un import, Dive Lens ne reprend plus les réglages de l'ancien module (section 1) : ce sont ceux du fichier qui sont gardés.

**Diagnostic**
- **Niveau du journal** : Normal ou Détaillé. À augmenter seulement pour signaler un problème.
- **Afficher le journal** : ouvre le dossier des deux journaux, celui de l'application (`DiveLens App.log`) et celui du module Lightroom (`DiveLens.log`).

---

## 6. Les champs Dive Lens dans Lightroom

Dans le module **Bibliothèque**, ouvrez le panneau **Métadonnées** et choisissez **Dive Lens** dans son menu. Les champs sont regroupés en quatre parties :

- **Photo** : Profondeur, Température, Temps écoulé ;
- **Plongée** : Plongée, Début de plongée, Fin de plongée, Durée, Profondeur max, Temp. min, Temp. max, Note ;
- **Site** : le lieu, la ville, le pays et le GPS de Lightroom ;
- **Technique** : Décalage horloge, Source.

Les champs Dive Lens restent dans le catalogue Lightroom : ils ne sont pas écrits dans les fichiers photo et ne suivent pas les exports. Seuls la position GPS, les champs de lieu et, si vous le choisissez, l'altitude les suivent.

---

## 7. Dépannage

**Une plongée n'a aucune photo.** Vérifiez l'horloge de l'appareil : utilisez **Suggérer**, ou réglez l'écart à la main. Si l'horloge d'un appareil ou de l'ordinateur de plongée a été changée pendant le voyage, traitez les jours d'avant et d'après le changement en deux groupes séparés, chacun avec son décalage (**Traiter la plongée N à part**, sur le récapitulatif, le fait pour les plongées restées sans photo).

**« Base MacDive introuvable ».** La base a peut-être été déplacée, par exemple dans iCloud : utilisez **Localiser la base…** ou **Réglages › Choisir…**. Si elle est dans iCloud, vérifiez qu'elle est bien téléchargée sur ce Mac.

**« Le carnet de plongée a changé… ».** Le carnet en service a changé (dans les réglages, par exemple) pendant que Dive Lens travaillait sur le précédent : l'action a donc été refusée. Après **Appliquer**, le bandeau orange indique « Le carnet de plongée a changé. Rien n'a été écrit : recommencez. » Après une action qui n'écrit rien (recherche des photos, affichage d'une courbe), il indique « Le carnet de plongée a changé : recommencez. » Il s'affiche à l'étape 1 de Créer les collections ou dans Tagger la sélection, jusqu'à ce que vous cliquiez sur **Fermer** ou passiez à la suite. Choisissez à nouveau les plongées : elles viennent maintenant du nouveau carnet.

**« Lightroom déconnecté ».** Ouvrez Lightroom Classic, vérifiez dans **Fichier › Gestionnaire de modules externes** que Dive Lens est activé, puis choisissez à nouveau un des menus Dive Lens.

**L'application ne s'ouvre pas (Mac).** Voir « Premier lancement sur Mac » plus haut : macOS la bloque tant que vous n'avez pas cliqué une fois sur **Ouvrir quand même**.

**Passer de MacDive à Subsurface.** Rien à migrer. Relancer une plongée retrouve sa collection quand le nom « numéro - site » est identique ; sinon une nouvelle collection est créée à côté. Les décalages d'horloge sont mémorisés séparément pour chaque application.

**Signaler un problème.** Passez **Réglages › Diagnostic › Niveau du journal** sur **Détaillé**, reproduisez le problème, puis **Afficher le journal** : le dossier contient les deux journaux. [Ouvrez ensuite un ticket](https://github.com/Ventrilok/divelens/issues/new/choose) qui décrit le problème, sans y joindre les journaux : le dépôt GitHub est public, et les journaux contiennent vos sites de plongée et des chemins de fichiers. S'ils sont nécessaires, ils vous seront demandés : envoyez-les uniquement par e-mail à [divelens@querton.com](mailto:divelens@querton.com), avec le numéro du ticket.

---

## 8. Confidentialité

- Votre carnet de plongée est seulement lu, jamais modifié. MacDive ou Subsurface peuvent rester ouverts pendant ce temps.
- Dive Lens n'envoie aucune donnée de plongée nulle part. Le seul accès à Internet est la carte du site : le service de carte (OpenFreeMap, données OpenStreetMap) reçoit la zone de carte affichée, rien d'autre. Sans Internet, la carte est remplacée par le message « Carte indisponible sans Internet. »
