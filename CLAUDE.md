# CLAUDE.md

Guidance for Claude Code in this repository.

## Project

Dive Lens 2: links underwater photos in Adobe Lightroom Classic with dive-log data. Free, open source, GPL-3.0. Public repository: never commit secrets, keys, certificates or real personal dive data.

The product specification is `docs/spec-v1.md` (French). It is the source of truth for scope and decisions; items are tagged Décidé / Proposé / À trancher.

## Architecture (decided)

- **Lightroom plugin (Lua)**, invisible: declares the DiveLens metadata fields, reads and writes the catalog, follows the photo selection, provides thumbnails, launches the app. Two Library menu items, used only as shortcuts that open the app on a given screen. No Lua dialogs.
- **Desktop app (Rust + Tauri)**: the only user interface. Contains one reader per dive-log application, each translating its data into a common dive format.
- **Link app ↔ plugin**: live read (selection), writes only on "Apply", in a single catalog write (one undo step).
- One package: the app ships inside the plugin folder. Auto-update via GitHub Releases.

V1 sources: MacDive (SQLite, macOS) and Subsurface (XML file and git-based storage, macOS and Windows). Dive logs are always opened read-only.

## Repository layout

- `core/`: Rust library crate `divelens-core` (common dive format, readers, per-photo computations). No Tauri dependency; tested on macOS and Windows in CI.
- `libdivecomputer-sys/`: compiles libdivecomputer (git submodule `third_party/libdivecomputer`, pinned to the commit DiverCity validated) and declares the few C functions used. Parsing only: no USB/Bluetooth/serial.
- MacDive profiles come from `ZRAWDATA` decoded by libdivecomputer (Shearwater data is decompressed first). `ZSAMPLES` is proprietary and not used. A `ZSAMPLES_JSON` column seen in some test databases was added by DiverCity's tooling, not by MacDive: never rely on it.
- `app/`: Tauri 2 app (React + TypeScript front end in `app/src`, Rust side in `app/src-tauri`, crate `divelens`).
- `plugin/DiveLens.lrdevplugin/`: the Lua Lightroom plugin, shipped as plain source (no compile step). Never port `LicenseHelper.lua`, compiled bytecode or the `luac5.1` binary from the legacy repo, and never copy Adobe SDK files.
- Link app ↔ plugin: the plugin listens on two local sockets (one per direction, LrSocket) and writes their ports and a token to `lightroom-link.json` in the app's settings folder; the app connects, says hello with the token, and receives the live selection. One JSON object per line. Plugin side `Link.lua`, app side `app/src-tauri/src/lightroom.rs`.
- The plugin's pure-Lua parts (JSON, framing, manifest, fields) are tested under Lua 5.1 from `app/src-tauri/tests/plugin_lua.rs`.
- Cargo workspace at the root: one `target/` and one `Cargo.lock` for its three crates (`core`, `libdivecomputer-sys`, `app/src-tauri`).

App bundle identifier: `com.querton.divelens.desktop` (fixed: it decides where app settings live). The product version is set only in `app/package.json`, with `npm run set-version <version>` from `app/`, which copies it to `Info.lua`, `Cargo.toml` and `Cargo.lock` (a test fails when a copy differs; see CONTRIBUTING.md, "Version numbers").

## Commands

- After cloning: `git submodule update --init` (the core does not build without libdivecomputer)
- Core tests: `cargo test -p divelens-core`
- App and plugin tests: `cargo test -p divelens` (needs `app/dist`, i.e. one front-end build)
- Plugin in Lightroom (dev): Plug-in Manager › Add › `plugin/DiveLens.lrdevplugin`. Its menus launch `target/release/bundle/macos/Dive Lens.app` (macOS) or `target/release/divelens.exe` (Windows) when no app ships in the plugin folder; a running dev window connects by itself. Plugin log: `DiveLens.log` in Lightroom's log folder (macOS `~/Library/Logs/Adobe/Lightroom/LrClassicLogs/`).
- Lint: `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` (the app crate needs `app/dist`; core only: `cargo clippy -p divelens-core -p libdivecomputer-sys --all-targets -- -D warnings`)
- Front-end type check (from `app/`): `npm run build` (runs `tsc`, then produces `app/dist`)
- Front-end tests (from `app/`): `npm test` (runs in a UTC+14 time zone on purpose: dive times are wall-clock and must never shift)
- Check a reader against a real dive log, any application (local only; the output is personal data and never goes into the repository, an issue or a PR): `cargo run --release -p divelens-core --example verify_log -- [--app MacDive|Subsurface] <path> [dive number]`
- App build (from `app/`): `npm ci`, then `npm run tauri build -- --bundles app` (macOS, skips the DMG) or `-- --no-bundle`
- Dev window (from `app/`): `npm run tauri dev` (opens a window and keeps running)

## Compatibility contract (do not change)

- Plugin identifier: `com.querton.divelens`.
- Existing metadata field ids: `DiveId`, `StartDate`, `EndDate`, `Duration`, `MaxDepth`, `TempHigh`, `TempLow`, `Rating`. New fields are added, never renamed.
- Collection set "Dive Lens", collections named "<dive number> - <site>".
- Product name wherever a person reads it (app, Lightroom texts, bundle name, docs): "Dive Lens", two words. Identifiers keep "DiveLens" / "divelens": plugin id and folder, bundle identifier, translation keys, crate names, log file names.

## Rules

- Test data must be fictional or anonymized.
- Signing keys and credentials live in GitHub secrets or the owner's password manager, never in the repo.

## Git & GitHub

- Never push, open a PR, merge, force-push or rewrite pushed history unless I ask explicitly. Never merge without green CI.
- CI runs on every push to a PR and again on merge; it is free (public repository), so there is no need to batch pushes.
- One branch per coherent change, never work on `main`. Branch names: `<type>/<short-kebab-slug>` with a commit type, e.g. `feat/tag-selection`, `fix/12-empty-profile` (issue number when there is one).
- Commits follow Conventional Commits 1.0.0: `<type>(<scope>): <description>`, imperative, lower-case, no trailing period, at most 72 characters. Types: `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `style`, `revert`. Optional scope: `core`, `macdive`, `subsurface`, `app`, `plugin`, `link`, `spec`, `deps`. PR titles use the same format (PRs are squash-merged).
- Don't mix unrelated changes in one commit; point out unrelated changes in the working tree instead of committing them.
- Never commit real dive logs or databases (fictional fixtures only), output from a real database, build output or generated files. Never paste real dive data into a PR or issue: the repository is public.
- Never move the `third_party/libdivecomputer` submodule unless I ask: it is pinned to a validated commit.
- Before committing, run the checks for what changed (see Commands): `cargo fmt --all --check` always; core clippy and tests for `core/`; `npm test`, `npm run build`, then app clippy and tests for the app, front end or plugin. Fix failures; never hide or skip one.
