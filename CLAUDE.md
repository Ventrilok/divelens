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

## Compatibility contract (do not change)

- Plugin identifier: `com.querton.divelens`.
- Existing metadata field ids: `DiveId`, `StartDate`, `EndDate`, `Duration`, `MaxDepth`, `TempHigh`, `TempLow`, `Rating`. New fields are added, never renamed.
- Collection set "Dive Lens", collections named "<dive number> - <site>".

## Rules

- Test data must be fictional or anonymized.
- Signing keys and credentials live in GitHub secrets or the owner's password manager, never in the repo.
