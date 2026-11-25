// Step 5 of the journey, "Summary" (spec F5), one screen for both paths: the
// run recalled, what was written in Lightroom (photos tagged, with depth,
// collections, then each collection under the place chosen in Settings), the
// warnings and the Undo reminder. Catalog path: "Back to scope selection",
// and for the dives that got no photo "Handle … separately"; selection path:
// "Done". Both main buttons start a new run at step 1.
// Taken over from the result of Create Collections.
//
// Rendered inside the journey's screen, under the step strip (which no longer
// leads back from here).

import { placeLabel } from "../collections/collectionPlace";
import { formatInt, t } from "../i18n";
import { Icon } from "../ui/Icon";
import type { DiveLog, Settings } from "../types";
import { summary, type Applied } from "./rules";
import type { Journey } from "./state";
import { StepLayout } from "./StepLayout";
import { collectionLine, emptyDives, separatelyLabel, summaryNotices, writtenCollections } from "./summary";

export function SummaryStep({
  applied,
  journey,
  log,
  settings,
  onHandleSeparately,
  onDone,
}: {
  applied: Applied;
  journey: Journey;
  log: DiveLog;
  settings: Settings;
  /** Catalog path: to step 3 with only these dives ticked, the photos looked for again. */
  onHandleSeparately: (ids: string[]) => void;
  /** Both paths: a new run at step 1, the path kept, nothing ticked or chosen. */
  onDone: () => void;
}) {
  const i = { journey, applied, dives: log.dives };
  const s = summary(i);
  const collections = writtenCollections(applied);
  const empty = emptyDives(applied, log.dives);
  const separately = separatelyLabel(empty);
  return (
    <StepLayout
      bar={
        <>
          <div className="selection-text">
            <span className="muted small">{t("coll.undoHint")}</span>
          </div>
          <div className="selection-actions">
            {applied.path === "catalog" ? (
              <>
                {separately && (
                  <button type="button" className="btn btn-s" onClick={() => onHandleSeparately(empty.map((d) => d.id))}>
                    {separately}
                  </button>
                )}
                <button type="button" className="btn btn-p" onClick={onDone}>
                  {t("journey.summary.backToScope")}
                </button>
              </>
            ) : (
              <button type="button" className="btn btn-p" onClick={onDone}>
                {t("journey.summary.done")}
              </button>
            )}
          </div>
        </>
      }
    >
      <p className="muted">{s.head}</p>
      <div className="result">
        <div className="result-head">
          <div className="result-icon">
            <Icon name="check" size={26} />
          </div>
          <h2>{t("journey.summary.written")}</h2>
        </div>
        <div className="stats">
          {s.stats.map((stat) => (
            <div key={stat.label} className="card stat">
              <div className="lbl">{stat.label}</div>
              <div className="stat-n">{formatInt(stat.n)}</div>
            </div>
          ))}
        </div>
        {collections.length > 0 && (
          <section className="card result-list" aria-labelledby="h-set">
            <h2 id="h-set">{placeLabel(settings.collections, t("settings.root"))}</h2>
            <ul>
              {collections.map((c) => (
                <li key={c.collection}>
                  <span>{c.collection}</span>
                  <span className="muted">{collectionLine(c)}</span>
                </li>
              ))}
            </ul>
          </section>
        )}
        {summaryNotices(i).map((n, k) => (
          <div key={k} className={`notice ${n.warn ? "notice-warn" : "card"}`} role="status">
            <span>{n.text}</span>
          </div>
        ))}
      </div>
    </StepLayout>
  );
}
