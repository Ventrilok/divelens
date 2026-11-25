// Steps 1 and 2 of the journey. Fictional dives and cameras only. Texts are
// compared word for word with the specification (F5). Runs in UTC+14
// (vitest.config.ts): every date below is a wall-clock date.
import { beforeEach, describe, expect, it } from "vitest";
import { setLanguage } from "../i18n";
import { catalogCard, catalogDivesBar, coversNote, photosBar, selectionCard, selectionDivesBar, selectionHead } from "./choose";
import { reduce, start } from "./state";
import type { Camera, Dive, LightroomStatus, SelectionPhotos } from "../types";

beforeEach(() => setLanguage("en"));

const dive = (id: string, number: number | null, site: string | null, start: string, end: string): Dive => ({
  id,
  number,
  start,
  end,
  durationS: 3000,
  maxDepthM: 18,
  tempHighC: 27,
  tempLowC: 26,
  rating: null,
  site: { id: site && `s-${id}`, name: site, location: null, country: null, lat: null, lon: null },
  hasProfile: true,
  invalid: false,
  notes: null,
});

const d1 = dive("d1", 12, "Blue Arch", "2026-03-06T09:00:00", "2026-03-06T09:50:00");
const d2 = dive("d2", 13, "Sand Patch", "2026-03-08T10:00:00", "2026-03-08T10:45:00");
const d3 = dive("d3", 11, "Coral Steps", "2026-03-03T23:30:00", "2026-03-04T00:20:00");
const dives = [d1, d2, d3];

type Live = Pick<LightroomStatus, "connected" | "selected" | "selectionCameras" | "selectionFrom" | "selectionTo">;

/** Five photos selected in Lightroom, one camera, on 6 March 2026. */
const five: Live = {
  connected: true,
  selected: 5,
  selectionCameras: [{ model: "Olympus TG-6", serial: null, from: "2026-03-06T09:10:00", to: "2026-03-06T09:40:00" }],
  selectionFrom: "2026-03-06T09:10:00",
  selectionTo: "2026-03-06T09:40:00",
};
const none: Live = { connected: true, selected: 0, selectionCameras: [], selectionFrom: null, selectionTo: null };
const offline: Live = { ...none, connected: false };

const catalog = reduce(start, { type: "choosePath", path: "catalog" });
const selection = reduce(start, { type: "choosePath", path: "selection" });

describe("catalogCard", () => {
  it("says what the entire catalog does, in both languages", () => {
    expect(catalogCard(none)).toEqual({
      title: "Entire catalog",
      text: "You tick one or more dives: each photo goes to the dive that matches its time.",
      sub: "For a whole trip or a series of dives.",
    });
    setLanguage("fr");
    expect(catalogCard(none)).toEqual({
      title: "Tout le catalogue",
      text: "Vous cochez une ou plusieurs plongées : chaque photo va dans la plongée qui correspond à son heure.",
      sub: "Pour un voyage entier ou une série de plongées.",
    });
  });

  it("says the Lightroom selection is ignored only while photos are selected", () => {
    expect(catalogCard(five).sub).toBe("For a whole trip or a series of dives. The Lightroom selection is ignored.");
    setLanguage("fr");
    expect(catalogCard(five).sub).toBe("Pour un voyage entier ou une série de plongées. La sélection Lightroom est ignorée.");
  });
});

