# Dive Lens — User guide

Dive Lens links your underwater photos in Adobe Lightroom Classic with your dive log. It reads your log from **MacDive** (Mac) or **Subsurface** (Mac and Windows), finds the photos taken during each dive, puts them in one collection per dive and writes the dive data on them: dive number, times, depth, temperatures, rating and site. When the dive has a profile, each photo also gets the **depth, water temperature and time into the dive at the moment it was taken**.

Dive Lens is free and open source (GPL v3).

> Draft for review. The examples (dives, sites, cameras) are fictional.

---

## 1. Install

You need Lightroom Classic and MacDive or Subsurface on the same computer. On Windows, only Subsurface is available.

1. Download the zip for your system: `DiveLens-<version>-macOS.zip` or `DiveLens-<version>-Windows.zip`.
2. Unzip it. You get a folder named **DiveLens.lrplugin** (on a Mac it looks like a single file). The Dive Lens app is inside it.
3. Move this folder to a place where it will stay, for example your Documents folder. Lightroom loads it from there each time it starts.
4. In Lightroom: **File › Plug-in Manager… › Add**, choose the **DiveLens.lrplugin** folder, then **Done**.

**Coming from the previous Dive Lens plug-in (0.9 and earlier)?** Quit Lightroom, replace the old DiveLens.lrplugin folder with the new one at the same place, and start Lightroom again. Your photos keep their dive data, and the app takes over your previous settings (log location and dive order) on its first launch. Dive Lens is now free: there is no licence key any more.

### First launch on a Mac

This version is not yet signed by Apple. The first time Lightroom opens the app, macOS refuses to open it, saying it cannot check the developer.

1. Close the message (**Done**, not **Move to Trash**).
2. Open **System Settings › Privacy & Security**. Near the bottom, a line says Dive Lens was blocked: click **Open Anyway** and confirm with your password.
3. Choose the Dive Lens menu in Lightroom again.

You only do this once.

### First launch on Windows

This version is not yet signed. When Lightroom opens the app for the first time, Windows may show **"Windows protected your PC"**: click **More info**, then **Run anyway**. You only do this once.

The app needs Microsoft's **WebView2**, which is already present on Windows 11 and on most Windows 10 computers. If the app does not open, install the "Microsoft Edge WebView2 Runtime" from Microsoft's website.

---

## 2. Open Dive Lens

Dive Lens has no window inside Lightroom. Everything happens in the Dive Lens app, which Lightroom opens for you:

- **Library › Plug-in Extras › Create Collections…**
- **Library › Plug-in Extras › Tag Selection…** (only when photos are selected)

Lightroom must stay open while you use the app. The app's sidebar shows whether Lightroom is connected and how many photos are selected. If it says **Lightroom not connected**, nothing can be written: check that Lightroom is open and the plug-in is enabled in the Plug-in Manager.

The first time, check **Settings** (section 5): the dive application and the location of your log.

---

## 3. Create Collections

The main use: choose dives, and Dive Lens files the photos taken during them.

### Step 1 — Select Dives

The list shows every dive of your log, grouped by month. Search by number, site, location or country, and narrow down with the filters (period, country, depth, rating, **With profile**). Click a dive to see its detail on the right: profile, duration, max depth, temperatures, rating, site GPS with a map, and your notes.

Tick one or more dives, then **Next**.

A dive **without a profile** (entered by hand, or from an older dive computer whose download MacDive did not keep) still works: its photos get the dive data and the site, but no depth, temperature or time.

### Step 2 — Adjust & Review

Dive Lens looks in the whole catalog for photos taken over the period of the chosen dives, and lists the cameras it finds (two bodies of the same model are told apart by serial number).

**Camera clocks.** A camera's clock is rarely exactly on time with the dive computer, and is often still on the time of home. For each camera, set the difference between its clock and the dive log's:

- **Suggest** proposes the offset that puts the most photos in the dives. It tries every quarter of an hour between −14 h and +14 h, so it also finds a forgotten time zone.
- **Suggest for all** does it for every camera at once, without touching cameras you set by hand or ignore.
- You can also type the offset (hours:minutes) or adjust it yourself.
- **Ignore** leaves out a camera, for example a phone that stayed on the boat: its photos are neither counted nor tagged.

Dive Lens remembers the last offset used for each camera and proposes it next time.

**Include photos taken at the surface.** Optional, 0 minutes by default: up to N minutes before and after each dive, for photos taken on the boat just before going in or just after getting out. They join the dive's collection and get the dive data, without depth.

**Preview by dive.** The counts update as you adjust: photos per dive, cameras, and whether per-photo data will be written. A dive with **no photo** is highlighted: it often means a camera clock was changed during the trip (see section 7).

The bar at the bottom tells you what will be written and where the collections will go. Click **Apply to N photos**.

### Step 3 — Summary

You see the collections created or added to, the photos tagged, how many got a depth, and the dives without photos (no collection is created for them).

### What is written

- One collection per dive, named **"<dive number> - <site>"**, in the **Dive Lens** collection set (the place can be changed in Settings).
- On each photo: dive number, start and end, duration, max depth, min and max temperatures, rating, the dive application it comes from, and the clock offset applied.
- The site: country, city, location and GPS, in Lightroom's standard fields.
- When the dive has a profile: the photo's **depth, temperature and time into the dive**, estimated between the two profile points around the moment of the photo.

Good to know:

