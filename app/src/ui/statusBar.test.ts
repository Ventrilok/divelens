import { beforeEach, describe, expect, it } from "vitest";
import { setLanguage, t } from "../i18n";
import { logItems, lightroomItems } from "./statusBar";
import { setApps } from "../sourceText";
import type { DiveApp, DiveLog, LightroomStatus } from "../types";

// A fictional dive application, described as the back end describes it.
const app = {
  id: "subsurface",
  name: "Subsurface",
  choose: { file: [], folder: true },
  texts: {},
} as unknown as DiveApp;

beforeEach(() => {
  setLanguage("en");
  setApps([app]);
});

const connected: LightroomStatus = {
  connected: true,
  catalog: "Voyage",
  lightroomVersion: null,
  pluginVersion: null,
  selected: 12,
  catalogPhotos: 48409,
  selectionVersion: 1,
  selectionFrom: null,
  selectionTo: null,
  coverFrom: null,
  coverTo: null,
  restartLightroom: false,
};

const logOf = (n: number) =>
  ({
    source: { kind: "subsurface" },
    dives: Array.from({ length: n }),
  }) as unknown as DiveLog;

describe("lightroomItems", () => {
  it("lists the catalog, its size and the selection when connected", () => {
    const g = lightroomItems(connected);
    expect(g.state).toBe("connected");
    expect(g.title).toBe("Lightroom connected");
    expect(g.hint).toBeNull();
    expect(g.items).toEqual([
      "Catalog: Voyage",
      "Catalog size: 48,409 items",
      "Selected photos: 12",
    ]);
  });

  it("speaks French", () => {
    setLanguage("fr");
    expect(lightroomItems(connected).items).toEqual([
      "Catalogue : Voyage",
      "Taille du catalogue : 48 409 éléments",
      "Photos sélectionnées : 12",
    ]);
  });

  it("says none when nothing is selected", () => {
    expect(lightroomItems({ ...connected, selected: 0 }).items[2]).toBe(
      "Selected photos: none",
    );
    setLanguage("fr");
    expect(lightroomItems({ ...connected, selected: 0 }).items[2]).toBe(
      "Photos sélectionnées : aucune",
    );
  });

  it("leaves the catalog size out until Lightroom has counted", () => {
    const items = lightroomItems({ ...connected, catalogPhotos: null }).items;
    expect(items).toEqual(["Catalog: Voyage", "Selected photos: 12"]);
  });

  it("shows only the link state when not connected", () => {
    const g = lightroomItems({ ...connected, connected: false });
    expect(g).toEqual({
      state: "disconnected",
      title: t("lr.disconnected"),
      hint: t("lr.disconnectedHint"),
      items: [],
    });
  });

  it("asks for a restart after an update", () => {
    const g = lightroomItems({
      ...connected,
      connected: false,
      restartLightroom: true,
    });
    expect(g).toEqual({
      state: "restart",
      title: t("lr.restart"),
      hint: t("lr.restartHint"),
      items: [],
    });
  });
});

describe("logItems", () => {
  it("names the dive log application and counts the dives", () => {
    expect(logItems(logOf(712))).toEqual([
      "Dive log app: Subsurface",
      "Dives logged: 712",
    ]);
    setLanguage("fr");
    expect(logItems(logOf(1))).toEqual([
      "Application de carnet : Subsurface",
      "Plongées enregistrées : 1",
    ]);
  });

  it("shows nothing without a dive log", () => {
    expect(logItems(null)).toEqual([]);
  });
});
