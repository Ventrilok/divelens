# Dive Lens — User guide

Dive Lens links your underwater photos in Adobe Lightroom Classic with your dive log. It reads your log from **MacDive** (Mac) or **Subsurface** (Mac and Windows). In one journey of five steps, you choose the photos and the dives, set the camera clocks, check what will be written, then apply: Dive Lens puts the photos in one collection per dive and writes the dive data on them: dive number, times, depth, temperatures, rating and site. When the dive has a profile, each photo also gets the **depth, water temperature and time into the dive at the moment it was taken**.

Dive Lens is free and open source (GPL v3).

> Draft for review. The examples (dives, sites, cameras) are fictional.

---

## 1. Install

You need Lightroom Classic and MacDive or Subsurface on the same computer. On Windows, only Subsurface is available.

**Coming from the previous Dive Lens plug-in (0.9 and earlier)?** Before using the installer, remove it: in Lightroom, **File › Plug-in Manager…**, choose Dive Lens, **Remove**, then **Done**. If **Remove** is greyed out, the old plug-in is already where the installer will replace it: there is nothing to do. Your photos keep their dive data, and the app takes over your previous settings (log location and dive order) on its first launch. Dive Lens is now free: there is no licence key any more.

### With the installer

1. Quit Lightroom.
2. Download the installer for your system: `DiveLens-<version>-macOS.pkg` or `DiveLens-<version>-Windows-Setup.exe`.
3. Open it and follow the steps. It installs Dive Lens for your account only, without an administrator password.
4. Start Lightroom: Dive Lens is in **Library › Plug-in Extras**. There is nothing to add in the Plug-in Manager.

On Windows, the installer is not signed yet: Windows may show **"Windows protected your PC"**. Click **More info**, then **Run anyway**.

### With the zip, by hand

1. Download the zip for your system: `DiveLens-<version>-macOS.zip` or `DiveLens-<version>-Windows.zip`.
2. Unzip it. You get a folder named **DiveLens.lrplugin** (on a Mac it looks like a single file). The Dive Lens app is inside it.
3. Move this folder to a place where it will stay, for example your Documents folder. Lightroom loads it from there each time it starts. If you used the previous plug-in, quit Lightroom and replace the old DiveLens.lrplugin folder with the new one at the same place instead: you can then skip step 4.
4. In Lightroom: **File › Plug-in Manager… › Add**, choose the **DiveLens.lrplugin** folder, then **Done**.

On a Mac, the first time Lightroom opens the app, macOS may ask you to confirm opening an app downloaded from the internet: click **Open**. On Windows, the app is not signed yet: if Windows shows **"Windows protected your PC"**, click **More info**, then **Run anyway**. You only do this once.

### WebView2 on Windows

The app needs Microsoft's **WebView2**, which is already present on Windows 11 and on most Windows 10 computers. If the app does not open, install the "Microsoft Edge WebView2 Runtime" from Microsoft's website.

### Uninstall

Your photos keep their dive data and their collections in the catalog: they come back if you install Dive Lens again.

**On a Mac**

1. Quit Lightroom.
2. In the Finder, choose **Go › Go to Folder…**, paste `~/Library/Application Support/Adobe/Lightroom/Modules`, then move **DiveLens.lrplugin** to the Trash. If you installed Dive Lens from the zip, remove it first in **File › Plug-in Manager…** (**Remove**), then move the DiveLens.lrplugin folder to the Trash from where you had put it.
3. Optional, to erase Dive Lens's settings as well: move the folder `~/Library/Application Support/com.querton.divelens.desktop` to the Trash.

**On Windows**

1. Quit Lightroom.
2. **Settings › Apps › Installed apps › Dive Lens › Uninstall**. If you installed Dive Lens from the zip, remove it in **File › Plug-in Manager…** (**Remove**), then delete the DiveLens.lrplugin folder from where you had put it. Once Dive Lens has updated it, a copy installed from the zip is listed in **Installed apps** too: uninstall it from there.
3. Optional, to erase Dive Lens's settings as well: delete the folder `%APPDATA%\com.querton.divelens.desktop`.

---

## 2. Open Dive Lens

Dive Lens has no window inside Lightroom. Everything happens in the Dive Lens app, which Lightroom opens for you:

- **Library › Plug-in Extras › Open Dive Lens…** opens the app at step 1 of the journey (section 3), whether photos are selected or not.
- **Help › Plug-in Extras › Dive Lens Settings…** opens it on **Settings** (section 9).

The Dive Lens entry in **File › Plug-in Manager…** also has **Open Dive Lens** and **Open Settings** buttons, and links to the Dive Lens website and to its source code on GitHub.

If the app is already open, the menu brings its window to the front. Chosen during the journey, **Open Dive Lens…** takes you back to step 1 and keeps your choices; chosen on the summary, it starts a new run.

Lightroom must stay open while you use the app. The first time, check **Settings** (section 9): the dive application and the location of your log.

### The window

- **At the top**: Dive Lens on the left; on the right, the gear opens **Settings** (section 9). The gear is greyed while Dive Lens writes to Lightroom.
- **At the bottom**, the status bar. On the left, Lightroom:
  - **Lightroom connected**, **Lightroom not connected**, or **Restart Lightroom**;
  - **Catalog:** the name of the catalog open in Lightroom;
  - **Catalog size:** how many photos it holds, counted again each time you come back to step 1;
  - **Selected photos:** how many are selected in Lightroom, live (**none** when none is).
