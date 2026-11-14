// One camera and its clock offset (spec F5), as Create Collections step 2
// and Tag Selection show it: photo count, offset slider and field,
// "Suggest", and a gauge of its photos in the dives. Ignoring the camera is
// offered only where the screen allows it.

import { useEffect, useState } from "react";
import { clampOffset, offsetField, parseOffset, SLIDER_RANGE_MIN, SLIDER_STEP_MIN } from "../cameras";
import { t, tn } from "../i18n";
import type { Camera } from "../types";
import { Icon } from "./Icon";

/** Offset typed to the minute; arrows add or remove a minute. */
function OffsetField({ offset, label, onOffset }: { offset: number; label: string; onOffset: (min: number) => void }) {
  const [text, setText] = useState(offsetField(offset));
  const [editing, setEditing] = useState(false);
  useEffect(() => {
    if (!editing) setText(offsetField(offset));
  }, [offset, editing]);
  const commit = () => {
    const value = parseOffset(text);
    if (value !== null) onOffset(value);
    else setText(offsetField(offset));
    setEditing(false);
  };
  const bad = editing && parseOffset(text) === null;
  return (
    <input
      className="inp off-field"
      aria-label={label}
      aria-invalid={bad}
      value={text}
      onFocus={() => setEditing(true)}
      onChange={(e) => setText(e.target.value)}
      onBlur={commit}
      onKeyDown={(e) => {
        if (e.key === "Enter") e.currentTarget.blur();
        else if (e.key === "Escape") {
          setText(offsetField(offset));
          setEditing(false);
          e.currentTarget.blur();
        } else if (e.key === "ArrowUp" || e.key === "ArrowDown") {
          e.preventDefault();
          const next = offset + (e.key === "ArrowUp" ? 1 : -1);
          onOffset(next);
          setText(offsetField(clampOffset(next)));
        }
      }}
    />
  );
}

export function CameraCard({
  camera,
  offset,
  remembered,
  inDives,
  outside,
  gauge,
  note,
  onOffset,
  onSuggest,
  ignored = false,
  onIgnore,
}: {
  camera: Camera;
  offset: number;
  /** The offset shown is the one last applied to this camera. */
  remembered: boolean;
  /** Photos in the dives, and the other photos of the camera. */
  inDives: number;
  outside: number;
  /** Text next to the gauge. */
  gauge: string;
  /** Said under the card (no offset to suggest), or null. */
  note: string | null;
  onOffset: (min: number) => void;
  onSuggest: () => void;
  ignored?: boolean;
  /** Without it, the camera cannot be ignored. */
  onIgnore?: (ignored: boolean) => void;
}) {
  const warn = !ignored && inDives === 0;
  const name = camera.model ?? t("coll.unknownCamera");
  const total = inDives + outside;
  return (
    <div className={`cam ${warn ? "cam-warn" : ""} ${ignored ? "cam-off" : ""}`}>
      <div className="cam-h">
        {warn ? (
          <Icon name="warn" className="warn" />
        ) : ignored ? (
          <Icon name="close" className="muted" />
        ) : (
          <Icon name="check" className="accent" />
        )}
        <div className="cam-n">
          <b>{name}</b>
          <span>
            {camera.serial && `${t("coll.serial", { n: camera.serial })} · `}
            {tn("coll.photos", camera.photos)}
          </span>
        </div>
        {onIgnore && (
          <label className="ign">
            <input type="checkbox" checked={ignored} onChange={(e) => onIgnore(e.target.checked)} />
            {t("coll.ignore")}
          </label>
        )}
      </div>
      {ignored ? (
        <p className="muted small">{t("coll.ignored")}</p>
      ) : (
        <>
          <div className="cam-c">
            <div className="slider">
              <input
                type="range"
                min={-SLIDER_RANGE_MIN}
                max={SLIDER_RANGE_MIN}
                step={SLIDER_STEP_MIN}
                value={Math.max(-SLIDER_RANGE_MIN, Math.min(SLIDER_RANGE_MIN, offset))}
                aria-label={t("coll.offset", { camera: name })}
                aria-valuetext={offsetField(offset)}
                onChange={(e) => onOffset(Number(e.target.value))}
              />
              <div className="slider-scale" aria-hidden="true">
                <span>−14 h</span>
                <span>0</span>
                <span>+14 h</span>
              </div>
            </div>
            <OffsetField offset={offset} label={t("coll.offsetField", { camera: name })} onOffset={onOffset} />
            <button type="button" className="mini" onClick={onSuggest}>
              <Icon name="spark" size={15} />
              {t("coll.suggest")}
            </button>
          </div>
          {remembered && <span className="cam-tag">{t("coll.remembered")}</span>}
          <div className={`cam-f ${warn ? "warn" : ""}`}>
            <span className="bar">
              <span className="bar-fill" style={{ width: `${total > 0 ? (100 * inDives) / total : 0}%` }} />
            </span>
            <span>{gauge}</span>
          </div>
          {note && <p className="muted small">{note}</p>}
        </>
      )}
    </div>
  );
}
