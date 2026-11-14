// Copies the version of app/package.json, the only place where it is set, to
// the files that cannot read it: the Lightroom plug-in's Info.lua, and the Rust
// workspace (Cargo.toml, Cargo.lock). `npm run set-version <version>` runs it
// once package.json and package-lock.json carry the new version (npm's
// `version` script); `node scripts/sync-version.mjs` alone re-syncs the copies.
// The app's tests fail when a copy differs (app/src-tauri/tests/version.rs).
//
// Part of Dive Lens, GPL-3.0-or-later.

import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const app = join(dirname(fileURLToPath(import.meta.url)), "..");
const root = join(app, "..");

const { version } = JSON.parse(readFileSync(join(app, "package.json"), "utf8"));
// MAJOR.MINOR.PATCH, optionally followed by a pre-release label (2.0.0-beta.1).
const parts = /^(\d+)\.(\d+)\.(\d+)(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$/.exec(version);
if (!parts) {
  console.error(
    `app/package.json version "${version}" is not MAJOR.MINOR.PATCH or MAJOR.MINOR.PATCH-label: nothing copied.`,
  );
  process.exit(1);
}
const [, major, minor, patch] = parts;

const copies = [
  {
    file: "plugin/DiveLens.lrdevplugin/Info.lua",
    pattern: /VERSION = \{ major = \d+, minor = \d+, revision = \d+, display = "[^"]*" \}/g,
    replace: () =>
      `VERSION = { major = ${major}, minor = ${minor}, revision = ${patch}, display = "${version}" }`,
    count: 1,
  },
  {
    file: "Cargo.toml",
    pattern: /(\[workspace\.package\][^[]*?\nversion = ")[^"]*"/g,
    replace: (_, start) => `${start}${version}"`,
    count: 1,
  },
  {
    // The workspace's own crates: the only entries without a `source`.
    file: "Cargo.lock",
    pattern: /(\[\[package\]\]\r?\nname = "(?:divelens|divelens-core|libdivecomputer-sys)"\r?\nversion = ")[^"]*"/g,
    replace: (_, start) => `${start}${version}"`,
    count: 3,
  },
];

// Check every file before writing any, so a failure changes nothing.
const updated = copies.map(({ file, pattern, replace, count }) => {
  const path = join(root, file);
  const text = readFileSync(path, "utf8");
  const found = text.match(pattern)?.length ?? 0;
  if (found !== count) {
    console.error(`${file}: expected ${count} version entries, found ${found}: nothing copied.`);
    process.exit(1);
  }
  return { file, path, text, next: text.replace(pattern, replace) };
});
for (const { file, path, text, next } of updated) {
  if (next !== text) {
    writeFileSync(path, next);
    console.log(`${file}: ${version}`);
  }
}
