import { beforeEach, describe, expect, it } from "vitest";
import { setLanguage, t } from "../i18n";
import { canApplyTag, describeAgain, selectionChanged, tagBlocker, tagSummary } from "./rules";
import type { Dive } from "../types";

beforeEach(() => setLanguage("en"));

/** Everything in order: nothing blocks. */
const ok = {
  connected: true,
  selected: 3,
  withDive: true,
  withPlace: true,
  chosen: true,
  needsValues: true,
  readFailure: null,
  profileLoading: false,
  current: true,
  freshCounts: true,
  usesDiveSite: true,
  pickedSite: false,
  gpsError: false,
};

const dive = (siteId: string | null, siteName: string | null, number: number | null = 12): Dive => ({
  id: "d1",
  number,
  start: "2025-12-30T07:25:00",
  end: "2025-12-30T08:21:00",
  durationS: 3360,
  maxDepthM: null,
  tempHighC: null,
  tempLowC: null,
  rating: null,
  site: { id: siteId, name: siteName, location: null, country: null, lat: null, lon: null },
  hasProfile: false,
  invalid: false,
  notes: null,
});

describe("tagBlocker", () => {
  it("lets Apply through when everything is in place", () => {
    expect(tagBlocker(ok)).toBeNull();
  });

  it("says Lightroom is disconnected first, whatever else is wrong", () => {
    const all = { ...ok, connected: false, selected: 0, withDive: false, withPlace: false, gpsError: true };
    expect(tagBlocker(all)).toBe(t("tag.disconnected"));
  });

  it("asks to select photos in Lightroom", () => {
    expect(tagBlocker({ ...ok, selected: 0, withDive: false, withPlace: false })).toBe(t("tag.selectInLr"));
  });

  it("asks to choose something to write", () => {
    expect(tagBlocker({ ...ok, withDive: false, withPlace: false, chosen: false })).toBe(t("tag.chooseSomething"));
  });

  it("asks to choose a dive", () => {
    expect(tagBlocker({ ...ok, chosen: false, readFailure: "x", profileLoading: true })).toBe(t("tag.chooseDive"));
  });

  it("does not ask for a dive when only the place is written", () => {
    expect(tagBlocker({ ...ok, withDive: false, chosen: false, needsValues: false })).toBeNull();
  });

  it("shows the read failure, only when per-photo values are needed", () => {
    expect(tagBlocker({ ...ok, readFailure: "Reading failed", profileLoading: true })).toBe("Reading failed");
    expect(tagBlocker({ ...ok, needsValues: false, readFailure: "Reading failed" })).toBeNull();
  });

  it("says the profile is loading", () => {
    expect(tagBlocker({ ...ok, profileLoading: true, current: false })).toBe(t("detail.profileLoading"));
  });

  it("says the photos are being read, when the selection is not described or counts are stale", () => {
    expect(tagBlocker({ ...ok, current: false })).toBe(t("tag.perPhoto.reading"));
    expect(tagBlocker({ ...ok, freshCounts: false })).toBe(t("tag.perPhoto.reading"));
    expect(tagBlocker({ ...ok, needsValues: false, current: false, freshCounts: false })).toBeNull();
  });

  it("asks to choose a site when the place is neither the dive's nor a picked site", () => {
    expect(tagBlocker({ ...ok, usesDiveSite: false, pickedSite: false, gpsError: true })).toBe(t("tag.chooseSite"));
    expect(tagBlocker({ ...ok, usesDiveSite: false, pickedSite: true })).toBeNull();
    expect(tagBlocker({ ...ok, withPlace: false, usesDiveSite: false })).toBeNull();
  });

  it("refuses a bad GPS position last", () => {
    expect(tagBlocker({ ...ok, usesDiveSite: false, pickedSite: true, gpsError: true })).toBe(t("tag.badGps"));
  });
});

describe("tagSummary", () => {
  it("is the blocker when there is one", () => {
    expect(tagSummary("Blocked", dive("s", "Reef"), "Reef")).toBe("Blocked");
  });

  it("names the dive and the place", () => {
    expect(tagSummary(null, dive("s", "Reef"), "Reef")).toBe(t("tag.sumDiveAndPlace", { n: "12", site: "Reef" }));
  });

  it("names the dive alone when the place is not written", () => {
    expect(tagSummary(null, dive("s", "Reef"), null)).toBe(t("tag.sumDive", { n: "12" }));
    expect(tagSummary(null, dive("s", "Reef"), undefined)).toBe(t("tag.sumDive", { n: "12" }));
  });

  it("falls back to the dive alone when the dive's site has no name", () => {
    // Known quirk, kept as is: the place is written but the summary does not say so.
    expect(tagSummary(null, dive("s", null), null)).toBe(t("tag.sumDive", { n: "12" }));
  });

  it("labels a dive without a number by its site name, then a dash", () => {
    expect(tagSummary(null, dive("s", "Reef", null), null)).toBe(t("tag.sumDive", { n: "Reef" }));
    expect(tagSummary(null, dive(null, null, null), null)).toBe(t("tag.sumDive", { n: "—" }));
  });

  it("names the place alone when no dive is written", () => {
    expect(tagSummary(null, null, "Reef")).toBe(t("tag.sumPlace", { site: "Reef" }));
    expect(tagSummary(null, null, undefined)).toBe(t("tag.sumPlace", { site: "" }));
  });
});

describe("canApplyTag", () => {
  it("needs no blocker and no write in progress", () => {
    expect(canApplyTag(null, false)).toBe(true);
    expect(canApplyTag(null, true)).toBe(false);
    expect(canApplyTag("Blocked", false)).toBe(false);
  });
});

describe("selectionChanged", () => {
  it("recognises a selection that moved on", () => {
    expect(selectionChanged({ error: "lightroom", reason: "selectionChanged", detail: null })).toBe(true);
    expect(selectionChanged({ error: "lightroom", reason: "staleSelection", detail: null })).toBe(true);
    expect(selectionChanged({ error: "lightroom", reason: "busy", detail: null })).toBe(false);
    expect(selectionChanged({ error: "noPhotos" } as never)).toBe(true);
    expect(selectionChanged({ error: "badGps" } as never)).toBe(false);
  });
});

describe("describeAgain", () => {
  it("describes the selection again after a Lightroom answer when per-photo values were asked", () => {
    const busy = { error: "lightroom", reason: "busy", detail: null } as const;
    expect(describeAgain(busy, true)).toBe(true);
    expect(describeAgain(busy, false)).toBe(false);
  });

  it("describes it again when the selection moved on, whatever was asked", () => {
    expect(describeAgain({ error: "noPhotos" }, false)).toBe(true);
    expect(describeAgain({ error: "lightroom", reason: "selectionChanged", detail: null }, false)).toBe(true);
  });

  it("leaves it as it is for a refusal before Lightroom was reached", () => {
    expect(describeAgain({ error: "badGps" }, true)).toBe(false);
    expect(describeAgain({ error: "notConnected" }, true)).toBe(false);
  });
});
