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
- `libdivecomputer-sys/`: compiles libdivecomputer (git submodule `third_party/libdivecomputer`, pinned to the commit DiverCity validated) and declares the few C functions used. Parsing only: no USB/Bluetooth/serial.
- MacDive profiles come from `ZRAWDATA` decoded by libdivecomputer (Shearwater data is decompressed first). `ZSAMPLES` is proprietary and not used. A `ZSAMPLES_JSON` column seen in some test databases was added by DiverCity's tooling, not by MacDive: never rely on it.
- `app/`: Tauri 2 app (React + TypeScript front end in `app/src`, Rust side in `app/src-tauri`, crate `divelens`).
- `plugin/DiveLens.lrdevplugin/`: the Lua Lightroom plugin, shipped as plain source (no compile step). Never port `LicenseHelper.lua`, compiled bytecode or the `luac5.1` binary from the legacy repo, and never copy Adobe SDK files.
- Link app ↔ plugin: the plugin listens on two local sockets (one per direction, LrSocket) and writes their ports and a token to `lightroom-link.json` in the app's settings folder; the app connects, says hello with the token, and receives the live selection. One JSON object per line. Plugin side `Link.lua`, app side `app/src-tauri/src/lightroom.rs`.
- The plugin's pure-Lua parts (JSON, framing, manifest, fields) are tested under Lua 5.1 from `app/src-tauri/tests/plugin_lua.rs`.
- Cargo workspace at the root: one `target/` and one `Cargo.lock` for both crates.

App bundle identifier: `com.querton.divelens.desktop` (fixed: it decides where app settings live). Version is set once in `app/package.json`.

## Commands

- Core tests: `cargo test -p divelens-core`
- App and plugin tests: `cargo test -p divelens` (needs `app/dist`, i.e. one front-end build)
- Plugin in Lightroom (dev): Plug-in Manager › Add › `plugin/DiveLens.lrdevplugin`. Its menus launch `target/release/bundle/macos/DiveLens.app` (macOS) or `target/release/divelens.exe` (Windows) when no app ships in the plugin folder; a running dev window connects by itself. Plugin log: `DiveLens.log` in Lightroom's log folder (macOS `~/Library/Logs/Adobe/Lightroom/LrClassicLogs/`).
- Lint: `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`
- Front-end tests (from `app/`): `npm test` (runs in a UTC+14 time zone on purpose: dive times are wall-clock and must never shift)
- Check the MacDive reader against a real database (local only, output is personal data): `cargo run -p divelens-core --example macdive_dump -- <path> [dive number]`
- App build (from `app/`): `npm ci`, then `npm run tauri build -- --bundles app` (macOS, skips the DMG) or `-- --no-bundle`
- Dev window (from `app/`): `npm run tauri dev` (opens a window and keeps running)

## Compatibility contract (do not change)

- Plugin identifier: `com.querton.divelens`.
- Existing metadata field ids: `DiveId`, `StartDate`, `EndDate`, `Duration`, `MaxDepth`, `TempHigh`, `TempLow`, `Rating`. New fields are added, never renamed.
- Collection set "Dive Lens", collections named "<dive number> - <site>".

## Rules

- Test data must be fictional or anonymized.
- Signing keys and credentials live in GitHub secrets or the owner's password manager, never in the repo.
