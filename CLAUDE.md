# CLAUDE.md

Guidance for Claude Code in this repository.

## Project

Dive Lens 2: links underwater photos in Adobe Lightroom Classic with dive-log data. Free, open source, GPL-3.0. Public repository: never commit secrets, keys, certificates or real personal dive data.

The product specification is `docs/spec-v1.md` (French). It is the source of truth for scope and decisions; items are tagged Décidé / Proposé / À trancher.

## Contributor guide

The contributor guide below is part of these instructions and applies in full: commands, checks before committing, branches, commits and pull requests, version numbers, test data, secrets, what must never be renamed, adding a dive application. This file adds only what the guide does not say: architecture and layout notes, and how to work here as Claude Code.

@CONTRIBUTING.md

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

## Rules

- Signing keys and credentials live in GitHub secrets or the owner's password manager, never in the repo.

## Git & GitHub

- Never push, open a PR, merge, force-push or rewrite pushed history unless I ask explicitly. Never merge without green CI.
- CI runs on every push to a PR and again on merge; it is free (public repository), so there is no need to batch pushes.
- Point out unrelated changes in the working tree instead of committing them.
- Never move the `third_party/libdivecomputer` submodule unless I ask: it is pinned to a validated commit.
- Before committing, run the checks for what changed, as the contributor guide lists them ("Checks before you commit", commands in "Commands"). Fix failures; never hide or skip one.
