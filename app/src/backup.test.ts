import { beforeEach, describe, expect, it } from "vitest";
import { fileName, importErrorText } from "./backup";
import { setLanguage } from "./i18n";

beforeEach(() => setLanguage("en"));

describe("settings backup", () => {
  it("names the file, from a macOS or Windows path", () => {
    expect(fileName("/Users/diver/Desktop/Dive Lens settings.json")).toBe("Dive Lens settings.json");
    expect(fileName("C:\\Users\\diver\\Documents\\Réglages Dive Lens.json")).toBe("Réglages Dive Lens.json");
    expect(fileName("settings.json")).toBe("settings.json");
  });

  it("explains why a file was not imported", () => {
    expect(importErrorText({ kind: "notSettings" })).toBe("This file is not a Dive Lens settings file.");
    expect(importErrorText({ kind: "unreadable", detail: "permission denied" })).toBe(
      "This file cannot be read: permission denied",
    );
    expect(importErrorText({ kind: "notSaved", detail: "disk full" })).toBe("Settings not imported: disk full");
    expect(importErrorText("link lost")).toBe("Settings not imported: link lost");
  });

  it("speaks French too", () => {
    setLanguage("fr");
    expect(importErrorText({ kind: "notSettings" })).toBe("Ce fichier n'est pas un fichier de réglages Dive Lens.");
  });
});
