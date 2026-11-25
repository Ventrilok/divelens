import { describe, expect, it } from "vitest";
import { isReadyScreen, requestedScreen, shownScreen } from "./screens";

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
  it("accepts the screens a request may open and nothing else", () => {
    expect(["journey", "settings"].every(isReadyScreen)).toBe(true);
    // The two menus before the journey: Lightroom no longer sends them.
    expect(isReadyScreen("collections")).toBe(false);
    expect(isReadyScreen("tag")).toBe(false);
    expect(isReadyScreen("nope")).toBe(false);
  });
});

describe("requestedScreen", () => {
  it("opens the journey on Create Collections, Settings on Settings", () => {
    expect(requestedScreen("journey")).toBe("collections");
    expect(requestedScreen("settings")).toBe("settings");
  });
});
