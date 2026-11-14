<!--
The title follows Conventional Commits, like a commit message:
<type>(<scope>): <description>, imperative, lower-case, no trailing period,
at most 72 characters. Pull requests are squash-merged: the title becomes the
commit on main. See CONTRIBUTING.md.

This repository is public: never attach or paste a real dive log, a dive-log
database, output produced from one, or the log files DiveLens.log and
DiveLens App.log. Crop or blur screenshots.
-->

## Summary

<!-- What changes, and why. Link the issue it closes, if any ("Closes #12"). -->

## Type

<!-- Tick the type used in the title. -->

- [ ] `feat`: new feature
- [ ] `fix`: bug fix
- [ ] `docs`: documentation only
- [ ] `refactor`, `perf`, `test`, `style`
- [ ] `build`, `ci`, `chore`, `revert`

## Checks run

<!-- Tick what you ran, for what you changed (see "Checks before you commit" in CONTRIBUTING.md). -->

- [ ] `cargo fmt --all --check`
- [ ] Core: `cargo clippy -p divelens-core -p libdivecomputer-sys --all-targets -- -D warnings` and `cargo test -p divelens-core`
- [ ] App, front end or plug-in: `npm test` and `npm run build` (in `app/`), then `cargo clippy -p divelens --all-targets -- -D warnings` and `cargo test -p divelens`
- [ ] Tried in Lightroom Classic (macOS / Windows):
- [ ] Documentation only: no check needed

## Data and privacy

- [ ] Test data and examples are fictional or anonymised.
- [ ] No real dive log, dive-log database, output from one, Dive Lens log file, secret or key is included, and screenshots have personal details removed.

## Visible choices

<!--
Does this change anything a diver would notice: a screen, a text, a
behaviour, what gets written on photos or in Lightroom? Describe it, and say
whether docs/spec-v1.md already decides it or this pull request proposes it.
Write "None" otherwise.
-->
