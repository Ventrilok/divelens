// Cases the rules of Create Collections and Tag Selection pinned before the
// journey, pinned again on the journey's rules where no other test of step 4
// already does. Fictional sites and dives only (fixtures.ts).
import { beforeEach, describe, expect, it } from "vitest";
import { setLanguage, t } from "../../i18n";
import { canApply, reviewBlocker } from "../rules";
import { input, nothingRead, pickBlueArch, read, ready, run, selectionAt4, withOptions, catalogAt4 } from "./fixtures";

beforeEach(() => setLanguage("en"));

describe("from Create Collections", () => {
  it("stays silent while the counts have simply not arrived, and Apply waits for them", () => {
    const counting = input(catalogAt4, { catalog: { search: ready, preview: null, previewFailed: false } });
    expect(reviewBlocker(counting)).toBeNull();
    expect(canApply(counting, false)).toBe(false);
  });

  it("says the photos are no longer available when the counts failed, in both languages", () => {
    const failed = input(catalogAt4, { catalog: { search: ready, preview: null, previewFailed: true } });
    expect(reviewBlocker(failed)).toBe(t("coll.previewFailed"));
    expect(t("coll.previewFailed")).not.toBe("");
    setLanguage("fr");
    expect(reviewBlocker(failed)).toBe(t("coll.previewFailed"));
    expect(t("coll.previewFailed")).not.toBe("");
    expect(t("coll.previewFailed")).not.toBe("coll.previewFailed");
  });

  it("says the photos are being searched when the search restarts after failed counts", () => {
    const again = input(catalogAt4, { catalog: { search: { state: "searching" }, preview: null, previewFailed: true } });
    expect(reviewBlocker(again)).toBe("Looking for the photos in Lightroom…");
  });
});

describe("from Tag Selection", () => {
  /** Dive 14, whose site has no id: the location can only be another site. */
  const withoutSite = run(
    selectionAt4,
    { type: "goTo", step: 2 },
    { type: "pickDive", pick: { kind: "dive", id: "d3" } },
    { type: "next" },
    { type: "next" },
  );

  it("says Lightroom is disconnected first, whatever else is wrong", () => {
    const all = withOptions(selectionAt4, { dive: false, place: false, gps: false, collections: false, site: "other", pick: { ...pickBlueArch, lat: "91" } });
    expect(reviewBlocker(input(all, { lr: { connected: false, selected: 0 } }))).toBe("Lightroom not connected: nothing can be written");
  });

  it("asks to select photos in Lightroom before anything about the options", () => {
    const none = withOptions(selectionAt4, { dive: false, place: false, gps: false, collections: false });
    expect(reviewBlocker(input(none, { lr: { connected: true, selected: 0 } }))).toBe("Select photos in Lightroom");
  });

  it("shows a reading failure only when per-photo values are written", () => {
    expect(reviewBlocker(input(selectionAt4, { selection: { ...nothingRead, readFailure: "Reading failed" } }))).toBeNull();
  });

  it("does not wait for the selection to be read without per-photo values", () => {
    expect(reviewBlocker(input(selectionAt4, { selection: { ...read, needsValues: false, current: null, freshCounts: null } }))).toBeNull();
  });

  it("does not ask for a site with the location and GPS unticked", () => {
    expect(reviewBlocker(input(withOptions(withoutSite, { place: false, gps: false }), { selection: nothingRead }))).toBeNull();
  });

  it("refuses a bad GPS position when the dive's site cannot be used", () => {
    const bad = withOptions(withoutSite, { pick: { ...pickBlueArch, lat: "91" } });
    expect(reviewBlocker(input(bad, { selection: nothingRead }))).toBe(
      "Latitude between -90 and 90, longitude between -180 and 180, or both empty.",
    );
  });
});
