import { beforeEach, describe, expect, it } from "vitest";
import { setLanguage, t } from "../i18n";
import { barText, canApplyCollections, collectionsBlocker, writesNote } from "./rules";
import type { CollectionsPreview } from "../types";

beforeEach(() => setLanguage("en"));

const preview = (total: number) => ({ total, dives: [], cameras: [] }) as unknown as CollectionsPreview;

const ok = {
  connected: true,
  search: { state: "idle" } as const,
  parentMissing: false,
  parent: ["Dive Lens", "2025"],
  preview: preview(4),
};

describe("collectionsBlocker", () => {
  it("lets Apply through when everything is in place", () => {
    expect(collectionsBlocker(ok)).toBeNull();
    expect(collectionsBlocker({ ...ok, preview: null })).toBeNull();
  });

  it("says the photos are no longer available when the counts failed, in both languages", () => {
    const failed = { ...ok, preview: null, previewFailed: true };
    expect(collectionsBlocker(failed)).toBe(t("coll.previewFailed"));
    expect(t("coll.previewFailed")).not.toBe("");
    setLanguage("fr");
    expect(collectionsBlocker(failed)).toBe(t("coll.previewFailed"));
    expect(t("coll.previewFailed")).not.toBe("");
    expect(t("coll.previewFailed")).not.toBe("coll.previewFailed");
  });

  it("keeps the earlier blockers before a failed preview", () => {
    expect(collectionsBlocker({ ...ok, preview: null, previewFailed: true, parentMissing: true })).toBe(
      t("coll.parentMissing", { path: ok.parent.join(" › ") }),
    );
  });

  it("stays silent while the counts have simply not arrived", () => {
    expect(collectionsBlocker({ ...ok, preview: null, previewFailed: false })).toBeNull();
  });

  it("says Lightroom is disconnected first", () => {
    const all = { ...ok, connected: false, search: { state: "searching" }, parentMissing: true, preview: preview(0) };
    expect(collectionsBlocker(all as never)).toBe(t("coll.disconnected"));
  });

  it("says the photos are being searched", () => {
    expect(collectionsBlocker({ ...ok, search: { state: "searching" }, parentMissing: true })).toBe(t("coll.searching"));
  });

  it("shows why the search failed", () => {
    const failed = { state: "failed", message: "Search failed" } as const;
    expect(collectionsBlocker({ ...ok, search: failed, parentMissing: true })).toBe("Search failed");
  });

  it("names the missing parent collection set", () => {
    expect(collectionsBlocker({ ...ok, parentMissing: true, preview: preview(0) })).toBe(
      t("coll.parentMissing", { path: "Dive Lens › 2025" }),
    );
  });

  it("says there is nothing to write when the counts are in and empty", () => {
    expect(collectionsBlocker({ ...ok, preview: preview(0) })).toBe(t("coll.err.nothingToWrite"));
  });
});

describe("canApplyCollections", () => {
  it("needs the counts to be in, besides no blocker and no write in progress", () => {
    expect(canApplyCollections(null, preview(2), false)).toBe(true);
    expect(canApplyCollections(null, null, false)).toBe(false);
    expect(canApplyCollections(null, preview(2), true)).toBe(false);
    expect(canApplyCollections("Blocked", preview(2), false)).toBe(false);
  });
});

describe("barText", () => {
  it("is the blocker when there is one, else where the collections go", () => {
    expect(barText(null, "Dive Lens")).toBe(t("coll.writes", { place: "Dive Lens" }));
    expect(barText("Blocked", "Dive Lens")).toBe("Blocked");
  });
});

describe("writesNote", () => {
  const note = (units: string, altitude: string) => t("coll.writesNote", { units, altitude });

  it("tells the units and whether depth is copied to altitude", () => {
    expect(writesNote({ depth: "m", temp: "c" }, true)).toBe(note("m, °C", t("coll.on")));
    expect(writesNote({ depth: "m", temp: "c" }, false)).toBe(note("m, °C", t("coll.off")));
    expect(writesNote({ depth: "ft", temp: "f" }, true)).toBe(note("ft, °F", t("coll.on")));
    expect(writesNote({ depth: "ft", temp: "f" }, false)).toBe(note("ft, °F", t("coll.off")));
  });
});
