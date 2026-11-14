# Contributing to Dive Lens

Thank you for your interest in Dive Lens. It links underwater photos in Adobe Lightroom Classic with dive-log data. It is free and open source, under the [GNU General Public License, version 3 or later](LICENSE) (GPL-3.0-or-later): by contributing, you agree that your contribution is published under GPL-3.0-or-later.

Bug reports, requests to support another dive application, fixes and documentation are all welcome.

**One rule before anything else: this repository is public.** Everything you put in it, in an issue or in a pull request, stays visible to everyone, including in the history. Never share a real dive log, a dive-log database, what a tool prints from one, the Dive Lens log files, or an exported Dive Lens settings file here. See [Test data: fictional or anonymised](#test-data-fictional-or-anonymised). When you are asked for the log files, or send a log to test a new dive application, they go privately by e-mail to [divelens@querton.com](mailto:divelens@querton.com), with the issue number: see [Reporting a bug or asking for a dive application](#reporting-a-bug-or-asking-for-a-dive-application).

## Contents

- [How Dive Lens is built](#how-dive-lens-is-built)
- [Getting set up](#getting-set-up)
- [Commands](#commands)
- [Trying the plug-in in Lightroom](#trying-the-plug-in-in-lightroom)
- [Checks before you commit](#checks-before-you-commit)
- [Branches, commits and pull requests](#branches-commits-and-pull-requests)
- [Version numbers](#version-numbers)
- [Test data: fictional or anonymised](#test-data-fictional-or-anonymised)
- [Secrets](#secrets)
- [What must never be renamed](#what-must-never-be-renamed)
- [The libdivecomputer submodule](#the-libdivecomputer-submodule)
- [Reporting a bug or asking for a dive application](#reporting-a-bug-or-asking-for-a-dive-application)

## How Dive Lens is built

Dive Lens has two parts that ship together. A **Lightroom Classic plug-in**, written in Lua, stays invisible: it declares the Dive Lens metadata fields, reads and writes the catalog, follows the photo selection, provides thumbnails and adds two Library menu items that only open the app on the matching screen. A **desktop app**, written in Rust with Tauri (and a React + TypeScript front end), is the only user interface. The app contains one *reader* per dive-log application (MacDive and Subsurface for now), each translating its data into a common dive format; dive logs are always opened read-only. The two parts talk over a local connection: the app follows the Lightroom selection live, and writes to the catalog only when the user clicks **Apply**, in a single write that Lightroom can undo in one step. In the released package, the app sits inside the plug-in folder.

The product specification is [`docs/spec-v1.md`](docs/spec-v1.md) (in French). It is the reference for scope and decisions; each item is tagged *Décidé* (decided), *Proposé* (proposed) or *À trancher* (to be decided). The user guide is in [`docs/user/`](docs/user/), in English and French.

Repository layout:

| Folder | What it holds |
|---|---|
| `core/` | Rust library `divelens-core`: the common dive format, the readers, the per-photo computations. No Tauri dependency, tested on macOS and Windows. |
| `libdivecomputer-sys/` | Builds [libdivecomputer](https://www.libdivecomputer.org) (C) and declares the few functions used to decode dive-computer data. Parsing only: no USB, Bluetooth or serial. |
| `third_party/libdivecomputer/` | libdivecomputer itself, as a git submodule pinned to a validated commit. |
| `app/` | The Tauri 2 app: front end in `app/src`, Rust side in `app/src-tauri` (crate `divelens`). |
| `plugin/DiveLens.lrdevplugin/` | The Lightroom plug-in, shipped as plain Lua source (no compile step). |
| `docs/` | Specification and user guide. |

The three Rust crates form one Cargo workspace at the root, with a single `target/` folder and a single `Cargo.lock`.

## Getting set up

You need:

- **Git**.
- **Rust**, stable toolchain, installed with [rustup](https://rustup.rs). The workspace uses the 2024 edition, so keep your toolchain up to date (`rustup update`).
- **Node.js**, LTS version, with npm.
- **The build tools for your system** that Tauri needs (on macOS the Xcode Command Line Tools; on Windows the Microsoft C++ Build Tools and WebView2): follow [Tauri's prerequisites](https://v2.tauri.app/start/prerequisites/). They also compile libdivecomputer's C code.
- **Lightroom Classic**, only to try the plug-in.

You do not need to install Lua: the tests bring their own Lua 5.1, the version Lightroom embeds.

Then:

```sh
git clone --recursive https://github.com/Ventrilok/divelens.git
cd divelens
# If you cloned without --recursive:
git submodule update --init

cd app
npm ci
npm run build
```

The core does not build without the libdivecomputer submodule. The first `npm run build` produces `app/dist`, the built front end that the app crate embeds: the app's Rust checks and tests need it.

## Commands

From the repository root:

| What | Command |
|---|---|
| Core tests | `cargo test -p divelens-core` |
| App and plug-in tests | `cargo test -p divelens` (needs `app/dist`) |
| Formatting check | `cargo fmt --all --check` (`cargo fmt --all` fixes it) |
| Lint, everything | `cargo clippy --workspace --all-targets -- -D warnings` (needs `app/dist`) |
| Lint, core only | `cargo clippy -p divelens-core -p libdivecomputer-sys --all-targets -- -D warnings` |
| Lint, app | `cargo clippy -p divelens --all-targets -- -D warnings` (needs `app/dist`) |

From `app/`:

| What | Command |
|---|---|
| Front-end tests | `npm test` |
| Front-end type check and build | `npm run build` (runs `tsc`, then produces `app/dist`) |
| Dev window | `npm run tauri dev` (opens the app and keeps running, rebuilding as you edit) |
| App build, macOS | `npm run tauri build -- --bundles app` (builds `Dive Lens.app`, skips the disk image) |
| App build, Windows | `npm run tauri build -- --no-bundle` |
| Change the version | `npm run set-version <version>` (see [Version numbers](#version-numbers)) |

The front-end tests run in a UTC+14 time zone on purpose. Dive times are wall-clock times, as the dive computer recorded them, and must never shift with the time zone of the computer running Dive Lens; running far from most people's zone catches any accidental shift.

The plug-in's pure-Lua parts (JSON, message framing, manifest, metadata fields) are tested under Lua 5.1 by `cargo test -p divelens`. The parts that need Lightroom itself are checked by hand in Lightroom.

## Trying the plug-in in Lightroom

1. Build the app once (see above). In development, the plug-in's menus launch the release build of the repository when no app ships in the plug-in folder: `target/release/bundle/macos/Dive Lens.app` on macOS, `target/release/divelens.exe` on Windows.
2. In Lightroom Classic: **File › Plug-in Manager… › Add**, and choose the `plugin/DiveLens.lrdevplugin` folder of your clone.
3. Use **Library › Plug-in Extras › Create Collections…** or **Tag Selection…**.

A dev window started with `npm run tauri dev` connects to Lightroom by itself while it runs.

The plug-in writes its log to `DiveLens.log` in Lightroom's log folder, next to the app's `DiveLens App.log`: `~/Library/Logs/Adobe/Lightroom/LrClassicLogs/` on macOS, `%LOCALAPPDATA%\Adobe\Lightroom\Logs\LrClassicLogs\` on Windows. These logs contain your own data (dive sites, file paths): read them, but never commit them or post them in an issue or a pull request.

Never copy files from the Adobe Lightroom SDK into the repository. The legacy 0.x plug-in's licence code, compiled Lua bytecode and the `luac5.1` binary must not be brought over either.

## Checks before you commit

Run the checks for what you changed, and fix any failure. Never skip, disable or loosen a test to make it pass.

- **Always:** `cargo fmt --all --check`.
- **`core/` or `libdivecomputer-sys/`:** core clippy and core tests.
- **The app, its front end or the plug-in:** from `app/`, `npm test` then `npm run build`; then app clippy and `cargo test -p divelens`.

Continuous integration runs the same checks on **macOS and Windows** for every push to a pull request, and again on merge. A change that touches only documentation (`docs/`, Markdown files, `LICENSE`) skips them.

## Branches, commits and pull requests

**Branches.** Never work on `main`. Use one branch per coherent change, named `<type>/<short-kebab-slug>` with a commit type from the list below, for example `feat/tag-selection`, or `fix/12-empty-profile` when there is an issue (here, issue 12).

**Commits** follow [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/):

```
<type>(<scope>): <description>
```

- The description is imperative ("add", not "added"), lower-case, without a trailing period, and the whole line is at most 72 characters.
- Types: `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `style`, `revert`.
- The scope is optional: `core`, `macdive`, `subsurface`, `app`, `plugin`, `link` (the app ↔ plug-in connection), `spec`, `deps`.
- Examples: `fix(subsurface): read dives without a site`, `docs: describe the settings backup in the user guide`.
- Keep unrelated changes in separate commits, and leave out of a commit anything that does not belong to it.
- Never commit build output or generated files.

**Pull requests.**

- The title follows the same format as a commit message: pull requests are **squash-merged**, so the title becomes the single commit on `main`.
- Fill in the pull request template: summary, checks run, and confirmation that the data is fictional or anonymised.
- **CI must be green on macOS and Windows** before a pull request is merged.
- Anything a diver would notice (a screen, a text, a behaviour, what gets written on photos or in Lightroom) is a product decision. Say in the pull request whether it is already decided in the specification or proposed by your change.
- Once a branch is pushed, don't rewrite its history (no force-push): to catch up with `main`, merge it into your branch. The squash merge keeps `main` tidy anyway.

**Visible text** says **"Dive Lens"**, two words, wherever a person reads it: the app, Lightroom texts, the bundle name, the documentation. Identifiers keep **"DiveLens"** / **"divelens"**: the plug-in identifier and folder, the bundle identifier, translation keys, crate names, log file names.

## Version numbers

**The version of Dive Lens is set in one place: `version` in `app/package.json`.** Everything that shows a version takes it from there: the app bundle, Settings › About, the app log, Lightroom's Plug-in Manager, the zip names `DiveLens-<version>-…` and the release tag `v<version>`, which the release workflow checks.

To change it, from `app/`:

```sh
npm run set-version 2.1.0
```

This sets the version in `package.json` and `package-lock.json` (it is npm's own `npm version`, without a git commit or tag), then copies it to the files that cannot read `package.json`: the plug-in's `Info.lua` (Lightroom reads that file alone, without access to any other file) and the Rust workspace's `Cargo.toml` and `Cargo.lock`. The app reads its version from `package.json` when it is built, through `tauri.conf.json`. Never edit a copy by hand: `cargo test -p divelens` fails as soon as one differs. Commit all the changed files together; once that commit is on `main`, pushing the tag `v2.1.0` builds the release as a draft.

The numbers follow [Semantic Versioning](https://semver.org): **MAJOR.MINOR.PATCH**.

- **MAJOR**: a change that breaks what users rely on, for example a catalog or settings that would need migrating. Version 2 marks the break with the 0.x plug-in: the first public version is 2.0.0, and development versions stay in 0.x until then.
- **MINOR**: new features, compatible with what came before.
- **PATCH**: fixes only.

A **pre-release** adds a label after a hyphen, for example `2.1.0-beta.1`, `2.1.0-beta.2`, then `2.1.0`. Always put a dot before the number (`beta.2`): versions are compared part by part, and only parts made of digits compare as numbers, so `beta10` would come before `beta2`. What the tools do with a label:

- Lightroom's `VERSION` holds three numbers (major, minor, revision) and a free text, `display`, which is what the Plug-in Manager shows: the numbers carry `2.1.0`, the text the whole version.
- macOS: the app bundle carries the whole version (`2.1.0-beta.1`) as both its version and its build number. Apple documents both as numbers only, but Tauri builds the app with the label all the same.
- Windows: the app's numeric file version drops the label (`2.1.0-beta.1` and `2.1.0` both read 2.1.0.0); its product version text keeps it. An MSI installer, which Dive Lens does not build, would accept only a numeric label of at most 65535 (`2.1.0-1`).
- No build metadata (`2.1.0+7`): npm drops it, and the tests refuse it.
- A tag such as `v2.1.0-beta.1` builds a draft release like any other; mark it as a pre-release on GitHub when publishing it.

Three other numbers are **compatibility versions**, not the version of Dive Lens: they never change with it, and each changes only when its own format does.

- **Link protocol**: `PROTOCOL` in `app/src-tauri/src/lightroom.rs` and in `plugin/DiveLens.lrdevplugin/Link.lua`. Raise both together when a message between the app and the plug-in changes in a way the other side would misread: an app and a plug-in with different numbers refuse to connect.
- **Metadata schema**: `schemaVersion` in `plugin/DiveLens.lrdevplugin/Metadata.lua`. Raise it when Dive Lens fields are added or their definitions change, as the Lightroom SDK asks. Each field's own `version` changes only when its definition changes incompatibly, which the compatibility contract rules out for the existing fields.
- **Settings file**: `CURRENT_VERSION` in `app/src-tauri/src/settings.rs`, written into the settings file and into exported settings. Raise it when the layout of the file changes, keeping older files readable (the same goes for the clock-offset files, `CURRENT_VERSION` in `app/src-tauri/src/offsets.rs`).

## Test data: fictional or anonymised

The repository is public, and git history keeps everything forever: a dive log committed by mistake and deleted later is still there. A dive log is personal data. It holds dates, places and GPS positions, often buddy names, notes and the diver's own name, and its file path usually contains a user name.

So:

- Test fixtures are **fictional** (invented dives, sites and people) or properly **anonymised**. The Subsurface fixtures are in `core/fixtures/`; the MacDive test databases are built by the tests themselves, in `core/src/macdive/tests.rs`.
- Never commit, attach or paste:
  - a real dive log or dive-log database (MacDive database, Subsurface file or git storage, or any other application's);
  - output produced from a real dive log;
  - the log files `DiveLens.log` and `DiveLens App.log`;
  - an exported settings file ("Dive Lens settings.json" or "Réglages Dive Lens.json"): it holds the full paths of your dive logs, with your user name, and your collection-set names. If it really helps, paste its content with those replaced;
  - screenshots showing real dive sites, GPS positions, names or file paths (crop or blur them).
- When something only shows up with your own log, check it locally and describe the result in words (for example "dives with two computers lose their profile"), without the data itself.

The `.gitignore` already blocks some dive-log file types and the build output, but it is a safety net, not a guarantee: check what you stage.

## Secrets

Never commit keys, certificates, passwords or tokens. Signing keys and credentials live in GitHub secrets or with the maintainer, never in the repository. If one was committed by mistake, deleting it in a new commit does not remove it from history: report it privately, as described in [SECURITY.md](SECURITY.md), so that it can be revoked. Never mention it in a public issue or pull request. Security vulnerabilities are reported the same way.

## What must never be renamed

People already have Lightroom catalogs written by earlier versions of Dive Lens. To keep them working without any migration, these stay exactly as they are:

- the plug-in identifier `com.querton.divelens`;
- the existing metadata field ids: `DiveId`, `StartDate`, `EndDate`, `Duration`, `MaxDepth`, `TempHigh`, `TempLow`, `Rating`. New fields are added, never renamed; only their displayed labels may change;
- the collection set **"Dive Lens"** and collection names of the form **"&lt;dive number&gt; - &lt;site&gt;"**.

The app bundle identifier `com.querton.divelens.desktop` never changes either, for a different reason: it decides where the app keeps its settings, so changing it would leave every user's settings behind.

## The libdivecomputer submodule

`third_party/libdivecomputer` is pinned to a validated commit. Do not move it in a pull request: if you think it needs an update, open an issue explaining why. libdivecomputer is LGPL 2.1; Dive Lens uses only its parsing code.

## Reporting a bug or asking for a dive application

Use the [issue templates](https://github.com/Ventrilok/divelens/issues/new/choose):

- **Bug report**: something does not work as expected.
- **Support a new dive application**: you would like Dive Lens to read another dive log. If you want to write that reader yourself, open this issue first so the approach can be discussed.

Both remind you of the privacy rule above:

- For a bug, describe the problem in the issue. If the Dive Lens log files are needed, you will be asked for them: send them only by e-mail to [divelens@querton.com](mailto:divelens@querton.com), with the issue number, never in an issue. In the issue itself, paste at most the relevant lines, with site names, places, people's names and file paths removed.
- For a new dive application, a fictional log, or your real log once anonymised, helps testing. Send it by e-mail to the same address, with the issue number, never attached to the issue. It is used to test the new reader. The repository itself only ever holds fictional or properly anonymised test data, never a real log as it is (see [Test data](#test-data-fictional-or-anonymised)).
