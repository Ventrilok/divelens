// Starts a new version of Dive Lens, in one command, from app/:
//
//   npm run release -- patch | minor | major | prerelease | <version>
//
// patch, minor and major raise that number (2.0.0 -> 2.0.1, 2.1.0, 3.0.0);
// prerelease raises the label's number (2.1.0-beta.1 -> 2.1.0-beta.2) and is
// accepted only on a pre-release; <version> is written as given (2.1.0-beta.1).
//
// It checks that the clone is on main, up to date and without uncommitted
// changes, writes the version everywhere (npm run set-version), commits it on
// a branch build/release-<version>, pushes it and opens the pull request
// "build: release <version>", set to merge by itself once its checks pass.
// Once merged, the release workflow builds the version and drafts its GitHub
// release (.github/workflows/release.yml); the owner reviews and publishes it.
//
// Needs git, and the GitHub CLI (gh) signed in.
//
// Part of Dive Lens, GPL-3.0-or-later.

import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const app = join(dirname(fileURLToPath(import.meta.url)), "..");
const root = join(app, "..");

function fail(message) {
  console.error(`release: ${message}`);
  process.exit(1);
}

// Runs a command in the repository; returns its output, or prints it when
// `show` is set.
function run(command, args, { show = false, cwd = root, allowFailure = false } = {}) {
  try {
    const out = execFileSync(command, args, {
      cwd,
      encoding: "utf8",
      stdio: show ? "inherit" : ["ignore", "pipe", "pipe"],
    });
    return (out ?? "").trim();
  } catch (error) {
    if (allowFailure) return null;
    const detail = (error.stderr || error.message || "").toString().trim();
    fail(`${command} ${args.slice(0, 2).join(" ")} failed${detail ? `:\n${detail}` : ""}`);
  }
}

const currentVersion = () => JSON.parse(readFileSync(join(app, "package.json"), "utf8")).version;

const SEMVER = /^\d+\.\d+\.\d+(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$/;
const request = process.argv[2];
if (!request || !(["patch", "minor", "major", "prerelease"].includes(request) || SEMVER.test(request))) {
  fail("say which version: npm run release -- patch | minor | major | prerelease | 2.1.0");
}

// The clone: on main, nothing uncommitted, same commit as GitHub's main.
if (run("gh", ["auth", "status"], { allowFailure: true }) === null) {
  fail("the GitHub CLI is not signed in: run `gh auth login` first");
}
if (run("git", ["rev-parse", "--abbrev-ref", "HEAD"]) !== "main") {
  fail("switch to main first (git switch main)");
}
if (run("git", ["status", "--porcelain", "--untracked-files=no"]) !== "") {
  fail("main has uncommitted changes: commit or set them aside first");
}
run("git", ["fetch", "--quiet", "origin", "main", "--tags"]);
if (run("git", ["rev-parse", "HEAD"]) !== run("git", ["rev-parse", "origin/main"])) {
  fail("main is not the same as GitHub's: run `git pull` (and push nothing to main directly)");
}

const from = currentVersion();
if (request === "prerelease" && !from.includes("-")) {
  fail(`${from} is not a pre-release: give the version, for example 2.1.0-beta.1`);
}
if (request === from) {
  fail(`the version is already ${from}`);
}

// Writes the version in package.json, package-lock.json, the plug-in's
// Info.lua, Cargo.toml and Cargo.lock.
run("npm", ["run", "--silent", "set-version", "--", request], { cwd: app, show: true });
const version = currentVersion();
const tag = `v${version}`;

const restore = () => run("git", ["checkout", "--", "."], { allowFailure: true });
if (run("git", ["ls-remote", "--tags", "origin", tag]) !== "") {
  restore();
  fail(`${tag} already exists on GitHub: choose another version`);
}
const published = run("gh", ["release", "view", tag, "--json", "isDraft", "--jq", ".isDraft"], {
  allowFailure: true,
});
if (published === "false") {
  restore();
  fail(`Dive Lens ${version} is already published: choose another version`);
}

const branch = `build/release-${version}`;
const title = `build: release ${version}`;
run("git", ["switch", "--quiet", "-c", branch]);
run("git", ["add", "--update"]);
run("git", ["commit", "--quiet", "-m", title]);
run("git", ["push", "--quiet", "-u", "origin", branch], { show: true });

const body = [
  `Sets the version to ${version} (from ${from}) with \`npm run release\`.`,
  "",
  "Once merged, the release workflow builds, signs and notarizes this version and drafts its GitHub release:",
  "review it on the Releases page, then publish it.",
].join("\n");
const url = run("gh", ["pr", "create", "--base", "main", "--head", branch, "--title", title, "--body", body]);
run("gh", ["pr", "merge", url, "--auto", "--squash", "--delete-branch"]);
run("git", ["switch", "--quiet", "main"]);

console.log(`
Version ${version}: ${url}
GitHub merges it once its checks pass, then drafts "Dive Lens ${version}":
https://github.com/Ventrilok/divelens/releases`);
