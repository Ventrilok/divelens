export type Screen = "collections" | "tag" | "settings";

/** Screens a request (a Lightroom menu) may open; anything else is ignored. */
export function isReadyScreen(id: string): id is Screen {
  return id === "settings" || id === "collections" || id === "tag";
}
