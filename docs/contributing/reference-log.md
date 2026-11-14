# The reference log

Every dive-log reader in Dive Lens reads the same ten fictional dives, and
must give the same values for them. This page describes those dives and the
traps they hold, and explains how to give a new reader its own copy.

All of it is fictional: invented sites, invented dives. Never add to the
repository a real dive log as it is, or anything copied from one without
anonymising it.

## Where things are

| What | Where |
|---|---|
| Expected values, written once for every reader | `core/src/conformance/reference.rs` |
| Common tests, run against each reader | `core/src/conformance.rs` |
| Each application's description, and the list of applications | `core/src/<application>/app.rs`, `core/src/apps.rs` |
| MacDive copy (built from SQL when the tests run) | `core/src/macdive/tests.rs` |
| Subsurface copy, XML file | `core/fixtures/subsurface/fictional.xml` |
| Subsurface copy, git storage (turned into a repository by the tests) | `core/fixtures/subsurface/git-tree/` |
| Example copy, for the example reader (built from SQL when the tests run) | `core/src/skeleton/tests.rs` |

Change a dive here, in the expected values and in every copy together:
`cargo test -p divelens-core` fails until they agree.

## What every reader must do

The common tests read each copy and check that the reader:

- gives **wall-clock times** exactly as the log holds them: the start and end
  the app receives are the literal times below, with no time zone, and the
  end is the start plus the duration. Nothing depends on the computer's time
  zone: on macOS and Linux the tests run a second time, 14 hours ahead of
  UTC;
- gives **stable identifiers**: the same id for each dive and each site when
  the log is read again or opened again; no two dives share an id (even when
  they start at the same second); dives at the same site share its id;
- gives **metric units**: depths in metres, temperatures in °C, matching the
  values below to within 0.05;
- gives **profiles sorted by time**, going through the points listed below
  (between two points the samples may be anything: each application records
  at its own rate);
- reports a **dive without a profile** as having none, with no samples at
  all, not an empty or made-up profile;
- **never modifies the log**: every file and folder next to it is compared
  before and after reading, and checked while the log is open; nothing may
  be changed, created or removed, including the temporary files a database
  may create on its own. The one write left is outside the tests: while
  MacDive itself is open, SQLite updates its shared index (`-shm`), as it
  does for any reader; the database and its journal (`-wal`) never change;
- reports an unknown dive id as an error;
- leaves out what the reader declares it cannot provide (temperatures, rating,
  GPS, profile).

Values are always what the dive application itself shows. When an
application stores something differently (in feet, or splitting names
another way), the reader translates it so that the result matches.

## The dives

Times are wall-clock times, as the dive computer showed them. "Low" and
"high" are the dive's lowest and highest water temperatures.

| No. | Start | End | Duration | Max depth | Low / high | Rating | Site | Profile |
|---|---|---|---|---|---|---|---|---|
| 100 | 2024-12-31 23:40:00 | 2025-01-01 00:12:00 | 32:00 | 14.2 m | 21.0 / 21.0 °C | 3 | Coral Garden | none |
| 101 | 2025-06-14 09:30:00 | 2025-06-14 10:12:30 | 42:30 | 18.4 m | 24.0 / 26.0 °C | 4 | Coral Garden | 10 points |
| 102 | 2025-06-14 14:00:00 | 2025-06-14 14:35:00 | 35:00 | 12.0 m | 22.0 / 23.0 °C | none | Glacier Wall | 4 points |
| 103 | 2025-06-15 08:00:00 | 2025-06-15 08:50:00 | 50:00 | 20.0 m | 27.0 / 27.0 °C | 2 | Shark Point / Bay | none |
| 104 | 2025-06-15 11:00:00 | 2025-06-15 11:30:00 | 30:00 | 15.0 m | none | none | Lemuria / Sunken City / Temple | none |
| none | 2025-06-16 07:00:00 | 2025-06-16 07:12:00 | 12:00 | 9.0 m | −1.5 / 0.0 °C | none | none | 3 points |
| 106 | 2025-06-17 10:00:00 | 2025-06-17 10:25:00 | 25:00 | 8.0 m | 28.0 / 28.5 °C | none | Pearl Bank | 2 points |
| 107 | 2025-06-17 10:00:00 | 2025-06-17 10:20:00 | 20:00 | 7.0 m | 28.0 / 28.0 °C | 5 | Pearl Bank | 2 points |
| 108 | 2025-06-18 09:00:00 | 2025-06-18 10:04:00 | 64:00 | 31.2 m | 25.0 / 27.0 °C | 3 | Récif d'Émeraude | 7 points |
| 109 | 2025-06-18 15:30:00 | 2025-06-18 15:54:00 | 24:00 | 16.0 m | 25.0 / 26.0 °C | none | (GPS only) | 4 points |

