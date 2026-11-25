# Dive Lens — Benutzerhandbuch

Dive Lens verknüpft Ihre Unterwasserfotos in Adobe Lightroom Classic mit Ihrem Logbuch. Es liest Ihr Logbuch aus **MacDive** (Mac) oder **Subsurface** (Mac und Windows). In einem Ablauf mit fünf Schritten wählen Sie die Fotos und die Tauchgänge, stellen die Kamerauhren ein, prüfen, was geschrieben wird, und wenden dann an: Dive Lens legt die Fotos in eine Sammlung pro Tauchgang und schreibt die Tauchgangsdaten darauf: Tauchgangsnummer, Zeiten, Tiefe, Temperaturen, Bewertung und Tauchplatz. Hat der Tauchgang ein Profil, erhält jedes Foto außerdem die **Tiefe, die Wassertemperatur und die Tauchzeit im Moment der Aufnahme**.

Dive Lens ist kostenlos und Open Source (GPL v3).

> Entwurf zur Durchsicht. Die Beispiele (Tauchgänge, Tauchplätze, Kameras) sind erfunden.

---

## 1. Installieren

Sie brauchen Lightroom Classic und MacDive oder Subsurface auf demselben Computer. Unter Windows ist nur Subsurface verfügbar.

**Sie kommen vom früheren Dive Lens-Zusatzmodul (0.9 und älter)?** Entfernen Sie es, bevor Sie das Installationsprogramm verwenden: in Lightroom **Datei › Zusatzmodul-Manager…**, Dive Lens wählen, **Entfernen**, dann **Fertig**. Ist **Entfernen** ausgegraut, liegt das alte Zusatzmodul bereits dort, wo das Installationsprogramm es ersetzt: Es ist nichts zu tun. Ihre Fotos behalten ihre Tauchgangsdaten, und die App übernimmt beim ersten Start Ihre früheren Einstellungen (Speicherort des Logbuchs und Reihenfolge der Tauchgänge). Dive Lens ist jetzt kostenlos: Es gibt keinen Lizenzschlüssel mehr.

### Mit dem Installationsprogramm

1. Beenden Sie Lightroom.
2. Laden Sie das Installationsprogramm für Ihr System herunter: `DiveLens-<version>-macOS.pkg` oder `DiveLens-<version>-Windows-Setup.exe`.
3. Öffnen Sie es und folgen Sie den Schritten. Es installiert Dive Lens nur für Ihr Benutzerkonto, ohne Administratorkennwort.
4. Starten Sie Lightroom: Dive Lens ist in **Bibliothek › Plug-in-Extras**. Im Zusatzmodul-Manager ist nichts hinzuzufügen.

Unter Windows ist das Installationsprogramm noch nicht signiert: Windows zeigt möglicherweise **„Der Computer wurde durch Windows geschützt“**. Klicken Sie auf **Weitere Informationen**, dann auf **Trotzdem ausführen**.

### Mit der Zip-Datei, von Hand

1. Laden Sie die Zip-Datei für Ihr System herunter: `DiveLens-<version>-macOS.zip` oder `DiveLens-<version>-Windows.zip`.
2. Entpacken Sie sie. Sie erhalten einen Ordner namens **DiveLens.lrplugin** (auf einem Mac sieht er wie eine einzelne Datei aus). Die Dive Lens-App liegt darin.
3. Verschieben Sie diesen Ordner an einen Ort, an dem er bleibt, zum Beispiel in Ihren Dokumente-Ordner. Lightroom lädt ihn bei jedem Start von dort. Wenn Sie das frühere Zusatzmodul verwendet haben, beenden Sie stattdessen Lightroom und ersetzen Sie den alten Ordner DiveLens.lrplugin an derselben Stelle durch den neuen: Schritt 4 können Sie dann überspringen.
4. In Lightroom: **Datei › Zusatzmodul-Manager… › Hinzufügen**, den Ordner **DiveLens.lrplugin** wählen, dann **Fertig**.

Auf einem Mac fragt macOS möglicherweise, wenn Lightroom die App zum ersten Mal öffnet, ob Sie eine aus dem Internet geladene App öffnen möchten: Klicken Sie auf **Öffnen**. Unter Windows ist die App noch nicht signiert: Zeigt Windows **„Der Computer wurde durch Windows geschützt“**, klicken Sie auf **Weitere Informationen**, dann auf **Trotzdem ausführen**. Das ist nur einmal nötig.

### WebView2 unter Windows

Die App braucht **WebView2** von Microsoft, das unter Windows 11 und auf den meisten Windows-10-Computern bereits vorhanden ist. Öffnet sich die App nicht, installieren Sie die „Microsoft Edge WebView2 Runtime“ von der Microsoft-Website.

### Deinstallieren

Ihre Fotos behalten ihre Tauchgangsdaten und ihre Sammlungen im Katalog: Sie sind wieder da, wenn Sie Dive Lens erneut installieren.

**Auf einem Mac**

1. Beenden Sie Lightroom.
2. Wählen Sie im Finder **Gehe zu › Gehe zum Ordner…**, fügen Sie `~/Library/Application Support/Adobe/Lightroom/Modules` ein und legen Sie dann **DiveLens.lrplugin** in den Papierkorb. Haben Sie Dive Lens aus der Zip-Datei installiert, entfernen Sie es zuerst in **Datei › Zusatzmodul-Manager…** (**Entfernen**) und legen Sie dann den Ordner DiveLens.lrplugin von dort, wo Sie ihn abgelegt hatten, in den Papierkorb.
3. Optional, um auch die Einstellungen von Dive Lens zu löschen: Legen Sie den Ordner `~/Library/Application Support/com.querton.divelens.desktop` in den Papierkorb.

**Unter Windows**

1. Beenden Sie Lightroom.
2. **Einstellungen › Apps › Installierte Apps › Dive Lens › Deinstallieren**. Haben Sie Dive Lens aus der Zip-Datei installiert, entfernen Sie es in **Datei › Zusatzmodul-Manager…** (**Entfernen**) und löschen Sie dann den Ordner DiveLens.lrplugin dort, wo Sie ihn abgelegt hatten. Sobald Dive Lens sie aktualisiert hat, steht auch eine aus der Zip-Datei installierte Kopie unter **Installierte Apps**: Deinstallieren Sie sie von dort.
3. Optional, um auch die Einstellungen von Dive Lens zu löschen: Löschen Sie den Ordner `%APPDATA%\com.querton.divelens.desktop`.

---

## 2. Dive Lens öffnen

Dive Lens hat kein Fenster in Lightroom. Alles geschieht in der Dive Lens-App, die Lightroom für Sie öffnet:

- **Bibliothek › Plug-in-Extras › Dive Lens öffnen…** öffnet die App bei Schritt 1 des Ablaufs (Abschnitt 3), ob Fotos ausgewählt sind oder nicht.
- **Hilfe › Plug-in-Extras › Dive Lens-Einstellungen…** öffnet sie in den **Einstellungen** (Abschnitt 9).

Der Eintrag Dive Lens in **Datei › Zusatzmodul-Manager…** hat außerdem die Schaltflächen **Dive Lens öffnen** und **Einstellungen öffnen** sowie Links zur Dive Lens-Website und zum Quellcode auf GitHub.

Ist die App bereits geöffnet, holt das Menü ihr Fenster nach vorn. Während des Ablaufs gewählt, bringt **Dive Lens öffnen…** Sie zu Schritt 1 zurück und behält Ihre Auswahl; in der Zusammenfassung gewählt, beginnt es einen neuen Durchlauf.

Lightroom muss geöffnet bleiben, solange Sie die App verwenden. Prüfen Sie beim ersten Mal die **Einstellungen** (Abschnitt 9): die Tauchanwendung und den Speicherort Ihres Logbuchs.

### Das Fenster

- **Oben**: links Dive Lens; rechts öffnet das Zahnrad die **Einstellungen** (Abschnitt 9). Das Zahnrad ist ausgegraut, während Dive Lens in Lightroom schreibt.
- **Unten** die Statusleiste. Links Lightroom:
  - **Lightroom verbunden**, **Lightroom nicht verbunden** oder **Lightroom neu starten**;
  - **Katalog:** der Name des in Lightroom geöffneten Katalogs;
  - **Kataloggröße:** wie viele Fotos er enthält, jedes Mal neu gezählt, wenn Sie zu Schritt 1 zurückkehren;
  - **Ausgewählte Fotos:** wie viele in Lightroom ausgewählt sind, live (**keine**, wenn keines ausgewählt ist).
- Rechts in der Statusleiste Ihr Logbuch: **Logbuch-App:** MacDive oder Subsurface, und **Tauchgänge im Logbuch:** wie viele Tauchgänge es enthält.

Meldet die Statusleiste **Lightroom nicht verbunden**, kann nichts geschrieben werden: Prüfen Sie, ob Lightroom geöffnet und das Zusatzmodul im Zusatzmodul-Manager aktiviert ist. Meldet sie **Lightroom neu starten**, wurde Dive Lens aktualisiert, während Lightroom geöffnet war: Beenden Sie Lightroom und starten Sie es neu.

---

## 3. Der Ablauf

Fünf Schritte, oben angezeigt: **Umfang › Tauchgänge › Uhren › Prüfen und anwenden › Zusammenfassung**. **Weiter** geht vor und **Zurück** geht zurück; ein Klick auf einen bereits erledigten Schritt führt ebenfalls dorthin zurück, außer von der Zusammenfassung aus. Unter den Schritten erinnert eine Zeile, die mit **Fotos:** beginnt, an Ihre Wahl. In Lightroom wird nichts geschrieben, bevor Sie in Schritt 4 auf **Auf N Fotos anwenden** klicken.

