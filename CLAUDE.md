# CLAUDE.md

Guidance for Claude Code in this repository.

## Project

Dive Lens 2: links underwater photos in Adobe Lightroom Classic with dive-log data. Free, open source, GPL-3.0. Public repository: never commit secrets, keys, certificates or real personal dive data.

The product specification is `docs/spec-v1.md` (French). It is the source of truth for scope and decisions; items are tagged Décidé / Proposé / À trancher.

## Contributor guide

The contributor guide below is part of these instructions and applies in full: commands, checks before committing, branches, commits and pull requests, version numbers, test data, secrets, what must never be renamed, adding a dive application. This file adds only what the guide does not say: a few architecture and layout details, and how to work here as Claude Code.

@CONTRIBUTING.md

## Architecture and layout, beyond the guide

- Plugin: it launches the app, and has no Lua dialogs. Auto-update goes through GitHub Releases.
- V1 sources: MacDive (SQLite, macOS) and Subsurface (XML file and git-based storage, macOS and Windows).
- The libdivecomputer submodule is pinned to the commit DiverCity validated.
- MacDive profiles come from `ZRAWDATA` decoded by libdivecomputer (Shearwater data is decompressed first). `ZSAMPLES` is proprietary and not used. A `ZSAMPLES_JSON` column seen in some test databases was added by DiverCity's tooling, not by MacDive: never rely on it.
- Link app ↔ plugin: the plugin listens on two local sockets (one per direction, LrSocket) and writes their ports and a token to `lightroom-link.json` in the app's settings folder; the app connects, says hello with the token, and receives the live selection. One JSON object per line. Plugin side `Link.lua`, app side `app/src-tauri/src/lightroom.rs`.
- The plugin's pure-Lua parts are tested from `app/src-tauri/tests/plugin_lua.rs`.

## Rules

- Signing keys and credentials live in GitHub secrets or the owner's password manager, never in the repo.

## Git & GitHub

- Never push, open a PR, open an issue or comment on GitHub, merge, force-push or rewrite pushed history unless I ask explicitly. Never merge without green CI.
- CI runs on every push to a PR and again on merge; it is free (public repository), so there is no need to batch pushes.
- Point out unrelated changes in the working tree instead of committing them.
- Never move the `third_party/libdivecomputer` submodule unless I ask: it is pinned to a validated commit.
- Before committing, run the checks for what changed, as the contributor guide lists them ("Checks before you commit", commands in "Commands"). Fix failures; never hide or skip one.
