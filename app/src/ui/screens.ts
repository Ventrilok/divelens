export type Screen = "collections" | "tag" | "settings";

/** Screens a request (a Lightroom menu, the Plug-in Manager, an update) may open. */
export type RequestedScreen = "journey" | "settings";

/** Whether a request names a screen it may open; anything else is ignored. */
export function isReadyScreen(id: string): id is RequestedScreen {
  return id === "journey" || id === "settings";
}

/**
 * The screen a request opens. Temporary, until the journey replaces the two
 * screens: "journey" opens Create Collections, at its first step.
 */
export function requestedScreen(id: RequestedScreen): Screen {
  return id === "journey" ? "collections" : id;
}

/** The screen of the journey that is mounted: the one in use, or, under Settings, the one they were opened from. */
export function shownScreen(screen: Screen, before: Screen): Exclude<Screen, "settings"> {
  const s = screen === "settings" ? before : screen;
  return s === "settings" ? "collections" : s;
}