- On the right of the status bar, your dive log: **Dive log app:** MacDive or Subsurface, and **Dives logged:** how many dives it holds.

If the status bar says **Lightroom not connected**, nothing can be written: check that Lightroom is open and the plug-in is enabled in the Plug-in Manager. If it says **Restart Lightroom**, Dive Lens was updated while Lightroom was open: quit Lightroom and start it again.

---

## 3. The journey

Five steps, shown at the top: **Scope › Dives › Clocks › Review & Apply › Summary**. **Next** moves on and **Back** goes back; a click on a step already done goes back to it too, except from the summary. Under the steps, a line starting with **Photos:** recalls what you chose. Nothing is written in Lightroom before you click **Apply to N photos** at step 4.

The bar with **Back** and **Next** (or **Apply to N photos**) stays at the bottom of the window, just above the status bar, and the steps stay at the top: when the window is small, only the content in between scrolls. Nothing is ever hidden under the bar.

### Two paths

Step 1 asks the one real question: which photos?

- **Entire catalog**: you tick one or more dives, and Dive Lens looks in the whole catalog for the photos taken during each one. Each photo goes to the dive that matches its time. For a whole trip or a series of dives. The photos selected in Lightroom are ignored.
- **Lightroom selection**: the photos you selected in Lightroom. You choose one dive, and all of them are attached to it, whatever their time. For photos without a reliable time, to attach photos to a dive by hand, or to give them only a location (section 7).

Choosing the other path at step 1 starts the next steps again: dives, clocks and options.

### Step 1 — Scope

"Which photos to work on?" Two cards: **Entire catalog**, or **Lightroom selection**. The selection card follows Lightroom live: the cameras and the day of the selected photos, and up to six thumbnails. With no photo selected, it is greyed and says **No photos selected**: select photos in Lightroom and it lights up. Choose a card, then **Next**.

### Step 2 — Dives

The list shows every dive of your log, grouped by month. Search by number, site, location or country, and narrow down with the filters (**Period**, **Country**, **Depth**, **Rating**, **With profile**, **With GPS**, **With notes**). **Sort** orders the list (**Newest first**, **Oldest first**, **Max depth**); Dive Lens remembers your choice. Click a dive to see its details on the right: profile, duration, max depth, temperatures, rating, site GPS with a map, and your notes.

Each row has three marks at its end, in colour when the dive has it and greyed when not: **Profile**, **GPS** (its site has a GPS position) and **Notes**. Hover over a mark to read it, for example "Site GPS available" or "No notes". The **GPS** column and the **With GPS** filter show only when the dive application gives site positions, as MacDive and Subsurface do.

- **Entire catalog**: tick one or more dives, then **Next**. The bar shows how many are ticked and their days; **Deselect all** unticks them.
- **Lightroom selection**: click one dive, then **Next**.
  - The dive whose times cover the selection carries the **Covers the selection** badge. It uses the last offset applied to each camera, then the offsets you set at step 3. It is only a hint: you choose.
  - The first row, **No dive — location only**, is for giving the photos only a location (section 7).

A dive **without a profile** (entered by hand, or from an older dive computer whose download MacDive did not keep) still works: its photos get the dive data and the site, but no depth, temperature or time.

### Step 3 — Clocks

**Camera clocks.** A camera's clock is rarely exactly on time with the dive computer, and is often still on the time of home. Each camera has a card (two bodies of the same model are told apart by serial number). Set the difference between its clock and the dive log's:

- **Suggest** proposes the offset that puts the most photos in the dives. It tries every quarter of an hour between −14 h and +14 h, so it also finds a forgotten time zone.
- **Suggest for all** does it for every camera at once, without touching cameras you set by hand or ignore.
- You can also type the offset (hours:minutes) or adjust it yourself.
- Each card starts from the last offset applied to that camera, on either path (**remembered offset**); with none, it starts at 0:00. Offsets are remembered separately for each dive application. **Settings › Remembered clock offsets** lists them and forgets the one of a camera (section 9).

**Preview by dive.** The counts follow your settings: photos per dive, cameras, and the per-photo data each dive gives (**Depth · temp. · time**, **No profile: dive data only**, **Surface photos only**). For a dive with no photo, its count, 0, and **No photo — clock changed?** are written in orange: a camera clock was often changed during the trip (section 11).

On the **Entire catalog** path:

- Dive Lens looks in the whole catalog for the photos taken over the period of the dives ticked.
- **Ignore** leaves out a camera, for example a phone that stayed on the boat: its photos are neither counted nor tagged.
- **Include photos taken at the surface**: up to N minutes before and after each dive, for photos taken on the boat just before going in or just after getting out. 0 minutes by default, and back to 0 on every run (**Handle dive N separately** and **Handle these dives separately** keep it). These photos get the dive data, without depth.
- **Outside the dives**: the photos of the period that fall in no dive, even with the margin (on land, on the boat…). They get nothing (section 7).

On the **Lightroom selection** path:

