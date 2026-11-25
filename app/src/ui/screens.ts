/** Screens a request (a Lightroom menu, the Plug-in Manager, an update) may open. */
export type RequestedScreen = "journey" | "settings";

/** The journey, or Settings in its place. */
export type Screen = RequestedScreen;

/** Whether a request names a screen it may open; anything else is ignored. */
export function isReadyScreen(id: string): id is RequestedScreen {
  return id === "journey" || id === "settings";
}

/** The screen a request opens: "journey" the journey (at step 1, its choices kept), "settings" Settings. */
export function requestedScreen(id: RequestedScreen): Screen {
  return id;
}
