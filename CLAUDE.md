# CLAUDE.md

Guidance for Claude Code in this repository.

## Project

DiveLens 2: links underwater photos in Adobe Lightroom Classic with dive-log data. Free, open source, GPL-3.0. Public repository: never commit secrets, keys, certificates or real personal dive data.

The product specification is `docs/spec-v1.md` (French). It is the source of truth for scope and decisions; items are tagged Décidé / Proposé / À trancher.

## Architecture (decided)

- **Lightroom plugin (Lua)**, invisible: declares the DiveLens metadata fields, reads and writes the catalog, follows the photo selection, provides thumbnails, launches the app. Two Library menu items, used only as shortcuts that open the app on a given screen. No Lua dialogs.
- **Desktop app (Rust + Tauri)**: the only user interface. Contains one reader per dive-log application, each translating its data into a common dive format.
- **Link app ↔ plugin**: live read (selection), writes only on "Apply", in a single catalog write (one undo step).
- One package: the app ships inside the plugin folder. Auto-update via GitHub Releases.

V1 sources: MacDive (SQLite, macOS) and Subsurface (XML file and git-based storage, macOS and Windows). Dive logs are always opened read-only.

## Repository layout

- `core/`: Rust library crate `divelens-core` (common dive format, readers, per-photo computations). No Tauri dependency; tested on macOS and Windows in CI.
- `app/`: Tauri 2 app (React + TypeScript front end in `app/src`, Rust side in `app/src-tauri`, crate `divelens`).
- `plugin/` (to come): the Lua Lightroom plugin. Never port `LicenseHelper.lua`, compiled bytecode or the `luac5.1` binary from the legacy repo.
- Cargo workspace at the root: one `target/` and one `Cargo.lock` for both crates.

App bundle identifier: `com.querton.divelens.desktop` (fixed: it decides where app settings live). Version is set once in `app/package.json`.

## Commands

- Core tests: `cargo test -p divelens-core`
- Lint: `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`
- App build (from `app/`): `npm ci`, then `npm run tauri build -- --bundles app` (macOS, skips the DMG) or `-- --no-bundle`
- Dev window (from `app/`): `npm run tauri dev` (opens a window and keeps running)

## Compatibility contract (do not change)

- Plugin identifier: `com.querton.divelens`.
- Existing metadata field ids: `DiveId`, `StartDate`, `EndDate`, `Duration`, `MaxDepth`, `TempHigh`, `TempLow`, `Rating`. New fields are added, never renamed.
- Collection set "Dive Lens", collections named "<dive number> - <site>".

## Rules

- Test data must be fictional or anonymized.
- Signing keys and credentials live in GitHub secrets or the owner's password manager, never in the repo.
