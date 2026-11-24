import { describe, expect, it } from "vitest";
import { isReadyScreen, shownScreen } from "./screens";

describe("shownScreen", () => {
  it("shows the screen in use", () => {
    expect(shownScreen("tag", "collections")).toBe("tag");
    expect(shownScreen("collections", "tag")).toBe("collections");
  });

  it("keeps the screen Settings were opened from under Settings", () => {
    expect(shownScreen("settings", "tag")).toBe("tag");
    expect(shownScreen("settings", "collections")).toBe("collections");
  });

  it("falls back to the default screen when nothing was left", () => {
    expect(shownScreen("settings", "settings")).toBe("collections");
  });
});

describe("isReadyScreen", () => {
  it("accepts the screens and nothing else", () => {
    expect(["collections", "tag", "settings"].every(isReadyScreen)).toBe(true);
    expect(isReadyScreen("nope")).toBe(false);
  });
});
