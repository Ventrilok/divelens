# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Dive Lens links underwater photos in Adobe Lightroom Classic with dive-log data. Two parts ship together: a Lightroom plug-in (Lua) and a desktop app (Rust + Tauri 2, React + TypeScript front end). The app is the only user interface; in the released package it sits inside the plug-in folder.

- `docs/spec-v1.md` (French) is the source of truth for scope and decisions; items are tagged Décidé / Proposé / À trancher. Anything a diver would notice (a screen, a text, a behaviour, what gets written on photos or in Lightroom) is a product decision: check the spec, and flag what it does not decide.
- `docs/superpowers/` holds dated designs and implementation plans (French). When a design changes the spec, the spec is rewritten first, so it stays the reference.
- `docs/user/` is the user guide, in English, French and German.

## Public repository

Git history is public and permanent. Never commit, or paste in a pull request or issue: a real dive log or values copied from one (dates, sites, positions, names), `verify_log` output, the log files `DiveLens.log` and `DiveLens App.log`, an exported settings file, keys or credentials. Test data is fictional. When something only shows with a real log, describe it in words. The logs can be read locally: `~/Library/Logs/Adobe/Lightroom/LrClassicLogs/` on macOS, `%LOCALAPPDATA%\Adobe\Lightroom\Logs\LrClassicLogs\` on Windows.

## Commands

Setup: `git submodule update --init` (the core does not build without libdivecomputer), then from `app/`: `npm ci` and `npm run build`. The app crate embeds `app/dist`: its Rust lint and tests fail without it.

From the repository root:

- Format: `cargo fmt --all --check` (`cargo fmt --all` fixes)
- Core lint: `cargo clippy -p divelens-core -p libdivecomputer-sys --all-targets -- -D warnings`
- Core tests: `cargo test -p divelens-core [test name]`
- App lint: `cargo clippy -p divelens --all-targets -- -D warnings`
- App and plug-in tests: `cargo test -p divelens [test name]`; one file of `app/src-tauri/tests/`: `--test plugin_collections`
- Read a real log locally: `cargo run --release -p divelens-core --example verify_log -- [--app <application>] <log> [dive number]` (its output is personal data)

From `app/`:

- Front-end tests: `npm test`; one file: `npx vitest run src/journey/rules.test.ts`, with `-t "<name>"` for one case
- Type check and build: `npm run build`
- Dev window: `npm run tauri dev` (connects to Lightroom by itself)
- Build the app Lightroom launches in development: `npm run tauri build -- --bundles app` (macOS), `npm run tauri build -- --no-bundle` (Windows)

## Checks before committing

CI runs every check on macOS and Windows for each push to a pull request, except when only documentation changed (`docs/`, Markdown files, `LICENSE`). Run the checks for what changed and fix failures; never skip, disable or loosen a test.

- Always: the format check.
- `core/` or `libdivecomputer-sys/`: core lint and core tests; the app checks too when what the core exposes to the app changed.
- App, front end or plug-in: `npm test`, `npm run build`, app lint, app tests.

## Architecture

- One Cargo workspace, one `target/`: `core/` (common dive format, readers, per-photo computations; no Tauri), `libdivecomputer-sys/` (builds the C decoder; parsing only), `app/src-tauri/` (crate `divelens`). Front end in `app/src`, plug-in in `plugin/DiveLens.lrdevplugin/` (plain Lua, no build step), installers in `packaging/`.
- Dive applications (MacDive, Subsurface): each has a reader (`core/src/<application>.rs`, implements `DiveSource`) and a description (`core/src/<application>/app.rs`, registered in `ALL` in `core/src/apps.rs`). Settings, the file chooser and the dive-log error screen are built from the descriptions. `core/src/conformance.rs` runs the same tests on every registered application, against a fictional reference log. To add one, follow `docs/contributing/reference-log.md`, starting from the example reader `core/src/skeleton.rs`.
- Readers give wall-clock times exactly as the log holds them (never converted with the computer's time zone), metric units, stable ids, profiles sorted by time, and open the log read-only without creating any file next to it (SQLite through `crate::sqlite::open_read_only`).
- MacDive profiles come from `ZRAWDATA` decoded by libdivecomputer (Shearwater data is decompressed first). `ZSAMPLES` is proprietary and not used. A `ZSAMPLES_JSON` column seen in some test databases was added by an external tool (DiverCity), not by MacDive: never rely on it.
- Link app ↔ plug-in: the plug-in listens on two local sockets (one per direction) and writes their ports and a token to `lightroom-link.json` in the app's settings folder; the app connects, says hello with the token, and receives the live selection. One JSON object per line. Plug-in side `Link.lua`, app side `app/src-tauri/src/lightroom.rs`.
- Catalog writes happen only when the user clicks Apply: two writes in a row, under the one Undo name "Dive Lens", with GPS and altitude in the second and nothing sent or yielded in between. Lightroom does not undo a GPS position or altitude written in the same write as a location text field. The first time, the collection set is created in a separate write before them: a set created inside a write cannot be used until that write ends.
- The plug-in has no screen of its own: menus that open the app, a section in Lightroom's Plug-in Manager (link state, buttons), and an error message when the app cannot be launched. Its pure-Lua parts run under Lua 5.1 in `app/src-tauri/tests/` (`plugin_*.rs`, one file per area, and `apply_contract.rs`); the rest is checked by hand in Lightroom (`docs/contributing/lightroom-checks.md`, before a release that touches the plug-in or the catalog writes).
- In development, the plug-in's menus launch the release build of the repository: `target/release/bundle/macos/Dive Lens.app` or `target/release/divelens.exe`.
- Front end ↔ Rust: the app's own commands are called only from `app/src/api.ts`. The Tauri dialog and opener plug-ins are imported directly where needed.
- Front-end tests run at UTC+14 on purpose, to catch any time-zone shift of dive times.

## Visible text

Every visible text exists in English, French and German. The places: `app/src/i18n/en.ts`, `fr.ts` and `de.ts` (English defines the keys); the plug-in's `LOC` strings, `TranslatedStrings_fr.txt` and `TranslatedStrings_de.txt`; each application's description in `core/`; the installers (`packaging/macos/resources/*.lproj`, the `[Messages]` section of `packaging/windows/divelens.iss`); the user guide in `docs/user/`.

Visible text says "Dive Lens", two words. Identifiers keep "DiveLens" / "divelens".

## Never change

Existing Lightroom catalogs and settings depend on these:

- the plug-in identifier `com.querton.divelens` and the app identifier `com.querton.divelens.desktop` (it decides where settings are kept);
- the ids of the metadata fields in `Metadata.lua`, all released: fields are added, never renamed; only displayed labels change;
- the collection set "Dive Lens" and the collection names built in `core/src/collections.rs` ("<dive number> - <site>").

Also never:

- move the `third_party/libdivecomputer` submodule unless I ask: it is pinned to a validated commit;
- copy Adobe Lightroom SDK files, or the 0.x plug-in's licence code, compiled Lua or `luac5.1`, into the repository.

## Versions

- The version of Dive Lens is set only in `app/package.json`, with `npm run set-version <version>`, which copies it to `plugin/DiveLens.lrdevplugin/Info.lua`, `Cargo.toml` and `Cargo.lock`. Never edit a copy by hand: `app/src-tauri/tests/version.rs` fails when one differs. A version change that reaches `main` triggers the release workflow.
- `npm run release -- <patch|minor|major|prerelease|version>` pushes a branch and opens a pull request set to merge by itself: never run it unless I ask.
- Compatibility numbers, separate from the version, each raised only when its own format changes:
  - `PROTOCOL` in `lightroom.rs` and `Link.lua`, both together, when a message changes in a way the other side would misread;
  - `schemaVersion` in `Metadata.lua`, when a field is added or its definition changes;
  - `CURRENT_VERSION` in `app/src-tauri/src/settings/file.rs` and in `app/src-tauri/src/apply/offsets.rs`, when the file layout changes, keeping older files readable.

## Git & GitHub

- Never work on `main`. Branches are `<type>/<short-kebab-slug>`, with the issue number when there is one (`fix/12-empty-profile`).
- Commits follow Conventional Commits: `<type>(<scope>): <description>`, imperative, lower-case, no trailing period, at most 72 characters. Types: `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `style`, `revert`. Scopes: `core`, `macdive`, `subsurface`, `app`, `plugin`, `link`, `spec`, `deps`, and one per new dive application.
- Pull requests are squash-merged: the title follows the commit format. Fill in `.github/pull_request_template.md`, including "Visible choices".
- Never push, open a PR, open an issue or comment on GitHub, merge, force-push or rewrite pushed history unless I ask explicitly. Never merge without green CI.
- CI is free (public repository): no need to batch pushes.
- Point out unrelated changes in the working tree instead of committing them. Never commit build output or generated files.