- The photos are attached to the chosen dive whatever their time. The clock only gives them a depth: only those taken during the dive get one.
- One card per camera of the selection. There is no **Ignore** (remove a photo from the selection in Lightroom instead) and no surface margin.
- **Outside the dive's times** lists the photos taken before or after the dive, for example "Taken at 15:45, 7 minutes after getting out of the water. Attached to the dive anyway: it gets its data and its location, without depth."
- A dive without a profile shows "No profile: no per-photo data" instead of the camera clocks, and a dive whose profile cannot be read "Profile could not be read: no per-photo data": the dive data are then written without per-photo data.
- With **No dive — location only**, this step is skipped: **Clocks** is greyed in the steps.

### Step 4 — Review & Apply

"What Dive Lens writes in Lightroom." The same options on both paths. They are all ticked on every run, except **Copy depth to GPS altitude**, which is unticked on every run: nothing is remembered from one run to the next. On the right of each option, its effect on this run, for example "12 photos · 9 with depth". [Section 4](#4-which-options-for-which-need) shows what each combination of options writes on the photos, and which options to choose for which need.

**Metadata**

- **Dive details**: dive number, start, end, duration, max depth, temperatures, rating and source (the dive application). On the photos taken during the dive, also the depth, temperature and time into the dive, and the clock offset.
  - **Copy depth to GPS altitude**, under it, greyed when **Dive details** is unticked: also writes the depth in the GPS altitude, as a negative value, in metres, on the photos that have a depth (section 6).
- **Site details**: the location and GPS of the dive's site, or of another site of your log.
  - **Entire catalog**: **Each dive's site**, or **Other site, the same for all photos…**.
  - **Lightroom selection**: **Dive's site: <name>**, or **Other site…**.
  - **Other site**: chosen by country › location › site among the sites of your log. Its GPS can be edited before applying, in decimal degrees (for example `12.5000` and `-45.2500`). The collections keep the name of their dive's own site.
  - **Location**: country, city and location. **GPS**: the position. Untick one to write only the other. A site without GPS leaves the GPS already on the photo in place.

**Collections**

- **Create collections** (an option of this step, no longer a Lightroom menu): on the **Entire catalog** path, one "<number> - <site>" collection per dive that has photos; on the **Lightroom selection** path, the chosen dive's collection, which gets all the selected photos. A photo already in the collection is not added twice. Where the collections go is set in Settings: the link **Place in Settings** opens them (section 9).

**Good to know before applying** lists what gets nothing or less: the dives without photo and the photos outside the dives (**Entire catalog**), the photos that will get no depth (**Lightroom selection**), and the units used.

The bar at the bottom sums up what will be written ("N photos · N collections") and reminds you that one Undo takes it all back. Click **Apply to N photos**. On the **Entire catalog** path, N is every photo found in the dives ticked. On the **Lightroom selection** path, the photos selected in Lightroom **at the moment you click** are written.

**Apply** stays greyed, with the reason in the bar, until something can be written, for example:

- "Tick at least one option";
- "Choose a site", while no site is chosen under **Other site**;
- "Select photos in Lightroom", when the selection is empty;
- "Nothing to write with these options.";
- the collection set chosen in Settings is no longer in the catalog (only with **Create collections** ticked): **Open Settings** lets you choose another place;
- Lightroom is not connected.

### Step 5 — Summary

**Written in Lightroom**: **Photos tagged** (the photos written, those only put in a collection included), **With depth**, and **Collections**, then each collection with its number of photos, **created** or **added to**. Then the warnings (dives without photo, photos outside the dives, photos without depth) and the reminder of how to undo (section 8).

- **Entire catalog**: **Back to scope selection** returns to step 1 and starts a new run: **Entire catalog** is still chosen, no dive is ticked and the options are back to their defaults. When a dive got no photo, **Handle dive N separately** (or **Handle these dives separately**) goes straight to step 3 with only those dives ticked, and looks for their photos again: give their camera its own offset (section 11).
- **Lightroom selection**: **Done** returns to step 1 and starts a new run in the same way, the **Lightroom selection** card still chosen.

---

## 4. Which options for which need

The options of step 4 decide what Dive Lens writes on each photo. This section shows what each combination writes, path by path, then which options to choose in situations divers often meet. The detailed rules are in [section 5](#5-what-is-written).

### How the options combine

- **Dive details**, **Site details** and **Create collections** are independent: what one of them writes never depends on the others, so the rows of the tables below combine. For example, **Dive details** and **Create collections** ticked without **Site details** write the dive data and the collections, and leave the location and GPS as they are.
- **Copy depth to GPS altitude** works only with **Dive details**: unticking **Dive details** greys it.
- Under **Site details**, **Location** and **GPS** are independent too; unticking both is the same as unticking **Site details**. Both come from the dive's own site, or from **Other site**.
- **Other site** changes only the location and GPS written on the photos: each collection keeps the name of its dive's own site.
- Even with **Dive details** unticked, the photos are still matched to their dive by their time (**Entire catalog**), for the site and the collections.
- **No dive — location only** (**Lightroom selection** path) leaves only **Site details**: **Dive details** and **Create collections** are greyed, and the site is always **Other site…**.

The photos written:

- **Entire catalog**: the photos placed in a dive ticked, taken during the dive or in its surface margin. Ignored cameras and photos outside the dives are not touched.
- **Lightroom selection**: every photo selected at the moment you click **Apply**, whatever its time.

### What each photo gets

In the tables, "Kept" means that Dive Lens does not touch the field: the photo keeps what it had, from an earlier run, from the camera or from you. "None" under **Collection** means that no collection is created or changed: the photo stays in the collections it was already in. A site without a country, city, location or GPS leaves that value on the photo as it was. The dive fields written describe one dive only: a value the dive does not have, such as a temperature or a rating, is emptied.

**Entire catalog**

| Ticked at step 4 | Dive fields | Depth, temperature, time | Clock offset | GPS altitude | Country, city, location | GPS | Collection |
|---|---|---|---|---|---|---|---|
| Everything (the default) | Written | During the dive¹ | Written | Kept² | Each dive's site | Each dive's site | Its dive's |
| Everything, and **Copy depth to GPS altitude** | Written | During the dive¹ | Written | The depth³ | Each dive's site | Each dive's site | Its dive's |
| **Dive details** only | Written | During the dive¹ | Written | Kept² | Kept | Kept | None |
| **Dive details** and **Copy depth to GPS altitude** only | Written | During the dive¹ | Written | The depth³ | Kept | Kept | None |
| **Site details** only (**Location** and **GPS**) | Kept | Kept | Kept | Kept | Each dive's site | Each dive's site | None |
| **Location** only | Kept | Kept | Kept | Kept | Each dive's site | Kept | None |
| **GPS** only | Kept | Kept | Kept | Kept | Kept | Each dive's site | None |
| Everything, with **Other site, the same for all photos…** | Written | During the dive¹ | Written | Kept² | The other site | The other site⁴ | Its dive's⁵ |
| **Create collections** only | Kept | Kept | Kept | Kept | Kept | Kept | Its dive's |

**Lightroom selection**

| Ticked at step 4 | Dive fields | Depth, temperature, time | Clock offset | GPS altitude | Country, city, location | GPS | Collection |
|---|---|---|---|---|---|---|---|
| Everything (the default) | Written | During the dive¹ | During the dive¹ | Kept² | The dive's site | The dive's site | The dive's |
| Everything, and **Copy depth to GPS altitude** | Written | During the dive¹ | During the dive¹ | The depth³ | The dive's site | The dive's site | The dive's |
| **Dive details** only | Written | During the dive¹ | During the dive¹ | Kept² | Kept | Kept | None |
| **Dive details** and **Copy depth to GPS altitude** only | Written | During the dive¹ | During the dive¹ | The depth³ | Kept | Kept | None |
| **Site details** only (**Location** and **GPS**) | Kept | Kept | Kept | Kept | The dive's site | The dive's site | None |
| **Location** only | Kept | Kept | Kept | Kept | The dive's site | Kept | None |
| **GPS** only | Kept | Kept | Kept | Kept | Kept | The dive's site | None |
| Everything, with **Other site…** | Written | During the dive¹ | During the dive¹ | Kept² | The other site | The other site⁴ | The dive's⁵ |
| **Create collections** only | Kept | Kept | Kept | Kept | Kept | Kept | The dive's |
| **No dive — location only**, with **Location** and **GPS** | Kept | Kept | Kept | Kept | The other site | The other site⁴ | None |
| **No dive — location only**, with **Location** only | Kept | Kept | Kept | Kept | The other site | Kept | None |
| **No dive — location only**, with **GPS** only | Kept | Kept | Kept | Kept | Kept | The other site⁴ | None |

1. On the photos taken during a dive that has a profile: the depth, temperature and time into the dive at the moment of the photo.
   - **Entire catalog**: the other photos placed in a dive (in the surface margin, or in a dive without a profile) have these three values emptied. Every photo placed in a dive gets the clock offset.
   - **Lightroom selection**: the clock offset goes only on the photos taken during the dive, with their depth. The other photos keep their values when they already carry the same dive; when they carried another dive, its depth, temperature, time and clock offset are emptied.
2. Kept, except an altitude Dive Lens copied itself: it is cleared when the photo is attached to another dive without a new altitude and, on the **Entire catalog** path, when the photo is found again without a depth ([section 6](#6-depth-in-the-gps-altitude)).
3. On the photos that get a depth: that depth, as a negative GPS altitude in metres. The other photos as in note 2.
4. The GPS of the site picked, or the position you typed under it. Emptying both coordinates writes the site's own position: to keep the photos' GPS, untick **GPS**.
5. The collection keeps the name of the dive's own site.

### When Apply stays greyed

| Situation | What the bar says |
|---|---|
| **Dive details**, **Site details** and **Create collections** all unticked; with **No dive — location only**, **Location** and **GPS** both unticked | "Tick at least one option" |
| **Other site** chosen, or imposed (**No dive — location only**, or a chosen dive that has no site), and no site picked yet | "Choose a site" |
| The position typed under **Other site** is not valid, while **GPS** is ticked | "Latitude between -90 and 90, longitude between -180 and 180, or both empty." |
| The options ticked leave nothing to write on any photo: for example **GPS** alone on a site without GPS, or **Location** alone on a site with no name, country or city | "Nothing to write with these options." |
| **Entire catalog**: no photo falls in the dives ticked | "No photo falls in the dives: adjust the clock offsets." |
| **Lightroom selection**: no photo selected | "Select photos in Lightroom" |
| **Create collections** ticked, and the collection set chosen in Settings is no longer in the catalog | "The collection set chosen in Settings (<place>) is no longer in the catalog. Choose another place in Settings.", with **Open Settings** |

On the **Entire catalog** path, **GPS** alone (or **Location** alone) is not refused when only some of the dives' sites give it: only the photos of those dives get it. The effect on the right of **Site details** counts the photos that really get something, while **Apply to N photos** still counts every photo found.

### Use cases

**Tagging a whole trip, the first time.** **Entire catalog**; tick the trip's dives; at step 3, **Suggest for all**, then check each camera's card and the preview by dive; leave the options as they are; **Apply**. Each photo taken during a dive gets the dive data, its site's location and GPS, the clock offset and, when the dive has a profile, its depth, temperature and time into the dive. Each dive with photos gets its "<number> - <site>" collection. The photos outside the dives get nothing: see the boat and the beach, below.

**Adding the collections later, without touching the photos' data.** **Entire catalog**, the same dives; at step 3 the cards start from the offsets remembered at the first run; at step 4, untick **Dive details** and **Site details** and keep **Create collections**. The bar reads "N collections · nothing is written on the photos". Each dive's photos go into its collection; a photo already in it is not added twice. The other way round works too: untick only **Create collections** to write the data without any collection.

**Photos taken on the boat or the beach between dives.** On the **Entire catalog** path they fall outside the dives and get nothing. Select them in Lightroom, then choose **Lightroom selection** and **No dive — location only**; at step 4, pick the site by country › location › site and, if the beach is not where the site is, type its position under it; **Apply**. They get only the country, city, location and GPS: no dive field and no collection, and the dive data some of them may already carry stay as they are. To give them a dive's data instead, choose that dive: they get its data, its site and its collection, without depth ([section 7](#7-location-only-and-photos-outside-the-dives)).

**A dive without a profile.** A dive entered by hand, or whose computer download the dive application did not keep, needs nothing special. Its photos get the dive data, the site and the collection, without depth, temperature or time, so **Copy depth to GPS altitude** has nothing to copy for them. On the **Entire catalog** path the clock offset still decides which photos belong to the dive, and is written on them; the preview by dive reads "No profile: dive data only". On the **Lightroom selection** path, step 3 reads "No profile: no per-photo data" and no clock offset is written.

**Correcting the site of photos already tagged.** Select the photos in Lightroom, choose **Lightroom selection** and their dive; at step 4, untick **Dive details** and **Create collections**, choose **Other site…** and the right site; **Apply**. Only the country, city, location and GPS change: the dive data and the collections stay. A value the new site does not have is not cleared: if the old site had a city and the new one has none, the old city stays; clear it in Lightroom. For the photos of several dives at once, do the same on the **Entire catalog** path with **Other site, the same for all photos…**. If the site is wrong in your dive log, correct it there instead and run the dive again: the photos get the new site, and with **Create collections** ticked, a collection with the new site's name is created next to the old one.

**Keeping a GPS position from the camera or set by hand.** Untick **GPS** under **Site details**: the country, city and location are written, the position stays. Even with **GPS** ticked, a site without a position leaves the photo's own. If a position was replaced by mistake, **Edit › Undo Dive Lens** in Lightroom brings it back.

**The depth as GPS altitude, for maps and other software.** Tick **Copy depth to GPS altitude**, under **Dive details**: each photo that gets a depth also gets it as a negative GPS altitude. The option is unticked on every run, so that you decide each time, knowing four things: the Dive Lens **Depth** field stays in Lightroom and is lost on export; the GPS altitude travels with exported photos and shows in other software; it is always in metres, whatever your units; and it replaces an altitude already on the photo (**Edit › Undo** brings it back). Details in [section 6](#6-depth-in-the-gps-altitude).

**Running Dive Lens again on photos already tagged.** **Entire catalog**: running dives again only adds. Nothing is duplicated, the photos found again get their values written again with the offsets shown, and a photo that now falls in another dive gets that dive's data and collection but also stays in its old collection: remove it from there by hand. **Lightroom selection**: with the dive the photos already carry, the dive data are written again, and so are the depth, temperature, time and clock offset of the photos taken during the dive; the other photos keep theirs. With another dive, its data replace the old dive's, and the photos stay in the old dive's collection.

**Photos from two cameras with different clocks.** **Entire catalog** handles them in one run: each camera has its own card at step 3 and its own offset; **Suggest for all** sets them all at once, without touching a card you set by hand; two bodies of the same model are told apart by their serial number. **Ignore** a camera whose photos have nothing to do with the dives, such as a phone that stayed on the boat. After **Apply**, each camera's offset is remembered for next time (**Settings › Remembered clock offsets**). A clock changed in the middle of the trip is another matter: run the days before and after the change separately, each with its own offset ([section 11](#11-troubleshooting)).

**Photos outside the dive's times, on the Lightroom selection path.** They are attached to the chosen dive anyway: they get its data, its site and its collection, but no depth, temperature, time or clock offset. Step 3 lists them under **Outside the dive's times**, step 4 under **Good to know before applying**, and the summary counts them as photos without depth. Not meant for that dive? Remove them from the selection in Lightroom before **Apply**. On the **Entire catalog** path, **Include photos taken at the surface** does the same for photos taken just before or just after a dive: they get the dive data, the site, the clock offset and the collection, without depth.

**Undo.** Whatever the options, one **Apply** is one step in Lightroom: **Edit › Undo Dive Lens** takes back everything it wrote, GPS and altitude included ([section 8](#8-undo)).

---

## 5. What is written

- **Create collections**: one collection per dive, named **"<dive number> - <site>"**, in the **Dive Lens** collection set (the place can be changed in Settings). A dive without a number gets its start date and time in place of the number.
- **Dive details**: on each photo, dive number, start and end, duration, max depth, min and max temperatures, rating and the dive application the data come from. On the photos taken during a dive that has a profile, also the photo's **depth, temperature and time into the dive**, estimated between the two profile points around the moment of the photo. And the clock offset applied: on every photo placed in a dive (**Entire catalog**), on the photos taken during the dive, when it has a profile (**Lightroom selection**).
- **Site details**: country, city, location and GPS, in Lightroom's standard fields.

Good to know, on both paths:

- The photo's time is its **original capture time**, so a TIFF that came back from Photoshop is matched like its original.
- A site without country, city or GPS leaves the values already on the photo untouched (a GPS position from a phone stays).
- With **Dive details** unticked, Dive Lens does not touch the dive fields at all; the photos are still attached to the dives for the site and the collections.
- A value the dive does not have (temperature, rating…) clears the old Dive Lens value on the photo, so the fields always describe a single dive.
- The units chosen in Settings apply to the values written. Photos already tagged keep the units in use when they were written.

On the **Entire catalog** path:

- Only the photos placed in a dive are written, in the dive or in the surface margin. Ignored cameras and photos outside the dives are not touched.
- A photo in the surface margin gets the dive data, the location and the clock offset, without per-photo data.
- A photo that would fall in two dives goes to the closer one.
- Running a group of dives again only adds: photos already filed stay where they are (correct them by hand if needed), and photos found again get their values rewritten.
- A dive whose profile cannot be read stops **Apply** when photos were taken during it: nothing is written.

On the **Lightroom selection** path:

- Every photo selected at the moment of the click is written.
- A photo taken outside the dive's times gets the dive data and the location, without per-photo data or clock offset.
- With **Dive details**, a photo that already carries a **different** dive loses that dive's own values: depth, temperature, time into the dive and clock offset, replaced by the new ones when the photo was taken during the new dive. Its GPS altitude follows section 6. The photo stays in the collection of its old dive: move it yourself if needed.
- Tagging a photo with the dive it already carries rewrites its per-photo data when it was taken during the dive, with the offset shown; the other photos keep theirs.
- If the selection changes in Lightroom just as you click **Apply** while per-photo data are written, nothing is written: "The selection changed in Lightroom. Nothing was written: check the per-photo data, then apply again."
- A profile that cannot be read never blocks: the dive data are written without per-photo data.

After **Apply**, Dive Lens remembers the offset of each camera for next time, on both paths: on the **Entire catalog** path, the cameras with a photo placed in a dive; on the **Lightroom selection** path, the cameras with a photo taken during the dive, when per-photo data are written.

---

## 6. Depth in the GPS altitude

The option **Copy depth to GPS altitude** sits at step 4, under **Dive details**. It is unticked on every run: tick it each time you want it. Four things to know:

- The Dive Lens **Depth** field only exists in Lightroom: it is not written into the photo file, and is lost on export. Dive Lens always fills it.
- The GPS altitude travels with exported photos and shows in other software. With the option ticked, Dive Lens also writes each photo's depth there, as a negative altitude.
- Dive Lens always writes the GPS altitude in metres, the field's standard, whatever the units chosen in Settings. With depths in feet, the same photo shows 60 ft in the Dive Lens **Depth** field, while its GPS altitude is in metres (18.3 m below sea level): this is expected.
- The copy replaces a GPS altitude already on the photo; **Edit › Undo** in Lightroom brings it back. Photos without a depth keep theirs.

And the rules that follow:

- Only the photos that get a depth get an altitude: not those in the surface margin, outside the dive's times, or in a dive without a profile.
- Leaving the option unticked on a later run does not clear the altitudes already copied, with two exceptions:
  - on the **Entire catalog** path, a photo found again without a depth (now in the surface margin, or in a dive without a profile) loses the altitude Dive Lens copied there;
  - a photo attached to another dive without a new altitude loses the one Dive Lens copied from its old depth.
- Dive Lens only clears an altitude it wrote itself: an altitude from the camera, or one you set or changed yourself, stays.
- With **Dive details** unticked, no altitude is written or cleared.

---

## 7. Location only, and photos outside the dives

On the **Entire catalog** path, photos taken outside any dive (on the boat, on the beach, while snorkelling) get nothing: step 3 shows them under **Outside the dives**, and so do the summary and **Good to know before applying**.

To give them a location, or to attach them to a dive anyway:

1. Select them in Lightroom, then **Library › Plug-in Extras › Open Dive Lens…** and choose the **Lightroom selection** card.
2. At step 2:
   - to attach them to a dive anyway, choose the dive: they get its data and its location, without depth;
   - to give them only a location, choose **No dive — location only**.

With **No dive — location only**, step 3 is skipped. At step 4, only **Site details** acts: **Dive details** and **Create collections** are greyed, and the site is **Other site…**. Choose the site by country › location › site, adjust its GPS if needed, then **Apply to N photos**. Only the country, city, location and GPS are written: no dive field and no collection.

---

## 8. Undo

**Edit › Undo** in Lightroom reverts everything one **Apply** wrote, in one step, GPS and altitude included, on both paths. Lightroom's Edit menu names it **Undo Dive Lens**.

- A collection created by **Apply** goes away with it; photos added to an existing collection leave it.
- **The first time**, when the collection set (**Dive Lens** by default) does not exist yet, Dive Lens creates it just before writing. If it is still there, empty, after the Undo, a second **Undo Dive Lens** removes it, or delete it in Lightroom. If the write is refused after the set was created, the empty set stays: undo it, or delete it in Lightroom.
- If Lightroom did not let Dive Lens write the GPS and altitude, everything else is written and Dive Lens says so: choose **Edit › Undo Dive Lens** in Lightroom first, then apply again.

---

## 9. Settings

The gear at the top opens Settings, as does **Help › Plug-in Extras › Dive Lens Settings…** in Lightroom. They take the whole window. **Back** returns to the journey, at the step where you were, with your choices; if the dive log changed meanwhile, the journey starts again at step 1. The gear is greyed while Dive Lens writes to Lightroom.

Every change is saved at once.

**Dive log**
- **Application**: MacDive or Subsurface. Only the applications available on your system are offered: on Windows, Subsurface. The application is changed only here, never from the journey.
- **MacDive database** / **Subsurface log**: the standard location is used unless you choose another. A Subsurface log is either a file (.xml or .ssrf) or the folder of its git storage (cloud storage keeps one on your computer). Under the location, Dive Lens shows whether the log is valid and how many dives and sites it holds.
- Each application keeps its own location: switching from one to the other and back finds yours again.
- **One dive log at a time.** Changing the application or the log starts the journey again at step 1, with nothing chosen: no dive ticked or chosen, clocks and options forgotten. This holds however the log changes: here, by importing settings, from the screen shown when the dive log cannot be read, or when Dive Lens takes over the previous plug-in's settings on connecting to Lightroom (section 1).
- While Dive Lens is writing to Lightroom, the dive log cannot change: **Application**, **Choose…** (or **Choose a file…** / **Choose a folder…**), **Standard location**, **Import…** and **Replace** are greyed, with the note "Writing to Lightroom: the dive log can be changed once the write is done."

**Display**
- **Language**: the system's by default. The menus and fields inside Lightroom follow Lightroom's language.
- **Depth** and **Temperature**: **As MacDive** / **As Subsurface** (the units chosen in that application), or forced to metres / feet and °C / °F. They apply in the app and to the values written on photos. The GPS altitude is always in metres (section 6).

**Collections**
- **Place in**: the catalog root or an existing collection set (Lightroom must be connected to browse them). This is where the **Create collections** option of step 4 puts the collections.
- **Group in the set**: "Dive Lens" by default, created when needed; leave it empty to put the collections straight in the place above. A preview shows the resulting path. Collections already created stay where they are.
- If the set chosen in **Place in** is no longer in the catalog, step 4 says so while **Create collections** is ticked, and **Apply** stays greyed until you choose another place here.

**Remembered clock offsets**
- Under the title, "Kept for each camera after Apply and offered again at the Clocks step, for <application>." The list shows the cameras whose offset is remembered for the dive application chosen above, sorted by name as at step 3. Each line gives the camera's name as on its card at step 3 (its model, or "Unknown camera"), with "no. <serial number>" under it when the photos give one; on the right, its offset as on the card (for example "+4:00", "0:00" or "−0:30"), and **Forget**. With none: "No clock offset remembered."
- **Forget** removes that camera's offset at once, without asking, and its line goes. The next time, step 3 starts its card at 0:00, without **remembered offset**; **Suggest** finds the offset again. The other cameras, and the offsets kept for the other dive application, stay.
- Nothing changes in Lightroom: the clock offset already written on photos stays.
- Forgotten during a journey, on either path: back from Settings, the camera is no longer shown as remembered. If its card at step 3 still carried the **remembered offset** label, it goes back to 0:00, as a new search would show it; an offset you set by hand or found with **Suggest** stays as it is. The label goes, and so does "with the remembered offset of…" under **Covers the selection** at step 2. The counts and **Covers the selection** follow the offset the card now shows, and **Apply** writes and remembers only what the cards show.

**Settings backup**
- **Export or import**: to get your settings back on another computer, or after reinstalling.
- **Export…** saves them in a file, named "Dive Lens settings.json" unless you choose another name. The file holds: the dive application and the log location of each one, language, units, where the collections go, the automatic check for updates, the log level and the order of the dive list. The clock offsets remembered for each camera are not in it: they stay on this computer.
- **Import…** asks before changing anything: **Replace** puts the file's settings in place of every setting on this screen, **Cancel** leaves them as they are.
- If the file's dive log is not on this computer, Dive Lens says so and keeps its location until you choose the right one (**Choose…**, or **Choose a file…** / **Choose a folder…** for Subsurface). If the collection set chosen in **Place in** is not in this catalog, step 4 says so while **Create collections** is ticked: choose another place in Settings.
- A file from a Mac that uses MacDive, imported on Windows: Dive Lens uses Subsurface, the application available there, and keeps the MacDive location: these settings, exported again and imported on a Mac, still use MacDive there.
- A file that does not look like a Dive Lens settings file is refused, and nothing changes. A file exported by an earlier version of Dive Lens can still be imported; its "Copy depth to GPS altitude" setting is ignored, since the option is now at step 4.
- After an import, Dive Lens no longer takes over the previous plug-in's settings (section 1): the imported settings are the ones kept.

**Updates**
- **Check automatically** (on by default): when Dive Lens starts, at most once a day, it looks on GitHub for a new version. Nothing is installed without your consent.
- **Check now** looks at once. Under **Dive Lens <version>**, the line says whether your version is up to date and when it was last checked.
- When a new version exists, a banner at the top of the screen says so: **See** opens this section, **×** closes the banner until the next version. Here, **What's new** opens the version's page on GitHub, and **Install** installs it.
- **Install** downloads the new version, checks that it comes from Dive Lens (signature), replaces the app and the Lightroom part, then Dive Lens starts again. Not while Lightroom is being written to.
- Then **restart Lightroom**: it loads the new Dive Lens part only when it starts. Until then, Dive Lens shows "Restart Lightroom to finish updating Dive Lens.", and the status bar **Restart Lightroom**.
- On Windows, a copy installed with the zip then appears in **Settings › Apps › Installed apps**, where you had put it.
- If Dive Lens cannot change its folder, it offers **Download from GitHub**: then install the new version yourself (section 1).

**Diagnostics**
- **Log level**: Normal or Detailed. Raise it only to report a problem.
- **Show log**: opens the folder with the two log files, the app's (`DiveLens App.log`) and the Lightroom plug-in's (`DiveLens.log`).

---

## 10. The Dive Lens fields in Lightroom

In the **Library** module, open the **Metadata** panel and choose **Dive Lens** in its menu. The fields are in four groups:

- **Photo**: Depth, Temperature, Elapsed Time;
- **Dive**: Dive, Start Dive, End Dive, Duration, Max Depth, Temp. Low, Temp. High, Rating;
- **Site**: Lightroom's location, city, country and GPS, with the GPS altitude under the GPS;
- **Technical**: Clock Offset, Source.

The Dive Lens fields stay in the Lightroom catalog: they are not written into the photo files and do not travel with exports. Only the GPS position, the location fields and, if you choose it, the altitude do.

---

## 11. Troubleshooting

**A dive has no photo.** Check the camera clock at step 3: use **Suggest**, or set the difference by hand. If a camera clock or the dive computer's clock was changed during the trip, run the days before and after the change as two separate groups, each with its own offset: **Handle dive N separately** on the summary does this for the dives left without photos.

**"MacDive database not found".** The database may have moved, for example to iCloud: use **Locate the database…** or **Settings › Choose…**. If it is in iCloud, check that it is downloaded on this Mac.

**"The dive log has changed…".** The dive log in use changed (in Settings, for example) while Dive Lens was working with the previous one, so the action was refused. After **Apply**, the orange notice reads "The dive log has changed. Nothing was written: start again." After an action that writes nothing (finding the photos, showing a profile), it reads "The dive log has changed: start again." It shows at step 1, under the steps, until you click **Close** or **Next**. Choose the dives again: they now come from the new log.

**Dive Lens is disabled after installing.** Lightroom may show Dive Lens as disabled, in particular when a previous Dive Lens plug-in had been disabled before it was removed. In **File › Plug-in Manager…**, choose Dive Lens, click **Enable**, then **Done**. You only do this once.

**"Lightroom not connected".** Open Lightroom Classic, check in **File › Plug-in Manager** that Dive Lens is enabled, then choose **Library › Plug-in Extras › Open Dive Lens…** again.

**The app does not open (Mac).** If macOS says it cannot check the app, download Dive Lens again from its release page: published versions are signed and checked by Apple, and open without this message.

**Switching from MacDive to Subsurface.** Nothing to migrate. Running a dive again finds its collection when the name "<number> - <site>" is the same; otherwise a new collection is created next to it. Clock offsets are remembered separately for each application.

**Reporting a problem.** Set **Settings › Diagnostics › Log level** to **Detailed**, reproduce the problem, then **Show log**: the folder holds the two log files. Then [open an issue](https://github.com/Ventrilok/divelens/issues/new/choose) describing the problem, without attaching the log files: the GitHub repository is public, and the logs contain your dive sites and file paths. If they are needed, you will be asked for them: send them only by e-mail to [divelens@querton.com](mailto:divelens@querton.com), with the issue number.

---

## 12. Privacy

- Your dive log is only read, never modified. MacDive or Subsurface can stay open meanwhile.
- Dive Lens sends no dive data anywhere. It goes on the Internet for two things only:
  - the site map: the map service (OpenFreeMap, OpenStreetMap data) receives the area of the map displayed, nothing else. Without Internet, the map is replaced by the message "Map unavailable without Internet.";
  - updates: GitHub receives the request for the latest version, and the download if you install, nothing else. **Settings › Updates › Check automatically** turns off the check at launch.
