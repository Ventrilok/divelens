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
- **Help › Plug-in Extras › Dive Lens Settings…** opens it on **Settings** (section 8).

The Dive Lens entry in **File › Plug-in Manager…** also has **Open Dive Lens** and **Open Settings** buttons, and links to the Dive Lens website and to its source code on GitHub.

If the app is already open, the menu brings its window to the front. Chosen during the journey, **Open Dive Lens…** takes you back to step 1 and keeps your choices; chosen on the summary, it starts a new run.

Lightroom must stay open while you use the app. The first time, check **Settings** (section 8): the dive application and the location of your log.

### The window

- **At the top**: Dive Lens on the left; on the right, the gear opens **Settings** (section 8). The gear is greyed while Dive Lens writes to Lightroom.
- **At the bottom**, the status bar. On the left, Lightroom:
  - **Lightroom connected**, **Lightroom not connected**, or **Restart Lightroom**;
  - **Catalog:** the name of the catalog open in Lightroom;
  - **Catalog size:** how many photos it holds, counted again each time you come back to step 1;
  - **Selected photos:** how many are selected in Lightroom, live (**none** when none is).
- On the right of the status bar, your dive log: **Dive log app:** MacDive or Subsurface, and **Dives logged:** how many dives it holds.

If the status bar says **Lightroom not connected**, nothing can be written: check that Lightroom is open and the plug-in is enabled in the Plug-in Manager. If it says **Restart Lightroom**, Dive Lens was updated while Lightroom was open: quit Lightroom and start it again.

---

## 3. The journey

Five steps, shown at the top: **Photos › Dives › Clocks › Review & Apply › Summary**. **Next** moves on and **Back** goes back; a click on a step already done goes back to it too, except from the summary. Under the steps, a line starting with **Photos:** recalls what you chose; at step 2, its **Change** takes you back to step 1. Nothing is written in Lightroom before you click **Apply to N photos** at step 4.

### Two paths

Step 1 asks the one real question: which photos?

- **Entire catalog**: you tick one or more dives, and Dive Lens looks in the whole catalog for the photos taken during each one. Each photo goes to the dive that matches its time. For a whole trip or a series of dives. The photos selected in Lightroom are ignored.
- **Lightroom selection**: the photos you selected in Lightroom. You choose one dive, and all of them are attached to it, whatever their time. For photos without a reliable time, to attach photos to a dive by hand, or to give them only a location (section 6).

Choosing the other path at step 1 starts the next steps again: dives, clocks and options.

### Step 1 — Photos

"Which photos to work on?" Two cards: **Entire catalog**, or **Lightroom selection**. The selection card follows Lightroom live: the cameras and the day of the selected photos, and up to six thumbnails. With no photo selected, it is greyed and says **No photos selected**: select photos in Lightroom and it lights up. Choose a card, then **Next**.

### Step 2 — Dives

The list shows every dive of your log, grouped by month. Search by number, site, location or country, and narrow down with the filters (**Period**, **Country**, **Depth**, **Rating**, **With profile**). **Sort** orders the list (**Newest first**, **Oldest first**, **Max depth**); Dive Lens remembers your choice. Click a dive to see its details on the right: profile, duration, max depth, temperatures, rating, site GPS with a map, and your notes.

- **Entire catalog**: tick one or more dives, then **Next**. The bar shows how many are ticked and their days; **Deselect all** unticks them.
- **Lightroom selection**: click one dive, then **Next**.
  - The dive whose times cover the selection carries the **Covers the selection** badge. It uses the last offset applied to each camera, then the offsets you set at step 3. It is only a hint: you choose.
  - The first row, **No dive — location only**, is for giving the photos only a location (section 6).

A dive **without a profile** (entered by hand, or from an older dive computer whose download MacDive did not keep) still works: its photos get the dive data and the site, but no depth, temperature or time.

### Step 3 — Clocks

**Camera clocks.** A camera's clock is rarely exactly on time with the dive computer, and is often still on the time of home. Each camera has a card (two bodies of the same model are told apart by serial number). Set the difference between its clock and the dive log's:

