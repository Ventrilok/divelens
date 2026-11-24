export type Screen = "collections" | "tag" | "settings";

/** Screens a request (a Lightroom menu) may open; anything else is ignored. */
export function isReadyScreen(id: string): id is Screen {
  return id === "settings" || id === "collections" || id === "tag";
}

/** The screen of the journey that is mounted: the one in use, or, under Settings, the one they were opened from. */
export function shownScreen(screen: Screen, before: Screen): Exclude<Screen, "settings"> {
  const s = screen === "settings" ? before : screen;
  return s === "settings" ? "collections" : s;
}
