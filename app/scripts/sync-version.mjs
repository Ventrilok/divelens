// Copies the version of app/package.json, the only place where it is set, to
// the files that cannot read it: the Lightroom plug-in's Info.lua, and the Rust
// workspace (Cargo.toml, Cargo.lock). Always change the version with
// `npm run set-version <version>`: npm sets it in package.json and
// package-lock.json, then runs this script (npm's `version` script). This
// script alone never touches package-lock.json.
// The app's tests fail when a copy differs (app/src-tauri/tests/version.rs).
//
// Part of Dive Lens, GPL-3.0-or-later.

import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const app = join(dirname(fileURLToPath(import.meta.url)), "..");
const root = join(app, "..");

const { version } = JSON.parse(readFileSync(join(app, "package.json"), "utf8"));

// When npm runs this script, it has already written the new version in
// package.json and package-lock.json: a failure leaves only the copies as they
// were, and the version test reports the mismatch until it is fixed.
function fail(reason) {
  console.error(
    `${reason}: Info.lua, Cargo.toml and Cargo.lock not changed; app/package.json says ${version}, ` +
      "so `cargo test -p divelens` reports the mismatch until this is fixed.",
  );
  process.exit(1);
}

// MAJOR.MINOR.PATCH, optionally followed by a pre-release label (2.1.0-beta.1).
const parts = /^(\d+)\.(\d+)\.(\d+)(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$/.exec(version);
if (!parts) {
  fail(`"${version}" is not MAJOR.MINOR.PATCH or MAJOR.MINOR.PATCH-label`);
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

// Check every copy before writing any, so that they are all changed or none.
const updated = copies.map(({ file, pattern, replace, count }) => {
  const path = join(root, file);
  const text = readFileSync(path, "utf8");
  const found = text.match(pattern)?.length ?? 0;
  if (found !== count) {
    fail(`${file}: expected ${count} version entries, found ${found}`);
  }
  return { file, path, text, next: text.replace(pattern, replace) };
});
for (const { file, path, text, next } of updated) {
  if (next !== text) {
    writeFileSync(path, next);
    console.log(`${file}: ${version}`);
  }
}