- **Suggest** proposes the offset that puts the most photos in the dives. It tries every quarter of an hour between −14 h and +14 h, so it also finds a forgotten time zone.
- **Suggest for all** does it for every camera at once, without touching cameras you set by hand or ignore.
- You can also type the offset (hours:minutes) or adjust it yourself.
- Each card starts from the last offset applied to that camera, on either path (**remembered offset**); with none, it starts at 0:00. Offsets are remembered separately for each dive application.

**Preview by dive.** The counts follow your settings: photos per dive, cameras, and the per-photo data each dive gives (**Depth · temp. · time**, **No profile: dive data only**, **Surface photos only**). A dive with no photo is shown in orange, **No photo — clock changed?**: a camera clock was often changed during the trip (section 10).

On the **Entire catalog** path:

- Dive Lens looks in the whole catalog for the photos taken over the period of the dives ticked.
- **Ignore** leaves out a camera, for example a phone that stayed on the boat: its photos are neither counted nor tagged.
- **Include photos taken at the surface**: up to N minutes before and after each dive, for photos taken on the boat just before going in or just after getting out. 0 minutes by default, and back to 0 on every run. These photos get the dive data, without depth.
- **Outside the dives**: the photos of the period that fall in no dive, even with the margin (on land, on the boat…). They get nothing (section 6).

On the **Lightroom selection** path:

- The photos are attached to the chosen dive whatever their time. The clock only gives them a depth: only those taken during the dive get one.
- One card per camera of the selection. There is no **Ignore** (remove a photo from the selection in Lightroom instead) and no surface margin.
- **Outside the dive's times** lists the photos taken before or after the dive, for example "Taken at 15:45, 7 minutes after getting out of the water. Attached to the dive anyway: it gets its data and its location, without depth."
- A dive without a profile shows "No profile: no per-photo data" instead of the camera clocks, and a dive whose profile cannot be read "Profile could not be read: no per-photo data": the dive data are then written without per-photo data.
- With **No dive — location only**, this step is skipped: **Clocks** is greyed in the steps.

### Step 4 — Review & Apply

"What Dive Lens writes in Lightroom." The same options on both paths. They are all ticked on every run, except **Copy depth to GPS altitude**, which is unticked on every run: nothing is remembered from one run to the next. On the right of each option, its effect on this run, for example "12 photos · 9 with depth".

**Metadata**

- **Dive details**: dive number, start, end, duration, max depth, temperatures, rating and source (the dive application). On the photos taken during the dive, also the depth, temperature and time into the dive, and the clock offset.
  - **Copy depth to GPS altitude**, under it, greyed when **Dive details** is unticked: also writes the depth in the GPS altitude, as a negative value, in metres, on the photos that have a depth (section 5).
- **Site details**: the location and GPS of the dive's site, or of another site of your log.
  - **Entire catalog**: **Each dive's site**, or **Other site, the same for all photos…**.
  - **Lightroom selection**: **Dive's site: <name>**, or **Other site…**.
  - **Other site**: chosen by country › location › site among the sites of your log. Its GPS can be edited before applying, in decimal degrees (for example `12.5000` and `-45.2500`). The collections keep the name of their dive's own site.
  - **Location**: country, city and location. **GPS**: the position. Untick one to write only the other. A site without GPS leaves the GPS already on the photo in place.

**Collections**

- **Create collections** (an option of this step, no longer a Lightroom menu): on the **Entire catalog** path, one "<number> - <site>" collection per dive that has photos; on the **Lightroom selection** path, the chosen dive's collection, which gets all the selected photos. A photo already in the collection is not added twice. Where the collections go is set in Settings: the link **Place in Settings** opens them (section 8).

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

**Written in Lightroom**: **Photos tagged** (the photos written, those only put in a collection included), **With depth**, and **Collections**, then each collection with its number of photos, **created** or **added to**. Then the warnings (dives without photo, photos outside the dives, photos without depth) and the reminder of how to undo (section 7).

- **Entire catalog**: **Back to dives** returns to step 2 with no dive ticked. When a dive got no photo, **Handle dive N separately** (or **Handle these dives separately**) goes straight to step 3 with only those dives ticked, and looks for their photos again: give their camera its own offset (section 10).
- **Lightroom selection**: **Done** returns to step 1.

---

## 4. What is written

