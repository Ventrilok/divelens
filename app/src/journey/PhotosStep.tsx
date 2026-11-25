// Step 1 of the journey, "Photos" (spec F5): which photos to work on, two
// cards separated by "or". "Entire catalog" leads to ticking dives; "The N
// photos selected in Lightroom" to choosing one dive, with the selection's
// cameras, day and live thumbnails (up to six). That card is always shown,
// greyed while nothing is selected, and lights up as soon as photos are.
//
// Rendered inside the journey's screen, under the step strip: the notice of a
// dive log that changed comes first, then the cards and the bottom bar.

import { useEffect, useState } from "react";
import { onSelectionPreview, selectionPreview } from "../api";
import { t } from "../i18n";
import { Icon } from "../ui/Icon";
import { LogNotice } from "../ui/LogNotice";
import type { LightroomStatus, SelectionPreview } from "../types";
import { catalogCard, photosBar, selectionCard } from "./choose";
import { canGoNext } from "./rules";
import type { Action, Journey } from "./state";

/** Thumbnails shown at most on the selection's card; "+N" for the others. */
const THUMBNAILS = 6;

/** Live thumbnails of the photos selected, as they arrive. */
function Thumbnails({ lr }: { lr: LightroomStatus }) {
  const [preview, setPreview] = useState<SelectionPreview>({ version: 0, thumbnails: [] });
  useEffect(() => {
    selectionPreview().then(setPreview);
    const stop = onSelectionPreview(setPreview);
    return () => {
      stop.then((f) => f());
    };
  }, []);
  const thumbnails = preview.version === lr.selectionVersion ? preview.thumbnails : [];
  return (
    <span className="thumbs" aria-hidden="true">
      {Array.from({ length: Math.min(THUMBNAILS, lr.selected) }, (_, i) =>
        thumbnails[i] ? (
          <img key={i} className="thumb" src={thumbnails[i] as string} alt="" />
        ) : (
          <span key={i} className="thumb" />
        ),
      )}
      {lr.selected > THUMBNAILS && <span className="thumb thumb-more">+{lr.selected - THUMBNAILS}</span>}
    </span>
  );
}

export function PhotosStep({
  journey,
  dispatch,
  lr,
  notice,
  onDismissNotice,
}: {
  journey: Journey;
  dispatch: (a: Action) => void;
  lr: LightroomStatus;
  /** Why the journey started again (the dive log changed), shown once. */
  notice: string | null;
  onDismissNotice: () => void;
}) {
  const catalog = catalogCard(lr);
  const selection = selectionCard(lr);
  const bar = photosBar(journey, lr);
  const next = canGoNext(journey, lr);
  const onCatalog = journey.path === "catalog";
  const onSelection = journey.path === "selection";

  return (
    <>
      {notice && <LogNotice text={notice} onDismiss={onDismissNotice} />}
      <div className="photos-head">
        <h2>{t("journey.photos.title")}</h2>
        <p className="muted small">{t("journey.photos.hint")}</p>
      </div>
      <div className="choices" role="radiogroup" aria-label={t("journey.steps.photos")}>
        <label className={`choice ${onCatalog ? "choice-on" : ""}`}>
          <input
            type="radio"
            name="journey-photos"
            checked={onCatalog}
            onChange={() => dispatch({ type: "choosePath", path: "catalog" })}
          />
          <span className="choice-text">
            <b>{catalog.title}</b>
            <span>{catalog.text}</span>
            <span className="muted">{catalog.sub}</span>
          </span>
        </label>
        <div className="choices-or" aria-hidden="true">
          {t("journey.photos.or")}
        </div>
        <label className={`choice ${onSelection && selection.enabled ? "choice-on" : ""} ${selection.enabled ? "" : "choice-off"}`}>
          <input
            type="radio"
            name="journey-photos"
            checked={onSelection}
            disabled={!selection.enabled}
            onChange={() => dispatch({ type: "choosePath", path: "selection" })}
          />
          <span className="choice-text">
            <b>{selection.title}</b>
            {selection.lines.map((line) => (
              <span key={line}>{line}</span>
            ))}
            {selection.sub && <span className="muted">{selection.sub}</span>}
            {selection.enabled && <Thumbnails lr={lr} />}
          </span>
        </label>
      </div>

      <div className="card selection-bar apply-bar">
        <div className="selection-text">
          {bar.choice && <span className="strong">{bar.choice}</span>}
          {bar.next && <span className="muted small">{bar.next}</span>}
          {bar.prompt && <span className="muted small">{bar.prompt}</span>}
        </div>
        <div className="selection-actions">
          <button type="button" className="btn btn-p" disabled={!next.allowed} onClick={() => dispatch({ type: "next" })}>
            {t("sel.next")}
            <Icon name="arrow" />
          </button>
        </div>
      </div>
    </>
  );
}