- The photo's time is its **original capture time**, so a TIFF that came back from Photoshop is matched like its original.
- A photo that would fall in two dives goes to the closer one.
- Running a group of dives again only adds: photos already filed stay where they are (correct them by hand if needed), and photos found again get their values rewritten.

**Undo.** **Edit › Undo** in Lightroom reverts everything in one step, except the GPS (and the altitude, if you copy the depth into it): Lightroom does not restore those.

---

## 4. Tag Selection

For photos without a reliable time, to attach photos to a dive by hand, or to set only a location (photos on the boat, on the beach, snorkelling).

1. In Lightroom, select the photos. The app follows the selection live and shows its thumbnails.
2. Tick what to write:
   - **Dive data**: then click a dive in the list. The dive whose times cover the selection carries the **Covers the selection** badge (it uses the camera's last offset). It is only a hint: you choose.
   - **Location**: the **Dive site** of the chosen dive, or an **Other site** chosen by country › location › site among the sites of your log. Its GPS can be edited (decimal degrees, for example `-20.2531` and `57.5125`).
3. Click **Apply to N photos**. The photos selected in Lightroom **at that moment** are written.

What is written: with **Dive data**, the dive number, start, end, duration, max depth, temperatures and rating (no per-photo depth and no collection); with **Location**, country, city, location and GPS.

- A value the chosen dive does not have (temperature, rating…) clears the old Dive Lens value on the photo, so the fields always describe a single dive.
- A site without country, city or GPS leaves the values already on the photo untouched (a GPS position from a phone stays).
- **Edit › Undo** in Lightroom reverts it in one step, except the GPS.

---

## 5. Settings

Every change is saved at once.

**Dive log**
- **Application**: MacDive or Subsurface (Subsurface only on Windows).
- **MacDive database** / **Subsurface log**: the standard location is used unless you choose another. A Subsurface log is either a file (.xml or .ssrf) or the folder of its git storage (cloud storage keeps one on your computer). Under the location, Dive Lens shows whether the log is valid and how many dives and sites it holds.
- Each application keeps its own location: switching from one to the other and back finds yours again.

**Display**
- **Language**: the system's by default. The menus and fields inside Lightroom follow Lightroom's language.
- **Depth** and **Temperature**: **As MacDive** / **As Subsurface** (the units chosen in that application), or forced to metres / feet and °C / °F. They apply in the app and to the values written on photos.

**Writing on photos**
- **Copy depth to GPS altitude** (off by default): also writes each photo's depth as a negative altitude, in metres.
  - The Dive Lens depth field only exists in Lightroom: it is lost on export.
  - The GPS altitude travels with exported photos and shows in other software.
  - The copy replaces a GPS altitude already on the photo, and Lightroom's Undo does not bring it back. Photos without a depth keep theirs.

**Collections**
- **Place in**: the catalog root or an existing collection set (Lightroom must be connected to browse them).
- **Group in the set**: "Dive Lens" by default, created when needed; leave it empty to put the collections straight in the place above. A preview shows the resulting path. Collections already created stay where they are.

**Diagnostics**
- **Log level**: Normal or Detailed. Raise it only to report a problem.
- **Show log**: opens the folder with the two log files, the app's (`DiveLens App.log`) and the Lightroom plug-in's (`DiveLens.log`).

---

## 6. The Dive Lens fields in Lightroom

In the **Library** module, open the **Metadata** panel and choose **Dive Lens** in its menu. The fields are in four groups:

- **Photo**: Depth, Temperature, Elapsed Time;
- **Dive**: Dive, Start Dive, End Dive, Duration, Max Depth, Temp. Low, Temp. High, Rating;
- **Site**: Lightroom's location, city, country and GPS;
- **Technical**: Clock Offset, Source.

The Dive Lens fields stay in the Lightroom catalog: they are not written into the photo files and do not travel with exports. Only the GPS position, the location fields and, if you choose it, the altitude do.

---

## 7. Troubleshooting

**A dive has no photo.** Check the camera clock: use **Suggest**, or set the difference by hand. If a camera clock or the dive computer's clock was changed during the trip, run the days before and after the change as two separate groups, each with its own offset (**Handle dive N separately** on the summary does this for the dives left without photos).

**"MacDive database not found".** The database may have moved, for example to iCloud: use **Locate the database…** or **Settings › Choose…**. If it is in iCloud, check that it is downloaded on this Mac.

**"Lightroom not connected".** Open Lightroom Classic, check in **File › Plug-in Manager** that Dive Lens is enabled, then choose one of the Dive Lens menus again.

**The app does not open (Mac).** See "First launch on a Mac" above: macOS blocks it until you click **Open Anyway** once.

**Switching from MacDive to Subsurface.** Nothing to migrate. Running a dive again finds its collection when the name "<number> - <site>" is the same; otherwise a new collection is created next to it. Clock offsets are remembered separately for each application.

**Reporting a problem.** Set **Settings › Diagnostics › Log level** to **Detailed**, reproduce the problem, then **Show log** and send the two files.

---

## 8. Privacy

- Your dive log is only read, never modified. MacDive or Subsurface can stay open meanwhile.
- Dive Lens sends no dive data anywhere. The only Internet access is the site map: the map service (OpenFreeMap, OpenStreetMap data) receives the area of the map displayed, nothing else. Without Internet, the map is replaced by the message "Map unavailable without Internet."
