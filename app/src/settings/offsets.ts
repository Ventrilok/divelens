// Remembered clock offsets in Settings (spec F2): each camera named as the Clocks step
// names it on its card, sorted as the Clocks step sorts its cameras, with its offset
// written as the Clocks step writes it.

import { offsetField } from "../apply/cameras";
import { t } from "../i18n";
import type { RememberedOffset } from "../types";

/** One camera of the list. */
export interface OffsetLine {
  /** What "Forget" names. */
  key: string;
  /** The model, or "Unknown camera". */
  name: string;
  /** "no. A1": two bodies of the same model told apart; null without a serial number. */
  serial: string | null;
  /** "+4:01", "0:00", "−0:30". */
  offset: string;
}

const compare = (a: string, b: string) => (a < b ? -1 : a > b ? 1 : 0);

/** The cameras by name, as the Clocks step sorts them: model whatever the case, then serial number. */
export function offsetLines(remembered: RememberedOffset[]): OffsetLine[] {
  const model = (r: RememberedOffset) => (r.model ?? "").toLowerCase();
  return [...remembered]
    .sort((a, b) => compare(model(a), model(b)) || compare(a.serial ?? "", b.serial ?? ""))
    .map((r) => ({
      key: r.key,
      name: r.model ?? t("coll.unknownCamera"),
      serial: r.serial ? t("coll.serial", { n: r.serial }) : null,
      offset: offsetField(r.offset),
    }));
}