- **Create collections**: one collection per dive, named **"<dive number> - <site>"**, in the **Dive Lens** collection set (the place can be changed in Settings). A dive without a number gets its date in place of the number.
- **Dive details**: on each photo, dive number, start and end, duration, max depth, min and max temperatures, rating and the dive application the data come from. On the photos taken during a dive that has a profile, also the photo's **depth, temperature and time into the dive**, estimated between the two profile points around the moment of the photo. And the clock offset applied: on every photo placed in a dive (**Entire catalog**), on the photos taken during the dive (**Lightroom selection**).
- **Site details**: country, city, location and GPS, in Lightroom's standard fields.

Good to know, on both paths:

- The photo's time is its **original capture time**, so a TIFF that came back from Photoshop is matched like its original.
- A site without country, city or GPS leaves the values already on the photo untouched (a GPS position from a phone stays).
- With **Dive details** unticked, Dive Lens does not touch the dive fields at all; the photos are still attached to the dives for the site and the collections.
- The units chosen in Settings apply to the values written. Photos already tagged keep the units in use when they were written.

On the **Entire catalog** path:

- Only the photos placed in a dive are written, in the dive or in the surface margin. Ignored cameras and photos outside the dives are not touched.
- A photo in the surface margin gets the dive data, the location and the clock offset, without per-photo data.
- A photo that would fall in two dives goes to the closer one.
- Running a group of dives again only adds: photos already filed stay where they are (correct them by hand if needed), and photos found again get their values rewritten.
- A dive whose profile cannot be read stops **Apply**: nothing is written.

On the **Lightroom selection** path:

- Every photo selected at the moment of the click is written.
- A photo taken outside the dive's times gets the dive data and the location, without per-photo data or clock offset.
- A value the chosen dive does not have (temperature, rating…) clears the old Dive Lens value on the photo, so the fields always describe a single dive.
- With **Dive details**, a photo that already carries a **different** dive loses that dive's own values: depth, temperature, time into the dive and clock offset, replaced by the new ones when the photo was taken during the new dive. Its GPS altitude follows section 5. The photo stays in the collection of its old dive: move it yourself if needed.
- Tagging a photo with the dive it already carries rewrites its per-photo data when it was taken during the dive, with the offset shown; the other photos keep theirs.
- If the selection changes in Lightroom just as you click **Apply** while per-photo data are written, nothing is written: "The selection changed in Lightroom. Nothing was written: check the per-photo data, then apply again."
- A profile that cannot be read never blocks: the dive data are written without per-photo data.

After **Apply**, Dive Lens remembers the offset of each camera for next time, on both paths: on the **Entire catalog** path, the cameras with a photo placed in a dive; on the **Lightroom selection** path, the cameras with a photo taken during the dive, when per-photo data are written.

---

## 5. Depth in the GPS altitude

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

## 6. Location only, and photos outside the dives

On the **Entire catalog** path, photos taken outside any dive (on the boat, on the beach, while snorkelling) get nothing: step 3 shows them under **Outside the dives**, and so do the summary and **Good to know before applying**.

To give them a location, or to attach them to a dive anyway:

1. Select them in Lightroom, then **Library › Plug-in Extras › Open Dive Lens…** and choose the **Lightroom selection** card.
2. At step 2:
   - to attach them to a dive anyway, choose the dive: they get its data and its location, without depth;
   - to give them only a location, choose **No dive — location only**.

With **No dive — location only**, step 3 is skipped. At step 4, only **Site details** acts: **Dive details** and **Create collections** are greyed, and the site is **Other site…**. Choose the site by country › location › site, adjust its GPS if needed, then **Apply to N photos**. Only the country, city, location and GPS are written: no dive field and no collection.

---

## 7. Undo

**Edit › Undo** in Lightroom reverts everything one **Apply** wrote, in one step, GPS and altitude included, on both paths. Lightroom's Edit menu names it **Undo Dive Lens**.

- A collection created by **Apply** goes away with it; photos added to an existing collection leave it.
- **The first time**, when the collection set (**Dive Lens** by default) does not exist yet, Dive Lens creates it on its own, just before writing. Lightroom keeps that as a separate step, named after the set: the first Undo takes back what was written, and a second one removes the set, now empty. If the write is refused after the set was created, the empty set stays: undo it, or delete it in Lightroom.
- If Lightroom did not let Dive Lens write the GPS and altitude, everything else is written and Dive Lens says so: choose **Edit › Undo Dive Lens** in Lightroom first, then apply again.

