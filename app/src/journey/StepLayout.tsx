// The frame of each step of the journey (spec F5), under the step strip: the
// step's content, which scrolls, then its bottom bar, always in view at the
// bottom of the window, just above the status bar. Nothing ever sits under
// the bar, whatever the window's height. The same for the five steps, so that
// they never drift apart.
//
// Step 2's dive list and detail card fill the height left and scroll on their
// own; below a usable height (App.css, .dives-body), the whole step scrolls.

import type { ReactNode } from "react";

export function StepLayout({
  children,
  beforeBar,
  bar,
}: {
  children: ReactNode;
  /** Shown between the content and the bar, always in view (Apply's failure). */
  beforeBar?: ReactNode;
  /** The bar's text and buttons. */
  bar: ReactNode;
}) {
  return (
    <>
      <div className="step-body">
        <div className="step-content">{children}</div>
      </div>
      {beforeBar}
      <div className="card selection-bar step-bar">{bar}</div>
    </>
  );
}
