import { beforeEach, describe, expect, it } from "vitest";
import { offsetLines } from "./offsets";
import { setLanguage } from "../i18n";
import type { RememberedOffset } from "../types";

beforeEach(() => setLanguage("en"));

const remembered = (model: string | null, serial: string | null, offset: number): RememberedOffset => ({
  key: `${model ?? ""}|${serial ?? ""}`,
  model,
  serial,
  offset,
});

describe("remembered clock offsets in Settings", () => {
  it("lists nothing when none is remembered", () => {
    expect(offsetLines([])).toEqual([]);
  });

  it("names each camera as the Clocks step does, with its offset as the Clocks step writes it", () => {
    expect(offsetLines([remembered("TG-7", "A1", 241), remembered("Nikon Z6", null, -30), remembered("GoPro", null, 0)])).toEqual([
      { key: "GoPro|", name: "GoPro", serial: null, offset: "0:00" },
      { key: "Nikon Z6|", name: "Nikon Z6", serial: null, offset: "−0:30" },
      { key: "TG-7|A1", name: "TG-7", serial: "no. A1", offset: "+4:01" },
    ]);
  });

  it("tells two bodies of one model apart by their serial number, in its order", () => {
    const lines = offsetLines([remembered("TG-7", "B2", 60), remembered("TG-7", "A1", 240)]);
    expect(lines.map((l) => [l.name, l.serial, l.offset])).toEqual([
      ["TG-7", "no. A1", "+4:00"],
      ["TG-7", "no. B2", "+1:00"],
    ]);
  });

  it("sorts by name whatever the case, as the Clocks step sorts its cameras", () => {
    const lines = offsetLines([remembered("olympus TG-6", null, 1), remembered("Nikon Z6", null, 2), remembered("OM System", null, 3)]);
    expect(lines.map((l) => l.name)).toEqual(["Nikon Z6", "olympus TG-6", "OM System"]);
  });

  it("names a camera without a model as the Clocks step does", () => {
    expect(offsetLines([remembered(null, "A1", 15)])).toEqual([{ key: "|A1", name: "Unknown camera", serial: "no. A1", offset: "+0:15" }]);
  });

  it("speaks French too", () => {
    setLanguage("fr");
    expect(offsetLines([remembered(null, "A1", -75)])).toEqual([{ key: "|A1", name: "Appareil inconnu", serial: "n° A1", offset: "−1:15" }]);
  });
});