What each dive is there to catch:

- **100: night dive across midnight and the new year.** A time-zone shift
  would move it to another day and year. It was recorded by a dive computer,
  but the log holds no profile for it. Notes: "Night dive, lobsters
  everywhere".
- **101: everything filled in.** Notes on two lines, with characters a file
  format must escape: `Turtle at the wall & a "reef" shark.` then `Second
  line`. Its profile starts 10 s into the dive (as Shearwater computers
  record it) and goes on past the duration the application shows.
- **102: two dive computers** (Subsurface). The first one, the one the
  application shows, gives the maximum depth, temperatures and profile; the
  second one's values (12.6 m, 21 °C) must not appear. No water temperature
  on the first computer: low and high come from its profile. Unrated (MacDive
  stores rating 0), blank notes.
- **103: entered by hand.** No profile, even though Subsurface stores a few
  hand-drawn points for it. Two-line notes: `Entered` then `by hand`.
- **104: no profile and no temperature.** Imported from MacDive into
  Subsurface ("No Computer"); in MacDive, a raw download from a dive computer
  libdivecomputer does not know.
- **The dive without number: no number, no site, 0 °C and below.** Only
  this dive has no number: the tests tell the dives apart by their numbers.
  Subsurface marks it invalid; it is still listed like the others. Low −1.5,
  high exactly 0.0 (zero is a temperature, not a missing value).
- **106 and 107: two dives starting at the same second**, at the same site.
  Each needs its own id, the same every time.
- **108: over an hour, accents and an apostrophe** in the site, its place,
  its country and the notes (`Manta ray at 30 m — café afterwards.`).
- **109: a site with a GPS position and no name.** The dive keeps the
  position; the list of sites leaves the site out.

### Profiles

Points are (time into the dive, depth, temperature). Every time is a
multiple of 10 s, so a copy can come from a dive computer that records every
10 s.

- **101**: 0:10 2.0 m 26.0 °C · 2:00 10.0 m 25.0 °C · 5:00 18.4 m 24.0 °C ·
  10:00 18.0 m 24.0 °C · 20:00 15.0 m 24.5 °C · 30:00 10.0 m 24.5 °C ·
  38:00 5.0 m 25.5 °C · 41:00 5.0 m 25.5 °C · 42:30 0.5 m 25.5 °C ·
  43:00 0.0 m 25.5 °C
- **102**: 0:20 3.0 m 23.0 °C · 10:00 12.0 m 22.0 °C · 25:00 9.0 m 22.5 °C ·
  35:00 0.0 m 22.5 °C
- **Without number**: 0:10 1.0 m 0.0 °C · 6:00 9.0 m −1.5 °C · 12:00 0.5 m
  −1.5 °C
- **106**: 0:20 2.0 m 28.5 °C · 25:00 0.5 m 28.5 °C
- **107**: 0:10 2.0 m 28.0 °C · 20:00 0.5 m 28.0 °C
- **108**: 0:10 3.0 m 27.0 °C · 5:00 25.0 m 26.0 °C · 10:00 31.2 m 25.0 °C ·
  30:00 22.0 m 25.0 °C · 50:00 12.0 m 26.0 °C · 60:00 5.0 m 26.5 °C ·
  64:00 0.0 m 26.5 °C
- **109**: 0:10 2.0 m 26.0 °C · 8:00 16.0 m 25.0 °C · 20:00 5.0 m 25.0 °C ·
  24:00 0.0 m 25.5 °C