describe("selectionCard", () => {
  it("names the photos selected, their cameras and day, and what the path does", () => {
    setLanguage("fr");
    expect(selectionCard(five)).toEqual({
      enabled: true,
      title: "Les 5 photos sélectionnées dans Lightroom",
      lines: [
        "Olympus TG-6 · vendredi 6 mars 2026",
        "Vous choisissez une plongée : les 5 photos y sont toutes rattachées, quelle que soit leur heure.",
      ],
      sub: null,
    });
    setLanguage("en");
    expect(selectionCard(five).title).toBe("Lightroom selection");
    expect(selectionCard(five).lines[1]).toBe("You choose one dive: all 5 photos are attached to it, whatever their time.");
  });

  it("one photo selected", () => {
    setLanguage("fr");
    const one = { ...five, selected: 1 };
    expect(selectionCard(one).title).toBe("La photo sélectionnée dans Lightroom");
    expect(selectionCard(one).lines[1]).toBe("Vous choisissez une plongée : la photo y est rattachée, quelle que soit son heure.");
  });

  it("is greyed while nothing is selected, and lights up as soon as photos are", () => {
    setLanguage("fr");
    expect(selectionCard(none)).toEqual({
      enabled: false,
      title: "Les photos sélectionnées dans Lightroom",
      lines: ["Aucune photo sélectionnée"],
      sub: "Sélectionnez-y des photos pour les rattacher toutes à une plongée de votre choix.",
    });
    setLanguage("en");
    expect(selectionCard(none).lines).toEqual(["No photos selected"]);
    expect(selectionCard(none).sub).toBe("Select photos there to attach them all to a dive of your choice.");
    expect(selectionCard(five).enabled).toBe(true);
  });

  it("is greyed while Lightroom is not connected", () => {
    expect(selectionCard(offline).enabled).toBe(false);
    expect(selectionCard({ ...five, connected: false }).enabled).toBe(false);
  });
});

describe("photosBar", () => {
  it("nothing chosen yet: nothing said", () => {
    expect(photosBar(start, five)).toEqual({ choice: null, next: null, prompt: null });
  });

  it("the entire catalog, then what comes next, in both languages", () => {
    expect(photosBar(catalog, none)).toEqual({ choice: "Entire catalog", next: "Next step: tick the dives", prompt: null });
    setLanguage("fr");
    expect(photosBar(catalog, five)).toEqual({
      choice: "Tout le catalogue",
      next: "Étape suivante : cocher les plongées",
      prompt: null,
    });
  });

  it("the photos selected, then what comes next", () => {
    expect(photosBar(selection, five)).toEqual({ choice: "The 5 selected photos", next: "Next step: choose their dive", prompt: null });
    setLanguage("fr");
    expect(photosBar(selection, five)).toEqual({
      choice: "Les 5 photos sélectionnées",
      next: "Étape suivante : choisir leur plongée",
      prompt: null,
    });
  });

  it("the selection path with nothing selected any more: only the prompt that greys Next", () => {
    expect(photosBar(selection, none)).toEqual({ choice: null, next: null, prompt: "Select photos in Lightroom" });
  });
});

describe("catalogDivesBar", () => {
  const at2 = reduce(catalog, { type: "next" });
  const all = new Set(dives.map((d) => d.id));

  it("asks to tick dives while none is", () => {
    expect(catalogDivesBar(at2, dives, all)).toEqual({ count: null, hidden: null, span: null, prompt: "Check one or more dives" });
    setLanguage("fr");
    expect(catalogDivesBar(at2, dives, all).prompt).toBe("Cochez une ou plusieurs plongées");
  });

  it("counts the dives ticked and gives their days, whatever the order of the log", () => {
    const j = reduce(at2, { type: "setDives", ids: ["d2", "d3"] });
    expect(catalogDivesBar(j, dives, all)).toEqual({
      count: "2 dives selected",
      hidden: null,
      span: "From 3 Mar 2026 to 8 Mar 2026 · photos looked for in the entire catalog",
      prompt: null,
    });
    setLanguage("fr");
    expect(catalogDivesBar(j, dives, all).count).toBe("2 plongées sélectionnées");
    expect(catalogDivesBar(j, dives, all).span).toBe("Du 3 mars 2026 au 8 mars 2026 · photos cherchées dans tout le catalogue");
  });

  it("tells how many dives ticked the filters hide, as before", () => {
    const j = reduce(at2, { type: "setDives", ids: ["d1", "d2"] });
    expect(catalogDivesBar(j, dives, new Set(["d1"])).hidden).toBe("including 1 hidden by the filters");
  });
});

describe("selectionHead", () => {
  it("says what choosing a dive does, in both languages", () => {
    expect(selectionHead(five)).toBe(
      "Choose their dive: all 5 photos will be attached to it, whatever their time. Without a dive, you will only set a location, chosen at step 4.",
    );
    setLanguage("fr");
    expect(selectionHead(five)).toBe(
      "Choisissez leur plongée : les 5 photos y seront toutes rattachées, quelle que soit leur heure. Sans plongée, vous poserez seulement un lieu, choisi à l'étape 4.",
    );
  });

  it("asks for photos when nothing is selected, or says Lightroom is away", () => {
    expect(selectionHead(none)).toBe("Select photos in Lightroom");
    expect(selectionHead(offline)).toBe("Lightroom not connected: nothing can be written");
  });
});

