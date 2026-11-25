import { describe, expect, it } from "vitest";
import { isReadyScreen } from "./screens";

describe("isReadyScreen", () => {
  it("accepts the screens a request may open and nothing else", () => {
    expect(["journey", "settings"].every(isReadyScreen)).toBe(true);
    // The two menus before the journey: Lightroom no longer sends them.
    expect(isReadyScreen("collections")).toBe(false);
    expect(isReadyScreen("tag")).toBe(false);
    expect(isReadyScreen("nope")).toBe(false);
  });
});