## The sites

| Site | Place | Country | GPS | Dives |
|---|---|---|---|---|
| Coral Garden | North Reef | Atlantis | 12.5, −45.25 | 100, 101 |
| Glacier Wall | Frostville | Atlantis | −64.125, −60.5 | 102 |
| Shark Point / Bay | none | none | none | 103 |
| Lemuria / Sunken City / Temple | none | Lemuria | −8.25, 115.5 | 104 |
| Pearl Bank | Oyster Town | none | none | 106, 107 |
| Blue Lagoon | none | none | 1.25, 2.5 | no dive |
| Récif d'Émeraude | Île Verte | Atlantide | −20.5, 57.25 | 108 |
| (no name) | none | none | 25.0, −77.5 | 109 |

The list of sites holds the seven named ones, including Blue Lagoon, which
has no dive yet. Traps: sites without a country (Shark Point, Pearl Bank,
Blue Lagoon), without GPS (Shark Point, Pearl Bank), without a place
(Lemuria, Blue Lagoon); a "/" in a name that must stay whole (Shark Point /
Bay has two parts; Lemuria / Sunken City / Temple has a country of its own).

## How each application stores the traps

**MacDive** (SQLite database):

- dates are seconds since 2001-01-01 00:00, holding the wall clock;
- profiles come from the dive computer's raw download, decoded by
  libdivecomputer. The copy stores them as Sporasub SP2 downloads, the
  simplest format libdivecomputer decodes to 1 cm and 0.1 °C: the device
  does not matter, the samples do;
- the database is in WAL mode, like every MacDive database, so that reading
  it must not leave its `-wal` and `-shm` files behind;
- dive 100 names a dive computer libdivecomputer knows, without a raw
  download; 103 has no computer at all; 104 has a raw download from a
  computer libdivecomputer does not know;
- GPS 0/0 means no position (Pearl Bank), rating 0 means unrated (102),
  blank fields mean nothing (the site of the dive without number, the
  location of Lemuria, the notes of 102), notes use Windows line breaks
  (101);
- the dive without number has no UUID either: its id falls back to the row
  number;
- MacDive has no "invalid" dives.

MacDive's own tests add two dives the reference log leaves out: a manual dive
with nothing but a date, and a raw download libdivecomputer cannot decode
(an error, not an empty profile).

**Subsurface** (XML file and git storage, the same dives in both):