---

## 8. Settings

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
- **Depth** and **Temperature**: **As MacDive** / **As Subsurface** (the units chosen in that application), or forced to metres / feet and °C / °F. They apply in the app and to the values written on photos. The GPS altitude is always in metres (section 5).

**Collections**
- **Place in**: the catalog root or an existing collection set (Lightroom must be connected to browse them). This is where the **Create collections** option of step 4 puts the collections.
- **Group in the set**: "Dive Lens" by default, created when needed; leave it empty to put the collections straight in the place above. A preview shows the resulting path. Collections already created stay where they are.
- If the set chosen in **Place in** is no longer in the catalog, step 4 says so while **Create collections** is ticked, and **Apply** stays greyed until you choose another place here.

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

## 9. The Dive Lens fields in Lightroom

In the **Library** module, open the **Metadata** panel and choose **Dive Lens** in its menu. The fields are in four groups:

- **Photo**: Depth, Temperature, Elapsed Time;
- **Dive**: Dive, Start Dive, End Dive, Duration, Max Depth, Temp. Low, Temp. High, Rating;
- **Site**: Lightroom's location, city, country and GPS;
- **Technical**: Clock Offset, Source.

The Dive Lens fields stay in the Lightroom catalog: they are not written into the photo files and do not travel with exports. Only the GPS position, the location fields and, if you choose it, the altitude do.

---

## 10. Troubleshooting

**A dive has no photo.** Check the camera clock at step 3: use **Suggest**, or set the difference by hand. If a camera clock or the dive computer's clock was changed during the trip, run the days before and after the change as two separate groups, each with its own offset: **Handle dive N separately** on the summary does this for the dives left without photos.

**"MacDive database not found".** The database may have moved, for example to iCloud: use **Locate the database…** or **Settings › Choose…**. If it is in iCloud, check that it is downloaded on this Mac.

**"The dive log has changed…".** The dive log in use changed (in Settings, for example) while Dive Lens was working with the previous one, so the action was refused. After **Apply**, the orange notice reads "The dive log has changed. Nothing was written: start again." After an action that writes nothing (finding the photos, showing a profile), it reads "The dive log has changed: start again." It shows at step 1, under the steps, until you click **Close** or **Next**. Choose the dives again: they now come from the new log.

**Dive Lens is disabled after installing.** Lightroom may show Dive Lens as disabled, in particular when a previous Dive Lens plug-in had been disabled before it was removed. In **File › Plug-in Manager…**, choose Dive Lens, click **Enable**, then **Done**. You only do this once.

**"Lightroom not connected".** Open Lightroom Classic, check in **File › Plug-in Manager** that Dive Lens is enabled, then choose **Library › Plug-in Extras › Open Dive Lens…** again.

**The app does not open (Mac).** If macOS says it cannot check the app, download Dive Lens again from its release page: published versions are signed and checked by Apple, and open without this message.

**Switching from MacDive to Subsurface.** Nothing to migrate. Running a dive again finds its collection when the name "<number> - <site>" is the same; otherwise a new collection is created next to it. Clock offsets are remembered separately for each application.

**Reporting a problem.** Set **Settings › Diagnostics › Log level** to **Detailed**, reproduce the problem, then **Show log**: the folder holds the two log files. Then [open an issue](https://github.com/Ventrilok/divelens/issues/new/choose) describing the problem, without attaching the log files: the GitHub repository is public, and the logs contain your dive sites and file paths. If they are needed, you will be asked for them: send them only by e-mail to [divelens@querton.com](mailto:divelens@querton.com), with the issue number.

---

## 11. Privacy

- Your dive log is only read, never modified. MacDive or Subsurface can stay open meanwhile.
- Dive Lens sends no dive data anywhere. It goes on the Internet for two things only:
  - the site map: the map service (OpenFreeMap, OpenStreetMap data) receives the area of the map displayed, nothing else. Without Internet, the map is replaced by the message "Map unavailable without Internet.";
  - updates: GitHub receives the request for the latest version, and the download if you install, nothing else. **Settings › Updates › Check automatically** turns off the check at launch.