Die Leiste mit **Zurück** und **Weiter** (oder **Auf N Fotos anwenden**) bleibt unten im Fenster, direkt über der Statusleiste, und die Schritte bleiben oben: Ist das Fenster klein, scrollt nur der Inhalt dazwischen. Unter der Leiste ist nie etwas verborgen.

### Zwei Wege

Schritt 1 stellt die eine echte Frage: welche Fotos?

- **Gesamter Katalog**: Sie kreuzen einen oder mehrere Tauchgänge an, und Dive Lens sucht im ganzen Katalog die Fotos, die während jedes Tauchgangs aufgenommen wurden. Jedes Foto kommt zu dem Tauchgang, der zu seiner Uhrzeit passt. Für eine ganze Reise oder eine Reihe von Tauchgängen. Die in Lightroom ausgewählten Fotos werden ignoriert.
- **Lightroom-Auswahl**: die Fotos, die Sie in Lightroom ausgewählt haben. Sie wählen einen Tauchgang, und alle werden ihm zugeordnet, unabhängig von ihrer Uhrzeit. Für Fotos ohne verlässliche Uhrzeit, um Fotos von Hand einem Tauchgang zuzuordnen oder um ihnen nur einen Ort zu geben (Abschnitt 7).

Wählen Sie in Schritt 1 den anderen Weg, beginnen die folgenden Schritte von vorn: Tauchgänge, Uhren und Optionen.

### Schritt 1 — Umfang

„Mit welchen Fotos arbeiten?“ Zwei Karten: **Gesamter Katalog** oder **Lightroom-Auswahl**. Die Auswahlkarte folgt Lightroom live: die Kameras und der Tag der ausgewählten Fotos und bis zu sechs Miniaturen. Ist kein Foto ausgewählt, ist sie ausgegraut und meldet **Keine Fotos ausgewählt**: Wählen Sie Fotos in Lightroom aus, und sie wird aktiv. Wählen Sie eine Karte, dann **Weiter**.

### Schritt 2 — Tauchgänge

Die Liste zeigt jeden Tauchgang Ihres Logbuchs, nach Monat gruppiert. Suchen Sie nach Nummer, Tauchplatz, Ort oder Land und grenzen Sie mit den Filtern ein (**Zeitraum**, **Land**, **Tiefe**, **Bewertung**, **Mit Profil**, **Mit GPS**, **Mit Notizen**). **Sortieren** ordnet die Liste (**Neueste zuerst**, **Älteste zuerst**, **Max. Tiefe**); Dive Lens merkt sich Ihre Wahl. Klicken Sie auf einen Tauchgang, um rechts seine Details zu sehen: Profil, Dauer, max. Tiefe, Temperaturen, Bewertung, GPS des Tauchplatzes mit Karte und Ihre Notizen.

Jede Zeile hat am Ende drei Markierungen, die nur erscheinen, wenn der Tauchgang sie hat: **Profil**, **GPS** (sein Tauchplatz hat eine GPS-Position) und **Notizen**. Sonst bleibt die Zelle leer. Fahren Sie mit der Maus über eine Markierung, um sie zu lesen, zum Beispiel „GPS des Tauchplatzes vorhanden“. Die Spalte **GPS** und der Filter **Mit GPS** erscheinen nur, wenn die Tauchanwendung Positionen der Tauchplätze liefert, wie MacDive und Subsurface es tun.

- **Gesamter Katalog**: Kreuzen Sie einen oder mehrere Tauchgänge an, dann **Weiter**. Die Leiste zeigt, wie viele angekreuzt sind, und ihre Tage; **Auswahl aufheben** entfernt die Kreuze.
- **Lightroom-Auswahl**: Klicken Sie auf einen Tauchgang, dann **Weiter**.
  - Der Tauchgang, dessen Zeiten die Auswahl abdecken, trägt das Kennzeichen **Deckt die Auswahl ab**. Es verwendet den zuletzt auf jede Kamera angewendeten Zeitversatz, dann die Zeitversätze, die Sie in Schritt 3 einstellen. Es ist nur ein Hinweis: Sie entscheiden.
  - Die erste Zeile, **Kein Tauchgang — nur Ort**, dient dazu, den Fotos nur einen Ort zu geben (Abschnitt 7).

Ein Tauchgang **ohne Profil** (von Hand eingetragen oder von einem älteren Tauchcomputer, dessen Download MacDive nicht aufbewahrt hat) funktioniert trotzdem: Seine Fotos erhalten die Tauchgangsdaten und den Tauchplatz, aber keine Tiefe, Temperatur oder Zeit.

### Schritt 3 — Uhren

**Kamerauhren.** Die Uhr einer Kamera geht selten genau wie die des Tauchcomputers und steht oft noch auf der Zeit von zu Hause. Jede Kamera hat eine Karte (zwei Gehäuse desselben Modells werden an der Seriennummer unterschieden). Stellen Sie die Abweichung zwischen ihrer Uhr und der des Logbuchs ein:

- **Vorschlagen** schlägt den Zeitversatz vor, der die meisten Fotos in die Tauchgänge bringt. Es probiert jede Viertelstunde zwischen −14 h und +14 h und findet so auch eine vergessene Zeitzone.
- **Für alle vorschlagen** tut dies für alle Kameras auf einmal, ohne Kameras anzutasten, die Sie von Hand eingestellt haben oder ignorieren.
- Sie können den Zeitversatz auch eintippen (Stunden:Minuten) oder selbst anpassen.
- Jede Karte beginnt mit dem zuletzt auf diese Kamera angewendeten Zeitversatz, auf beiden Wegen (**gemerkter Zeitversatz**); gibt es keinen, beginnt sie bei 0:00. Zeitversätze werden für jede Tauchanwendung getrennt gemerkt. **Einstellungen › Gemerkte Zeitversätze** listet sie auf und vergisst den einer Kamera (Abschnitt 9).

**Vorschau pro Tauchgang.** Die Zahlen folgen Ihren Einstellungen: Fotos pro Tauchgang, Kameras und die Daten pro Foto, die jeder Tauchgang liefert (**Tiefe · Temp. · Zeit**, **Kein Profil: nur Tauchgangsdaten**, **Nur Oberflächenfotos**). Bei einem Tauchgang ohne Foto stehen seine Zahl, 0, und **Kein Foto — Uhr umgestellt?** in Orange: Oft wurde während der Reise eine Kamerauhr umgestellt (Abschnitt 11).

Auf dem Weg **Gesamter Katalog**:

- Dive Lens sucht im ganzen Katalog die Fotos, die im Zeitraum der angekreuzten Tauchgänge aufgenommen wurden.
- **Ignorieren** lässt eine Kamera aus, zum Beispiel ein Telefon, das auf dem Boot geblieben ist: Ihre Fotos werden weder gezählt noch getaggt.
- **An der Oberfläche aufgenommene Fotos einbeziehen**: bis zu N Minuten vor und nach jedem Tauchgang, für Fotos, die auf dem Boot kurz vor dem Einstieg oder kurz nach dem Ausstieg aufgenommen wurden. Standardmäßig 0 Minuten, und bei jedem Durchlauf wieder 0 (**Tauchgang N getrennt behandeln** und **Diese Tauchgänge getrennt behandeln** behalten den Wert). Diese Fotos erhalten die Tauchgangsdaten, ohne Tiefe.
- **Außerhalb der Tauchgänge**: die Fotos aus dem Zeitraum, die in keinen Tauchgang fallen, auch nicht mit der Marge (an Land, auf dem Boot…). Sie erhalten nichts (Abschnitt 7).

Auf dem Weg **Lightroom-Auswahl**:

- Die Fotos werden dem gewählten Tauchgang zugeordnet, unabhängig von ihrer Uhrzeit. Die Uhr dient nur dazu, ihnen eine Tiefe zu geben: Nur die während des Tauchgangs aufgenommenen erhalten eine.
- Eine Karte pro Kamera der Auswahl. Es gibt kein **Ignorieren** (entfernen Sie stattdessen ein Foto in Lightroom aus der Auswahl) und keine Oberflächenmarge.
- **Außerhalb der Tauchgangszeit** listet die Fotos auf, die vor oder nach dem Tauchgang aufgenommen wurden, zum Beispiel „Aufgenommen um 15:45, 7 Minuten nach dem Ausstieg aus dem Wasser. Trotzdem dem Tauchgang zugeordnet: Es erhält seine Daten und seinen Ort, ohne Tiefe.“
- Ein Tauchgang ohne Profil zeigt statt der Kamerauhren „Kein Profil: keine Daten pro Foto“, und ein Tauchgang, dessen Profil nicht gelesen werden kann, „Profil nicht lesbar: keine Daten pro Foto“: Die Tauchgangsdaten werden dann ohne Daten pro Foto geschrieben.
- Mit **Kein Tauchgang — nur Ort** wird dieser Schritt übersprungen: **Uhren** ist in den Schritten ausgegraut.

### Schritt 4 — Prüfen und anwenden