- "Country / Place / Site" names are split only when Subsurface has no
  country or city for the site and the name has exactly three parts (Coral
  Garden, Récif d'Émeraude); otherwise the name stays whole;
- a site's country and city come from Subsurface's own fields when it has
  them (Glacier Wall; Pearl Bank's place is a "town");
- low temperature is the water temperature, else the coldest point; high is
  the warmest point, else the water temperature; never the air temperature
  (101 has an air temperature of 29 °C);
- "manually added dive" (103) and "No Computer" (104) have no profile; 100
  has a dive computer with no samples;
- a point without temperature keeps the previous one, as Subsurface writes
  them;
- 101 and 102 are in a trip whose notes never show on a dive;
- dives have no id in Subsurface: Dive Lens uses the start time, with `#2`
  for the second dive starting at the same second (107).

**Example** (the example reader, `core/src/skeleton.rs`: an invented
SQLite database that no real application writes):

- times are wall-clock text, `YYYY-MM-DD HH:MM:SS`; depths in metres and
  temperatures in °C;
- rating 0 means unrated (102 and the dives without rating), blank fields
  mean nothing (the place of Lemuria, the notes of 102), notes use Windows
  line breaks (101);
- a dive without samples has no profile (100, 103, 104); samples are stored
  last first, so that the reader must sort them;
- the database is in WAL mode, closed, so that reading it must not leave
  `-wal` and `-shm` files behind;
- Example has no "invalid" dives.

Example's own tests add a site the reference log leaves out, because
Subsurface cannot store it: a latitude without a longitude, which is no
position at all.

## Adding a reader

An application is a reader plus its description. The description
(`DiveApp`, in `core/src/apps.rs`) is everything the app needs besides the
reader: the name written on photos, the systems it runs on, its standard
location, whether the diver chooses a file or a folder, its unit
preferences, its clock-offsets file, and every text the screens show for
it, in English and French. The app builds Settings, the file chooser and
the dive-log error screen from it; nothing else in the app or in the
screens names an application. Before starting, open a "Support a new dive
application" issue so that the approach can be discussed.

1. Start from a copy of the example reader: `core/src/skeleton.rs`,
   `core/src/skeleton/app.rs` and `core/src/skeleton/tests.rs` become
   `core/src/<application>.rs`, `core/src/<application>/app.rs` and
   `core/src/<application>/tests.rs`. Each place to change is marked
   `ADAPT:`. The example reads a SQLite log, opened by the shared
   `crate::sqlite::open_read_only`; for files and folders, see the
   Subsurface reader.
2. Write the reader in `core/src/<application>.rs`: it implements
   `DiveSource` (`core/src/model.rs`) and has an `open(path)` that reads the
   log without writing anything. Its module holds, besides the reader:

   ```rust
   mod app;

   pub use app::APP;

   #[cfg(test)]
   pub(crate) mod tests;
   ```

   and `core/src/lib.rs` declares it as `pub mod <application>;` (the
   example reader is declared `#[cfg(test)]`: only the tests build it).
3. Write the reference log in the application's own format, from the tables
   above, shaped exactly like what the application itself writes. Files go
   in `core/fixtures/<application>/`, but files ending `.sqlite` or `.ssrf`
   are ignored by git there, as a guard against real logs: build a database
   from SQL in the tests, as MacDive's copy is, and name a Subsurface-style
   XML copy `.xml`. If the application's logs end with another extension,
   add it to `.gitignore` the same way, so that a real log cannot be
   committed by mistake. In the reader's tests, add a function that writes the
   copy into a folder and returns the path to open, and a `conformance()`
   function giving the application's runs of the common tests: one per way
   the log can be stored, each with that function, the capabilities the
   reader must declare, and whether the application marks dives invalid.
4. Describe the application in `core/src/<application>/app.rs`: a
   `pub static APP: DiveApp` with its id (camelCase, kept by settings files
   and never changed), its name, its texts in both languages, and its runs
   (`conformance: super::tests::conformance`). The runs exist only in the
   tests' build, so keep the `#[cfg(test)]` above them; there, the compiler
   refuses a description without them.
5. Add `APP` to `ALL` in `core/src/apps.rs`: with the `pub mod` line of step
   2, this registers the application. Its place in the list is the order of
   Settings, and the first application available on a system is the default
   there.
6. Run `cargo test -p divelens-core` until it passes. The common tests run
   every registered application, opened as the app opens it; a failure
   lists every value that differs from this page. Other tests check the
   description is complete: every text present in English and in French,
   none empty, the same placeholders in both languages, an id, name and
   offsets file no other application has.
7. Check the reader against a real log on your own computer, with the tool
   below: it reads every registered application, nothing to add there.

## Checking a reader against a real log

```
cargo run --release -p divelens-core --example verify_log -- [--app <application>] <log> [dive number]
```

`<log>` is the log of any registered application, for example the MacDive
database, a Subsurface XML file or a Subsurface git storage folder (`--help`
lists them all). `--app` takes the application's name. Without it, the tool
tries the applications whose logs look like `<log>`, in the order of the
list: those whose log can be a folder, for a folder (Subsurface); those whose
files have its extension, for a file (`.xml` or `.ssrf`: Subsurface;
`.sqlite`: MacDive); the first registered application (MacDive) for anything
else. The first one that reads the log is used; when none does, the first
one's error is shown. The tool prints the number of
dives and sites, decodes every profile (listing those that fail), reads the
log twice to compare identifiers, reports what breaks the rules above, and
says whether reading changed, created or removed any file of the log (sizes
and hashes only, never contents). With a dive number, it also prints that
dive and its profile.

**Its output is personal data**: dates, places, positions, notes. It stays on
your computer. Never put it, even in part, in the repository, an issue or a
pull request.