describe("selectionDivesBar", () => {
  const at2 = reduce(selection, { type: "next" });

  it("asks to choose a dive while none is chosen", () => {
    expect(selectionDivesBar(at2, dives, five)).toEqual({ choice: null, sub: null, prompt: "Choose a dive" });
    setLanguage("fr");
    expect(selectionDivesBar(at2, dives, five).prompt).toBe("Choisissez une plongée");
  });

  it("names the dive chosen and says the photos selected will be attached to it", () => {
    const j = reduce(at2, { type: "pickDive", pick: { kind: "dive", id: "d1" } });
    expect(selectionDivesBar(j, dives, five)).toEqual({
      choice: "12 Blue Arch",
      sub: "The 5 selected photos will be attached to it",
      prompt: null,
    });
    setLanguage("fr");
    expect(selectionDivesBar(j, dives, five).sub).toBe("Les 5 photos sélectionnées y seront rattachées");
    expect(selectionDivesBar(j, dives, { ...five, selected: 1 }).sub).toBe("La photo sélectionnée y sera rattachée");
  });

  it("no dive: says so, nothing is attached", () => {
    const j = reduce(at2, { type: "pickDive", pick: { kind: "noDive" } });
    expect(selectionDivesBar(j, dives, five)).toEqual({ choice: "No dive — location only", sub: null, prompt: null });
  });

  it("with nothing selected any more, says nothing about attaching", () => {
    const j = reduce(at2, { type: "pickDive", pick: { kind: "dive", id: "d1" } });
    expect(selectionDivesBar(j, dives, none).sub).toBeNull();
  });
});

describe("coversNote", () => {
  const tg6: Camera = { key: "a", model: "Olympus TG-6", serial: null, photos: 4, remembered: 240 };
  const described = (cameras: Camera[], photos = 5, undated = 1): SelectionPhotos => ({ search: 3, photos, undated, cameras });

  it("explains the mark with the remembered offset of the one camera, in both languages", () => {
    const why = coversNote(d1, described([tg6]), { a: 240 }, new Set());
    expect(why).toBe("Covers the selection: 4 of the 5 photos fall in this dive, with the remembered offset of the Olympus TG-6 (+4:00).");
    setLanguage("fr");
    expect(coversNote(d1, described([tg6]), { a: 240 }, new Set())).toBe(
      "Couvre la sélection : 4 des 5 photos tombent dans cette plongée, avec le décalage mémorisé de l'Olympus TG-6 (+4:00).",
    );
  });

  it("says nothing without a dive covering the selection, or before the selection is read", () => {
    expect(coversNote(null, described([tg6]), { a: 240 }, new Set())).toBeNull();
    expect(coversNote(d1, null, { a: 240 }, new Set())).toBeNull();
  });

  it("says nothing where the sentence would be wrong: no remembered offset, another offset shown, several cameras", () => {
    expect(coversNote(d1, described([{ ...tg6, remembered: null }]), { a: 0 }, new Set())).toBeNull();
    expect(coversNote(d1, described([tg6]), { a: 180 }, new Set(["a"]))).toBeNull();
    expect(coversNote(d1, described([tg6]), { a: 240 }, new Set(["a"]))).toBeNull();
    const phone: Camera = { key: "b", model: "Phone", serial: null, photos: 1, remembered: 0 };
    expect(coversNote(d1, described([tg6, phone]), { a: 240, b: 0 }, new Set())).toBeNull();
  });

  it("names a camera without a model as unknown, and a negative offset with its sign", () => {
    const unknown: Camera = { key: "c", model: null, serial: null, photos: 5, remembered: -90 };
    expect(coversNote(d1, described([unknown], 5, 0), { c: -90 }, new Set())).toBe(
      "Covers the selection: 5 of the 5 photos fall in this dive, with the remembered offset of the Unknown camera (−1:30).",
    );
  });
});