„Was Dive Lens in Lightroom schreibt.“ Dieselben Optionen auf beiden Wegen. Sie sind bei jedem Durchlauf alle angekreuzt, außer **Tiefe in die GPS-Höhe kopieren**, das bei jedem Durchlauf nicht angekreuzt ist: Von einem Durchlauf zum nächsten wird nichts gemerkt. Rechts neben jeder Option steht ihre Wirkung in diesem Durchlauf, zum Beispiel „12 Fotos · 9 mit Tiefe“. [Abschnitt 4](#4-welche-optionen-für-welchen-bedarf) zeigt, was jede Kombination von Optionen auf die Fotos schreibt und welche Optionen für welchen Bedarf zu wählen sind.

**Metadaten**

- **Tauchgangsdetails**: Tauchgangsnummer, Beginn, Ende, Dauer, max. Tiefe, Temperaturen, Bewertung und Quelle (die Tauchanwendung). Auf den während des Tauchgangs aufgenommenen Fotos auch die Tiefe, die Temperatur und die Tauchzeit sowie der Zeitversatz.
  - **Tiefe in die GPS-Höhe kopieren**, darunter, ausgegraut, wenn **Tauchgangsdetails** nicht angekreuzt ist: schreibt die Tiefe zusätzlich in die GPS-Höhe, als negativen Wert, in Metern, auf den Fotos, die eine Tiefe haben (Abschnitt 6).
- **Tauchplatzdetails**: der Ort und das GPS des Tauchplatzes des Tauchgangs oder eines anderen Tauchplatzes aus Ihrem Logbuch.
  - **Gesamter Katalog**: **Tauchplatz jedes Tauchgangs** oder **Anderer Tauchplatz, derselbe für alle Fotos…**.
  - **Lightroom-Auswahl**: **Tauchplatz des Tauchgangs: <Name>** oder **Anderer Tauchplatz…**.
  - **Anderer Tauchplatz**: gewählt über Land › Ort › Tauchplatz unter den Tauchplätzen Ihres Logbuchs. Sein GPS lässt sich vor dem Anwenden bearbeiten, in Dezimalgrad (zum Beispiel `12.5000` und `-45.2500`). Die Sammlungen behalten den Namen des eigenen Tauchplatzes ihres Tauchgangs.
  - **Ort**: Land, Stadt und Ortsdetail. **GPS**: die Position. Entfernen Sie ein Kreuz, um nur das andere zu schreiben. Ein Tauchplatz ohne GPS lässt das GPS, das das Foto schon hat, unverändert.

**Sammlungen**

- **Sammlungen erstellen** (eine Option dieses Schritts, kein Lightroom-Menü mehr): auf dem Weg **Gesamter Katalog** eine Sammlung pro Tauchgang mit Fotos, in der Regel „<Nummer> - <Tauchplatz>“ genannt (Abschnitt 5); auf dem Weg **Lightroom-Auswahl** die Sammlung des gewählten Tauchgangs, die alle ausgewählten Fotos erhält. Ein Foto, das schon in der Sammlung ist, wird nicht doppelt hinzugefügt. Wo die Sammlungen abgelegt werden, legen Sie in den Einstellungen fest: Der Link **Ablageort in den Einstellungen** öffnet sie (Abschnitt 9).

**Gut zu wissen vor dem Anwenden** listet auf, was nichts oder weniger erhält: die Tauchgänge ohne Foto und die Fotos außerhalb der Tauchgänge (**Gesamter Katalog**), die Fotos, die keine Tiefe erhalten (**Lightroom-Auswahl**), und die verwendeten Einheiten.

Die Leiste unten fasst zusammen, was geschrieben wird („N Fotos · N Sammlungen“), und erinnert daran, dass ein einziges Rückgängig alles zurücknimmt. Klicken Sie auf **Auf N Fotos anwenden**. Auf dem Weg **Gesamter Katalog** ist N jedes in den angekreuzten Tauchgängen gefundene Foto. Auf dem Weg **Lightroom-Auswahl** werden die Fotos geschrieben, die **im Moment des Klicks** in Lightroom ausgewählt sind.

**Anwenden** bleibt ausgegraut, mit dem Grund in der Leiste, bis etwas geschrieben werden kann, zum Beispiel:

- „Kreuzen Sie mindestens eine Option an“;
- „Wählen Sie einen Tauchplatz“, solange unter **Anderer Tauchplatz** kein Tauchplatz gewählt ist;
- „Wählen Sie Fotos in Lightroom aus“, wenn die Auswahl leer ist;
- „Mit diesen Optionen ist nichts zu schreiben.“;
- der in den Einstellungen gewählte Sammlungssatz ist nicht mehr im Katalog (nur wenn **Sammlungen erstellen** angekreuzt ist): Mit **Einstellungen öffnen** wählen Sie einen anderen Ort;
- Lightroom ist nicht verbunden.

### Schritt 5 — Zusammenfassung

**In Lightroom geschrieben**: **Getaggte Fotos** (die geschriebenen Fotos, einschließlich derer, die nur in eine Sammlung gelegt wurden), **Mit Tiefe** und **Sammlungen**, dann jede Sammlung mit ihrer Anzahl Fotos, **erstellt** oder **ergänzt**. Danach die Hinweise (Tauchgänge ohne Foto, Fotos außerhalb der Tauchgänge, Fotos ohne Tiefe) und die Erinnerung, wie man rückgängig macht (Abschnitt 8).

- **Gesamter Katalog**: **Zurück zur Wahl des Umfangs** kehrt zu Schritt 1 zurück und beginnt einen neuen Durchlauf: **Gesamter Katalog** ist weiterhin gewählt, kein Tauchgang ist angekreuzt, und die Optionen stehen wieder auf ihren Standardwerten. Hat ein Tauchgang kein Foto erhalten, geht **Tauchgang N getrennt behandeln** (oder **Diese Tauchgänge getrennt behandeln**) direkt zu Schritt 3, nur mit diesen Tauchgängen angekreuzt, und sucht ihre Fotos erneut: Geben Sie ihrer Kamera ihren eigenen Zeitversatz (Abschnitt 11).
- **Lightroom-Auswahl**: **Fertig** kehrt zu Schritt 1 zurück und beginnt auf dieselbe Weise einen neuen Durchlauf, die Karte **Lightroom-Auswahl** bleibt gewählt.

---

## 4. Welche Optionen für welchen Bedarf

Die Optionen von Schritt 4 entscheiden, was Dive Lens auf jedes Foto schreibt. Dieser Abschnitt zeigt, was jede Kombination schreibt, Weg für Weg, und dann, welche Optionen in Situationen zu wählen sind, denen Taucher oft begegnen. Die genauen Regeln stehen in [Abschnitt 5](#5-was-geschrieben-wird).

### Wie die Optionen zusammenwirken

- **Tauchgangsdetails**, **Tauchplatzdetails** und **Sammlungen erstellen** sind unabhängig: Was eine davon schreibt, hängt nie von den anderen ab, die Zeilen der Tabellen unten lassen sich also kombinieren (eine Ausnahme auf dem Weg **Gesamter Katalog**: Ein Tauchgang, dessen Profil nicht gelesen werden kann, hält **Anwenden** auf, was auch immer angekreuzt ist, Anmerkung 1). Zum Beispiel schreiben **Tauchgangsdetails** und **Sammlungen erstellen**, angekreuzt ohne **Tauchplatzdetails**, die Tauchgangsdaten und die Sammlungen und lassen Ort und GPS, wie sie sind.
- **Tiefe in die GPS-Höhe kopieren** wirkt nur mit **Tauchgangsdetails**: Ist **Tauchgangsdetails** nicht angekreuzt, ist es ausgegraut.
- Unter **Tauchplatzdetails** sind **Ort** und **GPS** ebenfalls unabhängig; beide nicht anzukreuzen ist dasselbe, wie **Tauchplatzdetails** nicht anzukreuzen. Beide stammen vom eigenen Tauchplatz des Tauchgangs oder von **Anderer Tauchplatz**.
- **Anderer Tauchplatz** ändert nur den Ort und das GPS, die auf die Fotos geschrieben werden: Jede Sammlung behält den Namen des eigenen Tauchplatzes ihres Tauchgangs.
- Auch wenn **Tauchgangsdetails** nicht angekreuzt ist, werden die Fotos weiterhin anhand ihrer Uhrzeit ihrem Tauchgang zugeordnet (**Gesamter Katalog**), für den Tauchplatz und die Sammlungen.
- **Kein Tauchgang — nur Ort** (Weg **Lightroom-Auswahl**) lässt nur **Tauchplatzdetails** übrig: **Tauchgangsdetails** und **Sammlungen erstellen** sind ausgegraut, und der Tauchplatz ist immer **Anderer Tauchplatz…**.

Die geschriebenen Fotos:

- **Gesamter Katalog**: die Fotos, die in einen angekreuzten Tauchgang gelegt wurden, aufgenommen während des Tauchgangs oder in seiner Oberflächenmarge. Ignorierte Kameras und Fotos außerhalb der Tauchgänge werden nicht angetastet.
- **Lightroom-Auswahl**: jedes Foto, das im Moment des Klicks auf **Anwenden** ausgewählt ist, unabhängig von seiner Uhrzeit.

### Was jedes Foto erhält

In den Tabellen bedeutet „Bleibt“, dass Dive Lens das Feld nicht antastet: Das Foto behält, was es hatte, aus einem früheren Durchlauf, von der Kamera oder von Ihnen. „Keine“ unter **Sammlung** bedeutet, dass keine Sammlung erstellt oder geändert wird: Das Foto bleibt in den Sammlungen, in denen es schon war. Ein Tauchplatz ohne Land, Stadt, Ortsdetail oder GPS lässt diesen Wert auf dem Foto, wie er war. Die geschriebenen Tauchgangsfelder beschreiben nur einen Tauchgang: Ein Wert, den der Tauchgang nicht hat, etwa eine Temperatur oder eine Bewertung, wird geleert.

**Gesamter Katalog**

| In Schritt 4 angekreuzt | Tauchgangsfelder | Tiefe, Temperatur, Zeit | Zeitversatz | GPS-Höhe | Land, Stadt, Ortsdetail | GPS | Sammlung |
|---|---|---|---|---|---|---|---|
| Alles (der Standard) | Geschrieben | Während des Tauchgangs¹ | Geschrieben | Bleibt² | Tauchplatz jedes Tauchgangs | Tauchplatz jedes Tauchgangs | Die seines Tauchgangs |
| Alles, und **Tiefe in die GPS-Höhe kopieren** | Geschrieben | Während des Tauchgangs¹ | Geschrieben | Die Tiefe³ | Tauchplatz jedes Tauchgangs | Tauchplatz jedes Tauchgangs | Die seines Tauchgangs |
| Nur **Tauchgangsdetails** | Geschrieben | Während des Tauchgangs¹ | Geschrieben | Bleibt² | Bleibt | Bleibt | Keine |
| Nur **Tauchgangsdetails** und **Tiefe in die GPS-Höhe kopieren** | Geschrieben | Während des Tauchgangs¹ | Geschrieben | Die Tiefe³ | Bleibt | Bleibt | Keine |
| Nur **Tauchplatzdetails** (**Ort** und **GPS**) | Bleibt | Bleibt | Bleibt | Bleibt | Tauchplatz jedes Tauchgangs | Tauchplatz jedes Tauchgangs | Keine |
| Nur **Ort** | Bleibt | Bleibt | Bleibt | Bleibt | Tauchplatz jedes Tauchgangs | Bleibt | Keine |
| Nur **GPS** | Bleibt | Bleibt | Bleibt | Bleibt | Bleibt | Tauchplatz jedes Tauchgangs | Keine |
| Alles, mit **Anderer Tauchplatz, derselbe für alle Fotos…** | Geschrieben | Während des Tauchgangs¹ | Geschrieben | Bleibt² | Der andere Tauchplatz | Der andere Tauchplatz⁴ | Die seines Tauchgangs⁵ |
| Nur **Sammlungen erstellen** | Bleibt | Bleibt | Bleibt | Bleibt | Bleibt | Bleibt | Die seines Tauchgangs |

**Lightroom-Auswahl**

| In Schritt 4 angekreuzt | Tauchgangsfelder | Tiefe, Temperatur, Zeit | Zeitversatz | GPS-Höhe | Land, Stadt, Ortsdetail | GPS | Sammlung |
|---|---|---|---|---|---|---|---|
| Alles (der Standard) | Geschrieben | Während des Tauchgangs¹ | Während des Tauchgangs¹ | Bleibt² | Tauchplatz des Tauchgangs | Tauchplatz des Tauchgangs | Die des Tauchgangs |
| Alles, und **Tiefe in die GPS-Höhe kopieren** | Geschrieben | Während des Tauchgangs¹ | Während des Tauchgangs¹ | Die Tiefe³ | Tauchplatz des Tauchgangs | Tauchplatz des Tauchgangs | Die des Tauchgangs |
| Nur **Tauchgangsdetails** | Geschrieben | Während des Tauchgangs¹ | Während des Tauchgangs¹ | Bleibt² | Bleibt | Bleibt | Keine |
| Nur **Tauchgangsdetails** und **Tiefe in die GPS-Höhe kopieren** | Geschrieben | Während des Tauchgangs¹ | Während des Tauchgangs¹ | Die Tiefe³ | Bleibt | Bleibt | Keine |
| Nur **Tauchplatzdetails** (**Ort** und **GPS**) | Bleibt | Bleibt | Bleibt | Bleibt | Tauchplatz des Tauchgangs | Tauchplatz des Tauchgangs | Keine |
| Nur **Ort** | Bleibt | Bleibt | Bleibt | Bleibt | Tauchplatz des Tauchgangs | Bleibt | Keine |
| Nur **GPS** | Bleibt | Bleibt | Bleibt | Bleibt | Bleibt | Tauchplatz des Tauchgangs | Keine |
| Alles, mit **Anderer Tauchplatz…** | Geschrieben | Während des Tauchgangs¹ | Während des Tauchgangs¹ | Bleibt² | Der andere Tauchplatz | Der andere Tauchplatz⁴ | Die des Tauchgangs⁵ |
| Nur **Sammlungen erstellen** | Bleibt | Bleibt | Bleibt | Bleibt | Bleibt | Bleibt | Die des Tauchgangs |
| **Kein Tauchgang — nur Ort**, mit **Ort** und **GPS** | Bleibt | Bleibt | Bleibt | Bleibt | Der andere Tauchplatz | Der andere Tauchplatz⁴ | Keine |
| **Kein Tauchgang — nur Ort**, nur mit **Ort** | Bleibt | Bleibt | Bleibt | Bleibt | Der andere Tauchplatz | Bleibt | Keine |
| **Kein Tauchgang — nur Ort**, nur mit **GPS** | Bleibt | Bleibt | Bleibt | Bleibt | Bleibt | Der andere Tauchplatz⁴ | Keine |

1. Auf den Fotos, die während eines Tauchgangs mit Profil aufgenommen wurden: die Tiefe, die Temperatur und die Tauchzeit im Moment des Fotos.
   - **Gesamter Katalog**: Bei den anderen Fotos, die in einen Tauchgang gelegt wurden (in der Oberflächenmarge oder in einem Tauchgang ohne Profil), werden diese drei Werte geleert. Jedes in einen Tauchgang gelegte Foto erhält den Zeitversatz.
   - **Lightroom-Auswahl**: Der Zeitversatz kommt nur auf die während des Tauchgangs aufgenommenen Fotos, zusammen mit ihrer Tiefe. Die anderen Fotos behalten ihre Werte, wenn sie bereits denselben Tauchgang tragen; trugen sie einen anderen Tauchgang, werden dessen Tiefe, Temperatur, Zeit und Zeitversatz geleert.
   - Ein Tauchgang, dessen Profil nicht gelesen werden kann: Auf dem Weg **Gesamter Katalog** hält er **Anwenden** auf, was auch immer angekreuzt ist, wenn Fotos während dieses Tauchgangs aufgenommen wurden: „Das Logbuch konnte nicht gelesen werden: … Es wurde nichts geschrieben.“ Auf dem Weg **Lightroom-Auswahl** erhalten seine Fotos die Tauchgangsdaten ohne Daten pro Foto.
2. Bleibt, außer einer Höhe, die Dive Lens selbst kopiert hat: Sie wird gelöscht, wenn das Foto einem anderen Tauchgang ohne neue Höhe zugeordnet wird und, auf dem Weg **Gesamter Katalog**, wenn das Foto ohne Tiefe wiedergefunden wird ([Abschnitt 6](#6-tiefe-in-der-gps-höhe)).
3. Auf den Fotos, die eine Tiefe erhalten: diese Tiefe, als negative GPS-Höhe in Metern. Die anderen Fotos wie in Anmerkung 2.
4. Das GPS des gewählten Tauchplatzes oder die Position, die Sie darunter eingetippt haben. Leeren Sie beide Koordinaten, wird die eigene Position des Tauchplatzes geschrieben: Um das GPS der Fotos zu behalten, kreuzen Sie **GPS** nicht an.
5. Die Sammlung behält den Namen des eigenen Tauchplatzes des Tauchgangs.

### Wenn Anwenden ausgegraut bleibt

| Situation | Was die Leiste meldet |
|---|---|
| Lightroom nicht verbunden | **Gesamter Katalog**: „Lightroom nicht verbunden: Fotos können weder gesucht noch beschrieben werden.“; **Lightroom-Auswahl**: „Lightroom nicht verbunden: Es kann nichts geschrieben werden“ |
| **Tauchgangsdetails**, **Tauchplatzdetails** und **Sammlungen erstellen** alle nicht angekreuzt; mit **Kein Tauchgang — nur Ort**, **Ort** und **GPS** beide nicht angekreuzt | „Kreuzen Sie mindestens eine Option an“ |
| **Anderer Tauchplatz** gewählt oder vorgegeben (**Kein Tauchgang — nur Ort** oder ein gewählter Tauchgang ohne Tauchplatz) und noch kein Tauchplatz gewählt, während **Ort** oder **GPS** angekreuzt ist | „Wählen Sie einen Tauchplatz“ |
| Die unter **Anderer Tauchplatz** eingetippte Position ist ungültig, während **GPS** angekreuzt ist | „Breitengrad zwischen -90 und 90, Längengrad zwischen -180 und 180, oder beide leer.“ |
| Die angekreuzten Optionen lassen auf keinem Foto etwas zu schreiben: zum Beispiel **GPS** allein bei einem Tauchplatz ohne GPS oder **Ort** allein bei einem Tauchplatz ohne Namen, Land oder Stadt | „Mit diesen Optionen ist nichts zu schreiben.“ |
| **Gesamter Katalog**: Kein Foto fällt in die angekreuzten Tauchgänge | „Kein Foto fällt in die Tauchgänge: Passen Sie die Zeitversätze an.“ |
| **Lightroom-Auswahl**: kein Foto ausgewählt | „Wählen Sie Fotos in Lightroom aus“ |
| **Sammlungen erstellen** angekreuzt, und der in den Einstellungen gewählte Sammlungssatz ist nicht mehr im Katalog | „Der in den Einstellungen gewählte Sammlungssatz (<Ort>) ist nicht mehr im Katalog. Wählen Sie in den Einstellungen einen anderen Ort.“, mit **Einstellungen öffnen** |

Auf dem Weg **Gesamter Katalog** wird **GPS** allein (oder **Ort** allein) nicht abgelehnt, wenn nur einige Tauchplätze der Tauchgänge es liefern: Nur die Fotos dieser Tauchgänge erhalten es. Die Wirkung rechts neben **Tauchplatzdetails** zählt die Fotos, die wirklich etwas erhalten, während **Auf N Fotos anwenden** weiterhin jedes gefundene Foto zählt.

### Anwendungsfälle

**Eine ganze Reise taggen, zum ersten Mal.** **Gesamter Katalog**; kreuzen Sie die Tauchgänge der Reise an; in Schritt 3 **Für alle vorschlagen**, dann die Karte jeder Kamera und die Vorschau pro Tauchgang prüfen; die Optionen lassen, wie sie sind; **Anwenden**. Jedes während eines Tauchgangs aufgenommene Foto erhält die Tauchgangsdaten, Ort und GPS seines Tauchplatzes, den Zeitversatz und, wenn der Tauchgang ein Profil hat, seine Tiefe, Temperatur und Tauchzeit. Jeder Tauchgang mit Fotos erhält seine Sammlung, in der Regel „<Nummer> - <Tauchplatz>“ genannt (Abschnitt 5). Die Fotos außerhalb der Tauchgänge erhalten nichts: siehe Boot und Strand weiter unten.

**Die Sammlungen später hinzufügen, ohne die Daten der Fotos anzutasten.** **Gesamter Katalog**, dieselben Tauchgänge; in Schritt 3 beginnt jede Karte mit dem zuletzt auf diese Kamera angewendeten Zeitversatz, den eine spätere Reise geändert haben kann: Stellen Sie die Zeitversätze dieser Reise wieder ein (das Feld Zeitversatz eines Fotos zeigt den, den es erhalten hat) und dieselbe Marge **An der Oberfläche aufgenommene Fotos einbeziehen** wie beim ersten Mal; in Schritt 4 entfernen Sie die Kreuze bei **Tauchgangsdetails** und **Tauchplatzdetails** und behalten **Sammlungen erstellen**. Die Leiste zeigt „N Sammlungen · auf die Fotos wird nichts geschrieben“. Die Fotos jedes Tauchgangs kommen in seine Sammlung; ein Foto, das schon darin ist, wird nicht doppelt hinzugefügt. Umgekehrt geht es auch: Entfernen Sie nur das Kreuz bei **Sammlungen erstellen**, um die Daten ohne Sammlung zu schreiben.

**Fotos, die zwischen den Tauchgängen auf dem Boot oder am Strand aufgenommen wurden.** Auf dem Weg **Gesamter Katalog** fallen sie außerhalb der Tauchgänge und erhalten nichts. Wählen Sie sie in Lightroom aus, dann **Lightroom-Auswahl** und **Kein Tauchgang — nur Ort**; in Schritt 4 wählen Sie den Tauchplatz über Land › Ort › Tauchplatz und tippen, wenn der Strand nicht dort liegt, wo der Tauchplatz ist, seine Position darunter ein; **Anwenden**. Sie erhalten nur Land, Stadt, Ortsdetail und GPS: kein Tauchgangsfeld und keine Sammlung, und die Tauchgangsdaten, die einige davon vielleicht schon tragen, bleiben, wie sie sind. Um ihnen stattdessen die Daten eines Tauchgangs zu geben, wählen Sie diesen Tauchgang: Sie erhalten seine Daten, seinen Tauchplatz und seine Sammlung, ohne Tiefe ([Abschnitt 7](#7-nur-ort-und-fotos-außerhalb-der-tauchgänge)).

**Ein Tauchgang ohne Profil.** Ein von Hand eingetragener Tauchgang, oder einer, dessen Computer-Download die Tauchanwendung nicht aufbewahrt hat, braucht nichts Besonderes. Seine Fotos erhalten die Tauchgangsdaten, den Tauchplatz und die Sammlung, ohne Tiefe, Temperatur oder Zeit, **Tiefe in die GPS-Höhe kopieren** hat für sie also nichts zu kopieren. Auf dem Weg **Gesamter Katalog** entscheidet der Zeitversatz weiterhin, welche Fotos zum Tauchgang gehören, und wird auf sie geschrieben; die Vorschau pro Tauchgang zeigt „Kein Profil: nur Tauchgangsdaten“. Auf dem Weg **Lightroom-Auswahl** zeigt Schritt 3 „Kein Profil: keine Daten pro Foto“, und es wird kein Zeitversatz geschrieben.

**Den Tauchplatz bereits getaggter Fotos korrigieren.** Wählen Sie die Fotos in Lightroom aus, wählen Sie **Lightroom-Auswahl** und ihren Tauchgang; in Schritt 4 entfernen Sie die Kreuze bei **Tauchgangsdetails** und **Sammlungen erstellen**, wählen **Anderer Tauchplatz…** und den richtigen Tauchplatz; **Anwenden**. Nur Land, Stadt, Ortsdetail und GPS ändern sich: Die Tauchgangsdaten und die Sammlungen bleiben. Ein Wert, den der neue Tauchplatz nicht hat, wird nicht gelöscht: Hatte der alte Tauchplatz eine Stadt und der neue keine, bleibt die alte Stadt; löschen Sie sie in Lightroom. Für die Fotos mehrerer Tauchgänge am selben Tauchplatz tun Sie dasselbe auf dem Weg **Gesamter Katalog** mit **Anderer Tauchplatz, derselbe für alle Fotos…**, mit den Zeitversätzen und der Oberflächenmarge des ersten Durchlaufs: Nur die in den Tauchgängen gefundenen Fotos erhalten den neuen Tauchplatz. Ist der Tauchplatz in Ihrem Logbuch falsch, korrigieren Sie ihn stattdessen dort und führen Sie den Tauchgang erneut durch: Die Fotos erhalten den neuen Tauchplatz, und wenn **Sammlungen erstellen** angekreuzt ist, wird neben der alten eine Sammlung mit dem Namen des neuen Tauchplatzes erstellt.

**Eine GPS-Position von der Kamera oder von Hand gesetzt behalten.** Entfernen Sie das Kreuz bei **GPS** unter **Tauchplatzdetails**: Land, Stadt und Ortsdetail werden geschrieben, die Position bleibt. Auch wenn **GPS** angekreuzt ist, lässt ein Tauchplatz ohne Position die eigene des Fotos. Wurde eine Position versehentlich ersetzt, bringt **Bearbeiten › Rückgängig: Dive Lens** in Lightroom sie zurück.

**Die Tiefe als GPS-Höhe, für Karten und andere Software.** Kreuzen Sie **Tiefe in die GPS-Höhe kopieren** unter **Tauchgangsdetails** an: Jedes Foto, das eine Tiefe erhält, erhält sie auch als negative GPS-Höhe. Die Option ist bei jedem Durchlauf nicht angekreuzt, damit Sie jedes Mal entscheiden, in Kenntnis von vier Dingen: Das Dive Lens-Feld **Tiefe** bleibt in Lightroom und geht beim Export verloren; die GPS-Höhe reist mit exportierten Fotos mit und erscheint in anderer Software; sie ist immer in Metern, unabhängig von Ihren Einheiten; und sie ersetzt eine Höhe, die das Foto schon hat (**Bearbeiten › Rückgängig** bringt sie zurück). Details in [Abschnitt 6](#6-tiefe-in-der-gps-höhe).

**Dive Lens erneut auf bereits getaggte Fotos anwenden.** **Gesamter Katalog**: Tauchgänge erneut durchzuführen fügt nur hinzu. Nichts wird verdoppelt, die wiedergefundenen Fotos erhalten ihre Werte mit den angezeigten Zeitversätzen erneut geschrieben, und ein Foto, das nun in einen anderen Tauchgang fällt, erhält dessen Daten und Sammlung, bleibt aber auch in seiner alten Sammlung: Entfernen Sie es dort von Hand. **Lightroom-Auswahl**: Mit dem Tauchgang, den die Fotos bereits tragen, werden die Tauchgangsdaten erneut geschrieben, ebenso Tiefe, Temperatur, Zeit und Zeitversatz der während des Tauchgangs aufgenommenen Fotos; die anderen Fotos behalten ihre. Mit einem anderen Tauchgang ersetzen dessen Daten die des alten, und die Fotos bleiben in der Sammlung des alten Tauchgangs.

**Fotos von zwei Kameras mit unterschiedlichen Uhren.** **Gesamter Katalog** behandelt sie in einem Durchlauf: Jede Kamera hat in Schritt 3 ihre eigene Karte und ihren eigenen Zeitversatz; **Für alle vorschlagen** stellt alle auf einmal ein, ohne eine Karte anzutasten, die Sie von Hand eingestellt haben; zwei Gehäuse desselben Modells werden an ihrer Seriennummer unterschieden. **Ignorieren** Sie eine Kamera, deren Fotos nichts mit den Tauchgängen zu tun haben, etwa ein Telefon, das auf dem Boot geblieben ist. Nach **Anwenden** wird der Zeitversatz jeder Kamera für das nächste Mal gemerkt (**Einstellungen › Gemerkte Zeitversätze**). Eine mitten in der Reise umgestellte Uhr ist etwas anderes: Führen Sie die Tage vor und nach der Umstellung getrennt durch, jeweils mit eigenem Zeitversatz ([Abschnitt 11](#11-problembehebung)).

**Fotos außerhalb der Tauchgangszeit, auf dem Weg Lightroom-Auswahl.** Sie werden dem gewählten Tauchgang trotzdem zugeordnet: Sie erhalten seine Daten, seinen Tauchplatz und seine Sammlung, aber keine Tiefe, Temperatur, Zeit und keinen Zeitversatz. Schritt 3 listet sie unter **Außerhalb der Tauchgangszeit** auf, Schritt 4 unter **Gut zu wissen vor dem Anwenden**, und die Zusammenfassung zählt sie als Fotos ohne Tiefe. Nicht für diesen Tauchgang gedacht? Entfernen Sie sie vor **Anwenden** in Lightroom aus der Auswahl. Auf dem Weg **Gesamter Katalog** tut **An der Oberfläche aufgenommene Fotos einbeziehen** dasselbe für Fotos, die kurz vor oder kurz nach einem Tauchgang aufgenommen wurden: Sie erhalten die Tauchgangsdaten, den Tauchplatz, den Zeitversatz und die Sammlung, ohne Tiefe.

**Rückgängig.** Unabhängig von den Optionen ist ein **Anwenden** ein Schritt in Lightroom: **Bearbeiten › Rückgängig: Dive Lens** nimmt alles zurück, was es geschrieben hat, GPS und Höhe inbegriffen ([Abschnitt 8](#8-rückgängig)).

---

## 5. Was geschrieben wird

- **Sammlungen erstellen**: eine Sammlung pro Tauchgang, **„<Tauchgangsnummer> - <Tauchplatz>“** genannt, im Sammlungssatz **Dive Lens** (der Ort lässt sich in den Einstellungen ändern). Ein Tauchgang ohne Tauchplatz wird nur mit seiner Nummer benannt; ein Tauchgang ohne Nummer erhält statt der Nummer sein Startdatum und seine Startzeit.
- **Tauchgangsdetails**: auf jedem Foto Tauchgangsnummer, Beginn und Ende, Dauer, max. Tiefe, min. und max. Temperatur, Bewertung und die Tauchanwendung, aus der die Daten stammen. Auf den Fotos, die während eines Tauchgangs mit Profil aufgenommen wurden, auch **Tiefe, Temperatur und Tauchzeit** des Fotos, geschätzt zwischen den beiden Profilpunkten um den Moment des Fotos. Und der angewendete Zeitversatz: auf jedem in einen Tauchgang gelegten Foto (**Gesamter Katalog**), auf den während des Tauchgangs aufgenommenen Fotos, wenn er ein Profil hat (**Lightroom-Auswahl**).
- **Tauchplatzdetails**: Land, Stadt, Ortsdetail und GPS, in den Standardfeldern von Lightroom.

Gut zu wissen, auf beiden Wegen:

- Die Uhrzeit des Fotos ist seine **ursprüngliche Aufnahmezeit**, ein aus Photoshop zurückgekommenes TIFF wird also wie sein Original zugeordnet.
- Ein Tauchplatz ohne Land, Stadt oder GPS lässt die Werte, die das Foto schon hat, unangetastet (eine GPS-Position von einem Telefon bleibt).
- Ist **Tauchgangsdetails** nicht angekreuzt, tastet Dive Lens die Tauchgangsfelder gar nicht an; die Fotos werden weiterhin den Tauchgängen zugeordnet, für den Tauchplatz und die Sammlungen.
- Ein Wert, den der Tauchgang nicht hat (Temperatur, Bewertung…), löscht den alten Dive Lens-Wert auf dem Foto, die Felder beschreiben also immer einen einzigen Tauchgang.
- Die in den Einstellungen gewählten Einheiten gelten für die geschriebenen Werte. Bereits getaggte Fotos behalten die Einheiten, die beim Schreiben galten.

Auf dem Weg **Gesamter Katalog**:

- Nur die in einen Tauchgang gelegten Fotos werden geschrieben, im Tauchgang oder in der Oberflächenmarge. Ignorierte Kameras und Fotos außerhalb der Tauchgänge werden nicht angetastet.
- Ein Foto in der Oberflächenmarge erhält die Tauchgangsdaten, den Ort und den Zeitversatz, ohne Daten pro Foto.
- Ein Foto, das in zwei Tauchgänge fallen würde, kommt zum näheren.
- Eine Gruppe von Tauchgängen erneut durchzuführen fügt nur hinzu: Bereits abgelegte Fotos bleiben, wo sie sind (korrigieren Sie sie bei Bedarf von Hand), und wiedergefundene Fotos erhalten ihre Werte neu geschrieben.
- Ein Tauchgang, dessen Profil nicht gelesen werden kann, hält **Anwenden** auf, wenn Fotos während dieses Tauchgangs aufgenommen wurden: Es wird nichts geschrieben.

Auf dem Weg **Lightroom-Auswahl**:

- Jedes im Moment des Klicks ausgewählte Foto wird geschrieben.
- Ein außerhalb der Tauchgangszeit aufgenommenes Foto erhält die Tauchgangsdaten und den Ort, ohne Daten pro Foto und ohne Zeitversatz.
- Mit **Tauchgangsdetails** verliert ein Foto, das bereits einen **anderen** Tauchgang trägt, dessen eigene Werte: Tiefe, Temperatur, Tauchzeit und Zeitversatz, ersetzt durch die neuen, wenn das Foto während des neuen Tauchgangs aufgenommen wurde. Seine GPS-Höhe folgt Abschnitt 6. Das Foto bleibt in der Sammlung seines alten Tauchgangs: Verschieben Sie es bei Bedarf selbst.
- Ein Foto mit dem Tauchgang zu taggen, den es bereits trägt, schreibt seine Daten pro Foto neu, wenn es während des Tauchgangs aufgenommen wurde, mit dem angezeigten Zeitversatz; die anderen Fotos behalten ihre.
- Ändert sich die Auswahl in Lightroom genau beim Klick auf **Anwenden**, während Daten pro Foto geschrieben werden, wird nichts geschrieben: „Die Auswahl in Lightroom hat sich geändert. Es wurde nichts geschrieben: Prüfen Sie die Daten pro Foto und wenden Sie dann erneut an.“
- Ein Profil, das nicht gelesen werden kann, blockiert nie: Die Tauchgangsdaten werden ohne Daten pro Foto geschrieben.

Nach **Anwenden** merkt sich Dive Lens den Zeitversatz jeder Kamera für das nächste Mal, auf beiden Wegen: auf dem Weg **Gesamter Katalog** die Kameras mit einem in einen Tauchgang gelegten Foto; auf dem Weg **Lightroom-Auswahl** die Kameras mit einem während des Tauchgangs aufgenommenen Foto, wenn Daten pro Foto geschrieben werden.

---

## 6. Tiefe in der GPS-Höhe

Die Option **Tiefe in die GPS-Höhe kopieren** steht in Schritt 4 unter **Tauchgangsdetails**. Sie ist bei jedem Durchlauf nicht angekreuzt: Kreuzen Sie sie jedes Mal an, wenn Sie sie wünschen. Vier Dinge sind zu wissen:

- Das Dive Lens-Feld **Tiefe** gibt es nur in Lightroom: Es wird nicht in die Fotodatei geschrieben und geht beim Export verloren. Dive Lens füllt es immer aus.
- Die GPS-Höhe reist mit exportierten Fotos mit und erscheint in anderer Software. Ist die Option angekreuzt, schreibt Dive Lens die Tiefe jedes Fotos auch dorthin, als negative Höhe.
- Dive Lens schreibt die GPS-Höhe immer in Metern, dem Standard des Feldes, unabhängig von den in den Einstellungen gewählten Einheiten. Bei Tiefen in Fuß zeigt dasselbe Foto 60 ft im Dive Lens-Feld **Tiefe**, während seine GPS-Höhe in Metern ist (18,3 m unter dem Meeresspiegel): Das ist so vorgesehen.
- Die Kopie ersetzt eine GPS-Höhe, die das Foto schon hat; **Bearbeiten › Rückgängig** in Lightroom bringt sie zurück. Fotos ohne Tiefe behalten ihre.

Und die Regeln, die daraus folgen:

- Nur die Fotos, die eine Tiefe erhalten, erhalten eine Höhe: nicht die in der Oberflächenmarge, außerhalb der Tauchgangszeit oder in einem Tauchgang ohne Profil.
- Die Option bei einem späteren Durchlauf nicht anzukreuzen löscht die bereits kopierten Höhen nicht, mit zwei Ausnahmen:
  - auf dem Weg **Gesamter Katalog** verliert ein Foto, das ohne Tiefe wiedergefunden wird (nun in der Oberflächenmarge oder in einem Tauchgang ohne Profil), die Höhe, die Dive Lens dorthin kopiert hat;
  - ein Foto, das einem anderen Tauchgang ohne neue Höhe zugeordnet wird, verliert die Höhe, die Dive Lens aus seiner alten Tiefe kopiert hat.
- Dive Lens löscht nur eine Höhe, die es selbst geschrieben hat: Eine Höhe von der Kamera oder eine, die Sie selbst gesetzt oder geändert haben, bleibt.
- Ist **Tauchgangsdetails** nicht angekreuzt, wird keine Höhe geschrieben oder gelöscht.

---

## 7. Nur Ort, und Fotos außerhalb der Tauchgänge

Auf dem Weg **Gesamter Katalog** erhalten Fotos, die außerhalb jedes Tauchgangs aufgenommen wurden (auf dem Boot, am Strand, beim Schnorcheln), nichts: Schritt 3 zeigt sie unter **Außerhalb der Tauchgänge**, ebenso die Zusammenfassung und **Gut zu wissen vor dem Anwenden**.

Um ihnen einen Ort zu geben oder sie trotzdem einem Tauchgang zuzuordnen:

1. Wählen Sie sie in Lightroom aus, dann **Bibliothek › Plug-in-Extras › Dive Lens öffnen…**, und wählen Sie die Karte **Lightroom-Auswahl**.
2. In Schritt 2:
   - um sie trotzdem einem Tauchgang zuzuordnen, wählen Sie den Tauchgang: Sie erhalten seine Daten und seinen Ort, ohne Tiefe;
   - um ihnen nur einen Ort zu geben, wählen Sie **Kein Tauchgang — nur Ort**.

Mit **Kein Tauchgang — nur Ort** wird Schritt 3 übersprungen. In Schritt 4 wirkt nur **Tauchplatzdetails**: **Tauchgangsdetails** und **Sammlungen erstellen** sind ausgegraut, und der Tauchplatz ist **Anderer Tauchplatz…**. Wählen Sie den Tauchplatz über Land › Ort › Tauchplatz, passen Sie bei Bedarf sein GPS an, dann **Auf N Fotos anwenden**. Nur Land, Stadt, Ortsdetail und GPS werden geschrieben: kein Tauchgangsfeld und keine Sammlung.

---

## 8. Rückgängig

**Bearbeiten › Rückgängig** in Lightroom nimmt alles zurück, was ein **Anwenden** geschrieben hat, in einem Schritt, GPS und Höhe inbegriffen, auf beiden Wegen. Im Menü Bearbeiten von Lightroom heißt es **Rückgängig: Dive Lens**.

- Eine von **Anwenden** erstellte Sammlung verschwindet damit; Fotos, die einer vorhandenen Sammlung hinzugefügt wurden, verlassen sie.
- **Beim ersten Mal**, wenn der Sammlungssatz (standardmäßig **Dive Lens**) noch nicht existiert, erstellt Dive Lens ihn kurz vor dem Schreiben. Ist er nach dem Rückgängig noch da, leer, entfernt ihn ein zweites **Rückgängig: Dive Lens**, oder löschen Sie ihn in Lightroom. Wird das Schreiben abgelehnt, nachdem der Satz erstellt wurde, bleibt der leere Satz: Machen Sie es rückgängig oder löschen Sie ihn in Lightroom.
- Hat Lightroom Dive Lens das GPS und die Höhe nicht schreiben lassen, ist alles andere geschrieben, und Dive Lens meldet es: Wählen Sie zuerst **Bearbeiten › Rückgängig: Dive Lens** in Lightroom und wenden Sie dann erneut an.

---

## 9. Einstellungen

Das Zahnrad oben öffnet die Einstellungen, ebenso **Hilfe › Plug-in-Extras › Dive Lens-Einstellungen…** in Lightroom. Sie nehmen das ganze Fenster ein. **Zurück** kehrt zum Ablauf zurück, zu dem Schritt, bei dem Sie waren, mit Ihrer Auswahl; hat sich das Logbuch inzwischen geändert, beginnt der Ablauf wieder bei Schritt 1. Das Zahnrad ist ausgegraut, während Dive Lens in Lightroom schreibt.

Jede Änderung wird sofort gespeichert.

**Logbuch**
- **Anwendung**: MacDive oder Subsurface. Nur die auf Ihrem System verfügbaren Anwendungen werden angeboten: unter Windows Subsurface. Die Anwendung wird nur hier geändert, nie aus dem Ablauf heraus.
- **MacDive-Datenbank** / **Subsurface-Logbuch**: Der Standardspeicherort wird verwendet, sofern Sie keinen anderen wählen. Ein Subsurface-Logbuch ist entweder eine Datei (.xml oder .ssrf) oder der Ordner seines Git-Speichers (der Cloud-Speicher hält einen auf Ihrem Computer). Unter dem Speicherort zeigt Dive Lens, ob das Logbuch gültig ist und wie viele Tauchgänge und Tauchplätze es enthält.
- Jede Anwendung behält ihren eigenen Speicherort: Wechseln Sie von der einen zur anderen und zurück, ist Ihrer wieder da.
- **Ein Logbuch auf einmal.** Die Anwendung oder das Logbuch zu ändern beginnt den Ablauf wieder bei Schritt 1, ohne jede Wahl: kein Tauchgang angekreuzt oder gewählt, Uhren und Optionen vergessen. Das gilt, wie auch immer das Logbuch wechselt: hier, durch den Import von Einstellungen, über den Bildschirm, der erscheint, wenn das Logbuch nicht gelesen werden kann, oder wenn Dive Lens beim Verbinden mit Lightroom die Einstellungen des früheren Zusatzmoduls übernimmt (Abschnitt 1).
- Während Dive Lens in Lightroom schreibt, kann das Logbuch nicht wechseln: **Anwendung**, **Wählen…** (oder **Datei wählen…** / **Ordner wählen…**), **Standardspeicherort**, **Importieren…** und **Ersetzen** sind ausgegraut, mit dem Hinweis „Es wird in Lightroom geschrieben: Das Logbuch kann geändert werden, sobald das Schreiben beendet ist.“

**Anzeige**
- **Sprache**: standardmäßig die des Systems. Die Menüs und Felder in Lightroom folgen der Sprache von Lightroom.
- **Tiefe** und **Temperatur**: **Wie MacDive** / **Wie Subsurface** (die in dieser Anwendung gewählten Einheiten) oder fest auf Meter / Fuß und °C / °F. Sie gelten in der App und für die auf die Fotos geschriebenen Werte. Die GPS-Höhe ist immer in Metern (Abschnitt 6).

**Sammlungen**
- **Ablegen in**: die oberste Ebene des Katalogs oder ein vorhandener Sammlungssatz (Lightroom muss verbunden sein, um sie zu durchsuchen). Dort legt die Option **Sammlungen erstellen** von Schritt 4 die Sammlungen ab.
- **Im Satz gruppieren**: standardmäßig „Dive Lens“, bei Bedarf erstellt; lassen Sie es leer, um die Sammlungen direkt am Ort oben abzulegen. Eine Vorschau zeigt den sich ergebenden Pfad. Bereits erstellte Sammlungen bleiben, wo sie sind.
- Ist der in **Ablegen in** gewählte Satz nicht mehr im Katalog, meldet Schritt 4 es, solange **Sammlungen erstellen** angekreuzt ist, und **Anwenden** bleibt ausgegraut, bis Sie hier einen anderen Ort wählen.

**Gemerkte Zeitversätze**
- Unter dem Titel: „Nach „Anwenden“ für jede Kamera aufbewahrt und im Schritt Uhren erneut angeboten, für <Anwendung>.“ Die Liste zeigt die Kameras, deren Zeitversatz für die oben gewählte Tauchanwendung gemerkt ist, nach Namen sortiert wie in Schritt 3. Jede Zeile gibt den Namen der Kamera wie auf ihrer Karte in Schritt 3 an (ihr Modell oder „Unbekannte Kamera“), darunter „Nr. <Seriennummer>“, wenn die Fotos eine liefern; rechts ihren Zeitversatz wie auf der Karte (zum Beispiel „+4:00“, „0:00“ oder „−0:30“) und **Vergessen**. Gibt es keinen: „Kein Zeitversatz gemerkt.“
- **Vergessen** entfernt den Zeitversatz dieser Kamera sofort, ohne Rückfrage, und ihre Zeile verschwindet. Beim nächsten Mal beginnt ihre Karte in Schritt 3 bei 0:00, ohne **gemerkter Zeitversatz**; **Vorschlagen** findet den Zeitversatz wieder. Die anderen Kameras und die für die andere Tauchanwendung aufbewahrten Zeitversätze bleiben.
- In Lightroom ändert sich nichts: Der bereits auf Fotos geschriebene Zeitversatz bleibt.
- Während eines Ablaufs vergessen, auf beiden Wegen: Zurück aus den Einstellungen wird die Kamera nicht mehr als gemerkt angezeigt. Trug ihre Karte in Schritt 3 noch das Kennzeichen **gemerkter Zeitversatz**, geht sie auf 0:00 zurück, wie eine neue Suche sie zeigen würde; ein von Hand eingestellter Zeitversatz bleibt, wie er ist, ebenso ein mit **Vorschlagen** gefundener, es sei denn, er entspricht dem gemerkten Zeitversatz (die Karte zeigt dann weiterhin das Kennzeichen). Das Kennzeichen verschwindet, ebenso „mit dem gemerkten Zeitversatz von …“ unter **Deckt die Auswahl ab** in Schritt 2. Die Zahlen und **Deckt die Auswahl ab** folgen dem Zeitversatz, den die Karte nun zeigt, und **Anwenden** schreibt und merkt sich nur, was die Karten zeigen.

**Sicherung der Einstellungen**
- **Exportieren oder importieren**: um Ihre Einstellungen auf einem anderen Computer oder nach einer Neuinstallation wiederzubekommen.
- **Exportieren…** speichert sie in einer Datei, „Dive Lens Einstellungen.json“ genannt, sofern Sie keinen anderen Namen wählen. Die Datei enthält: die Tauchanwendung und den Speicherort des Logbuchs jeder Anwendung, Sprache, Einheiten, den Ablageort der Sammlungen, die automatische Suche nach Updates, die Protokollstufe und die Reihenfolge der Tauchgangsliste. Die für jede Kamera gemerkten Zeitversätze sind nicht darin: Sie bleiben auf diesem Computer.
- **Importieren…** fragt, bevor etwas geändert wird: **Ersetzen** setzt die Einstellungen der Datei an die Stelle aller Einstellungen dieses Bildschirms, **Abbrechen** lässt sie, wie sie sind.
- Ist das Logbuch der Datei nicht auf diesem Computer, meldet Dive Lens es und behält seinen Speicherort, bis Sie den richtigen wählen (**Wählen…**, oder **Datei wählen…** / **Ordner wählen…** für Subsurface). Ist der in **Ablegen in** gewählte Sammlungssatz nicht in diesem Katalog, meldet Schritt 4 es, solange **Sammlungen erstellen** angekreuzt ist: Wählen Sie in den Einstellungen einen anderen Ort.
- Eine Datei von einem Mac, der MacDive verwendet, unter Windows importiert: Dive Lens verwendet Subsurface, die dort verfügbare Anwendung, und behält den MacDive-Speicherort: Diese Einstellungen, erneut exportiert und auf einem Mac importiert, verwenden dort weiterhin MacDive.
- Eine Datei, die nicht wie eine Dive Lens-Einstellungsdatei aussieht, wird abgelehnt, und nichts ändert sich. Eine von einer früheren Version von Dive Lens exportierte Datei lässt sich weiterhin importieren; ihre Einstellung „Tiefe in die GPS-Höhe kopieren“ wird ignoriert, da die Option jetzt in Schritt 4 steht.
- Nach einem Import übernimmt Dive Lens die Einstellungen des früheren Zusatzmoduls nicht mehr (Abschnitt 1): Die importierten Einstellungen sind die, die gelten.

**Updates**
- **Automatisch suchen** (standardmäßig an): Beim Start von Dive Lens, höchstens einmal am Tag, sucht es auf GitHub nach einer neuen Version. Nichts wird ohne Ihre Zustimmung installiert.
- **Jetzt suchen** sucht sofort. Unter **Dive Lens <Version>** steht, ob Ihre Version aktuell ist und wann zuletzt gesucht wurde.
- Gibt es eine neue Version, meldet es ein Banner oben im Bildschirm: **Ansehen** öffnet diesen Abschnitt, **×** schließt das Banner bis zur nächsten Version. Hier öffnet **Neuerungen** die Seite der Version auf GitHub, und **Installieren** installiert sie.
- **Installieren** lädt die neue Version herunter, prüft, dass sie von Dive Lens stammt (Signatur), ersetzt die App und den Lightroom-Teil, dann startet Dive Lens neu. Nicht, während in Lightroom geschrieben wird.
- Danach **Lightroom neu starten**: Es lädt den neuen Dive Lens-Teil nur beim Start. Bis dahin zeigt Dive Lens „Starten Sie Lightroom neu, um das Update von Dive Lens abzuschließen.“ und die Statusleiste **Lightroom neu starten**.
- Unter Windows erscheint eine mit der Zip-Datei installierte Kopie danach in **Einstellungen › Apps › Installierte Apps**, dort, wo Sie sie abgelegt hatten.
- Kann Dive Lens seinen Ordner nicht ändern, bietet es **Von GitHub herunterladen** an: Installieren Sie die neue Version dann selbst (Abschnitt 1).

**Diagnose**
- **Protokollstufe**: Normal oder Ausführlich. Erhöhen Sie sie nur, um ein Problem zu melden.
- **Protokoll anzeigen**: öffnet den Ordner mit den beiden Protokolldateien, der der App (`DiveLens App.log`) und der des Lightroom-Zusatzmoduls (`DiveLens.log`).

---

## 10. Die Dive Lens-Felder in Lightroom

Öffnen Sie im Modul **Bibliothek** das Bedienfeld **Metadaten** und wählen Sie in seinem Menü **Dive Lens**. Die Felder stehen in vier Gruppen:

- **Foto**: Tiefe, Temperatur, Verstrichene Zeit;
- **Tauchgang**: Tauchgang, Tauchgangsbeginn, Tauchgangsende, Dauer, Max. Tiefe, Min. Temp., Max. Temp., Bewertung;
- **Tauchplatz**: Ortsdetail, Stadt, Land und GPS von Lightroom, mit der GPS-Höhe unter dem GPS;
- **Technik**: Zeitversatz, Quelle.

Die Dive Lens-Felder bleiben im Lightroom-Katalog: Sie werden nicht in die Fotodateien geschrieben und reisen nicht mit Exporten mit. Das tun nur die GPS-Position, die Ortsfelder und, wenn Sie es wählen, die Höhe.

---

## 11. Problembehebung

**Ein Tauchgang hat kein Foto.** Prüfen Sie die Kamerauhr in Schritt 3: Verwenden Sie **Vorschlagen** oder stellen Sie die Abweichung von Hand ein. Wurde während der Reise eine Kamerauhr oder die Uhr des Tauchcomputers umgestellt, führen Sie die Tage vor und nach der Umstellung als zwei getrennte Gruppen durch, jeweils mit eigenem Zeitversatz: **Tauchgang N getrennt behandeln** in der Zusammenfassung tut dies für die Tauchgänge, die ohne Fotos geblieben sind.

**„MacDive-Datenbank nicht gefunden“.** Die Datenbank wurde vielleicht verschoben, zum Beispiel in iCloud: Verwenden Sie **Datenbank suchen…** oder **Einstellungen › Wählen…**. Liegt sie in iCloud, prüfen Sie, ob sie auf diesen Mac geladen ist.

**„Das Logbuch hat sich geändert…“.** Das verwendete Logbuch hat gewechselt (zum Beispiel in den Einstellungen), während Dive Lens mit dem vorherigen arbeitete, die Aktion wurde also abgelehnt. Nach **Anwenden** lautet der orange Hinweis „Das Logbuch hat sich geändert. Es wurde nichts geschrieben: Beginnen Sie von vorn.“ Nach einer Aktion, die nichts schreibt (Fotos suchen, ein Profil anzeigen), lautet er „Das Logbuch hat sich geändert: Beginnen Sie von vorn.“ Er erscheint in Schritt 1, unter den Schritten, bis Sie auf **Schließen** oder **Weiter** klicken. Wählen Sie die Tauchgänge erneut: Sie stammen nun aus dem neuen Logbuch.

**Dive Lens ist nach der Installation deaktiviert.** Lightroom zeigt Dive Lens möglicherweise als deaktiviert, insbesondere wenn ein früheres Dive Lens-Zusatzmodul deaktiviert worden war, bevor es entfernt wurde. Wählen Sie in **Datei › Zusatzmodul-Manager…** Dive Lens, klicken Sie auf **Aktivieren**, dann auf **Fertig**. Das ist nur einmal nötig.

**„Lightroom nicht verbunden“.** Öffnen Sie Lightroom Classic, prüfen Sie in **Datei › Zusatzmodul-Manager**, ob Dive Lens aktiviert ist, und wählen Sie dann erneut **Bibliothek › Plug-in-Extras › Dive Lens öffnen…**.

**Die App öffnet sich nicht (Mac).** Meldet macOS, dass es die App nicht überprüfen kann, laden Sie Dive Lens erneut von seiner Release-Seite herunter: Veröffentlichte Versionen sind signiert und von Apple geprüft und öffnen sich ohne diese Meldung.

**Von MacDive zu Subsurface wechseln.** Nichts zu migrieren. Einen Tauchgang erneut durchzuführen findet seine Sammlung, wenn der Name „<Nummer> - <Tauchplatz>“ derselbe ist; sonst wird daneben eine neue Sammlung erstellt. Zeitversätze werden für jede Anwendung getrennt gemerkt.

**Ein Problem melden.** Stellen Sie **Einstellungen › Diagnose › Protokollstufe** auf **Ausführlich**, lösen Sie das Problem erneut aus, dann **Protokoll anzeigen**: Der Ordner enthält die beiden Protokolldateien. [Eröffnen Sie dann ein Issue](https://github.com/Ventrilok/divelens/issues/new/choose), das das Problem beschreibt, ohne die Protokolldateien anzuhängen: Das GitHub-Repository ist öffentlich, und die Protokolle enthalten Ihre Tauchplätze und Dateipfade. Werden sie gebraucht, werden Sie darum gebeten: Senden Sie sie nur per E-Mail an [divelens@querton.com](mailto:divelens@querton.com), mit der Issue-Nummer.

---

## 12. Datenschutz

- Ihr Logbuch wird nur gelesen, nie verändert. MacDive oder Subsurface können währenddessen geöffnet bleiben.
- Dive Lens sendet keine Tauchdaten irgendwohin. Es geht nur für zwei Dinge ins Internet:
  - die Karte des Tauchplatzes: Der Kartendienst (OpenFreeMap, OpenStreetMap-Daten) erhält den Bereich der angezeigten Karte, sonst nichts. Ohne Internet wird die Karte durch die Meldung „Karte ohne Internet nicht verfügbar.“ ersetzt;
  - Updates: GitHub erhält die Anfrage nach der neuesten Version und, wenn Sie installieren, den Download, sonst nichts. **Einstellungen › Updates › Automatisch suchen** schaltet die Suche beim Start aus.
