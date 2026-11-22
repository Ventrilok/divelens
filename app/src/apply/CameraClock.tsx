// The cards of the cameras' clock offsets (spec F5), as Create Collections step 2 and
// Tag Selection show them. The screens keep what differs around them (title, "Suggest
// all", the empty case); ignoring a camera is offered only when `onIgnore` is given.

import { CameraCard } from "./CameraCard";
import type { Camera } from "../types";

export function CameraClock({
  cameras,
  offsets,
  touched,
  notes,
  counts,
  gauge,
  onChange,
  onSuggest,
  ignored,
  onIgnore,
  disabled,
}: {
  cameras: Camera[];
  offsets: Record<string, number>;
  /** Cameras whose offset the diver changed: their offset is no longer the remembered one. */
  touched: Set<string>;
  notes: Record<string, string>;
  /** How many photos of a camera are in the dives and outside them; undefined: not counted yet. */
  counts: (key: string) => { inDives: number; outside: number } | undefined;
  /** The gauge's text, from the photos in the dives and outside them. */
  gauge: (inDives: number, outside: number) => string;
  onChange: (key: string, minutes: number) => void;
  onSuggest: (key: string) => void;
  ignored?: Set<string>;
  onIgnore?: (key: string, on: boolean) => void;
  disabled: boolean;
}) {
  return (
    <div className="cams">
      {cameras.map((c) => {
        const inDives = counts(c.key)?.inDives ?? 0;
        const outside = counts(c.key)?.outside ?? 0;
        return (
          <CameraCard
            key={c.key}
            camera={c}
            offset={offsets[c.key] ?? 0}
            ignored={ignored?.has(c.key)}
            remembered={c.remembered !== null && offsets[c.key] === c.remembered && !touched.has(c.key)}
            inDives={inDives}
            outside={outside}
            gauge={gauge(inDives, outside)}
            note={notes[c.key] || null}
            onOffset={(m) => onChange(c.key, m)}
            onIgnore={onIgnore && ((on) => onIgnore(c.key, on))}
            onSuggest={() => onSuggest(c.key)}
            disabled={disabled}
          />
        );
      })}
    </div>
  );
}
