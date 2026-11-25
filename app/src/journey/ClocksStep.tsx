// Step 3 of the journey, "Clocks" (spec F5), one screen for both paths: one
// clock offset per camera, then the preview by dive. Catalog path: the
// cameras found over the period of the dives ticked, "Ignore", the surface
// margin and the photos outside the dives. Selection path: the cameras of the
// photos selected, all attached to the dive chosen whatever their time, the
// clock only giving them a depth; no "Ignore", no margin.
//
// Rendered inside the journey's screen, under the step strip: it brings the
// line "Photos: …", its blocks and the bottom bar. The two photo readings
// are the journey's (useCatalogPhotos, useSelectionPhotos), so that what the
// diver sets here survives going back and forth between steps.

import type { ReactNode } from "react";
import { MARGIN_STEP_MIN, MAX_MARGIN_MIN } from "../apply/cameras";
import { CameraClock } from "../apply/CameraClock";
import { formatInt, t, tn } from "../i18n";
import { Icon } from "../ui/Icon";
import type { Dive, DiveLog, LightroomStatus } from "../types";
import { catalogBar, catalogOutside, catalogRows, selectionBar, selectionOutside, selectionRow, selectionWaiting } from "./clocks";
import { DivePreview } from "./DivePreview";
import { canGoNext } from "./rules";
import { chosenDive, tickedDives } from "./rules/shared";
import type { Journey, Path } from "./state";
import { diveLabel, photosLine } from "./texts";
import type { CatalogReading } from "./useCatalogPhotos";
import type { SelectionReading } from "./useSelectionPhotos";

/** "Camera clocks": its title and text, "Suggest for all" when given, then the cards or what they wait for. */
function ClockBlock({ onSuggestAll, children }: { onSuggestAll: (() => void) | null; children: ReactNode }) {
  return (
    <section className="card coll-sec" aria-labelledby="h-clock">
      <div className="coll-sec-head">
        <div>
          <h2 id="h-clock">{t("coll.clock.title")}</h2>
          <p className="muted small">{t("coll.clock.text")}</p>
        </div>
        {onSuggestAll && (
          <button type="button" className="btn btn-s" onClick={onSuggestAll}>
            <Icon name="spark" />
            {t("coll.suggestAll")}
          </button>
        )}
      </div>
      {children}
    </section>
  );
}

function Retry({ message, onRetry }: { message: string; onRetry: () => void }) {
  return (
    <div className="notice notice-warn" role="alert">
      <span>{message}</span>
      <button type="button" className="btn btn-g" onClick={onRetry}>
        {t("coll.retry")}
      </button>
    </div>
  );
}

/** Catalog path: "Include photos taken at the surface", up to N minutes before and after the dive. */
function MarginBlock({ margin, onChange }: { margin: number; onChange: (minutes: number) => void }) {
  return (
    <section className="card coll-sec coll-margin" aria-labelledby="h-margin">
      <div className="coll-margin-text">
        <h2 id="h-margin">{t("coll.margin.title")}</h2>
        <p className="muted small">{t("journey.clocks.marginText")}</p>
      </div>
      <div className="coll-margin-ctl">
        <span>{t("coll.margin.upTo")}</span>
        <button
          type="button"
          className="step"
          aria-label={t("coll.margin.less")}
          disabled={margin <= 0}
          onClick={() => onChange(margin - MARGIN_STEP_MIN)}
        >
          −
        </button>
        <span className="coll-margin-n">{margin} min</span>
        <button
          type="button"
          className="step"
          aria-label={t("coll.margin.more")}
          disabled={margin >= MAX_MARGIN_MIN}
          onClick={() => onChange(margin + MARGIN_STEP_MIN)}
        >
          +
        </button>
        <span>{t("coll.margin.around")}</span>
      </div>
    </section>
  );
}

function CatalogClocks({ c, dives }: { c: CatalogReading; dives: Dive[] }) {
  if (c.search.state === "searching") return <p className="muted">{t("coll.searching")}</p>;
  if (c.search.state === "failed") return <Retry message={c.search.message} onRetry={c.searchAgain} />;
  if (!c.period) return null;
  const counts = new Map(c.preview?.cameras.map((k) => [k.key, k]) ?? []);
  return (
    <>
      <ClockBlock onSuggestAll={c.cameras.length > 0 ? c.suggestAll : null}>
        {c.cameras.length === 0 ? (
          <p className="muted">{t("coll.noPhotos")}</p>
        ) : (
          <CameraClock
            cameras={c.cameras}
            offsets={c.offsets}
            touched={c.touched}
            notes={c.notes}
            counts={(key) => counts.get(key)}
            gauge={(inDives, outside) =>
              inDives === 0 ? t("coll.noneInDives") : t("coll.inDives", { n: formatInt(inDives), out: formatInt(outside) })
            }
            ignored={c.ignored}
            onChange={c.setOffset}
            onIgnore={c.setIgnored}
            onSuggest={c.suggest}
            disabled={false}
          />
        )}
      </ClockBlock>
      <MarginBlock margin={c.margin} onChange={c.setMargin} />
      <DivePreview rows={catalogRows(dives, c.preview)} outside={catalogOutside(c.cameras, c.preview)} />
    </>
  );
}

function SelectionClocks({ s, dive, lr }: { s: SelectionReading; dive: Dive; lr: LightroomStatus }) {
  const cameras = s.described?.photos.cameras ?? [];
  const counts = s.counts;
  const waiting = selectionWaiting(s, lr);
  // The counts the cards show, once they show.
  const shown = !s.readFailure && !waiting && s.described && counts ? counts : null;
  // Without a profile that can be read, one line in place of the camera clocks.
  const oneLine = !dive.hasProfile ? t("tag.perPhoto.noProfile") : s.unreadable ? t("tag.perPhoto.unreadable") : null;
  return (
    <>
      {lr.selected > 0 && (
        <div role="note" className="card clocks-note">
          <Icon name="info" className="muted" />
          <span>{tn("journey.clocks.selectionHead", lr.selected, { dive: diveLabel(dive) })}</span>
        </div>
      )}
      {oneLine ? (
        <section className="card coll-sec">
          <p>{oneLine}</p>
        </section>
      ) : (
        <ClockBlock onSuggestAll={shown && cameras.length > 0 ? s.suggestAll : null}>
          {s.readFailure ? (
            <Retry message={s.readFailure} onRetry={s.describeAgain} />
          ) : waiting ? (
            <p className="muted small">{waiting}</p>
          ) : (
            shown &&
            cameras.length > 0 && (
              <CameraClock
                cameras={cameras}
                offsets={s.offsets}
                touched={s.touched}
                notes={s.notes}
                counts={(key) => shown.cameras.find((k) => k.key === key)}
                gauge={(inDive, outside) =>
                  t("journey.clocks.inDive", { n: formatInt(inDive), out: formatInt(outside) })
                }
                onChange={s.setOffset}
                onSuggest={s.suggest}
                disabled={false}
              />
            )
          )}
        </ClockBlock>
      )}
      <DivePreview
        rows={[selectionRow(dive, counts, s.unreadable)]}
        outside={s.needsValues ? selectionOutside(cameras, counts) : null}
      />
    </>
  );
}

export function ClocksStep({
  path,
  journey,
  catalog,
  selection,
  log,
  lr,
  onBack,
  onNext,
}: {
  path: Path;
  journey: Journey;
  catalog: CatalogReading;
  selection: SelectionReading;
  log: DiveLog;
  lr: LightroomStatus;
  onBack: () => void;
  onNext: () => void;
}) {
  const onCatalog = path === "catalog";
  const line = photosLine(journey, { lr, dives: log.dives, period: onCatalog ? catalog.period : null });
  const dives = onCatalog ? tickedDives(journey, log.dives) : [];
  const dive = onCatalog ? null : chosenDive(journey, log.dives);
  const next = canGoNext(journey, lr);

  // The bar: the total in bold, then what goes with it.
  let main: { text: string; strong: boolean } | null = null;
  let notes: { text: string; warn: boolean }[] = [];
  if (onCatalog) {
    // Counts only for photos found: none while Lightroom is away or the photos are looked for again.
    const bar = catalogBar(dives, catalog.period ? catalog.preview : null);
    if (bar.total) main = { text: bar.total, strong: true };
    notes = bar.emptyDives.map((text) => ({ text, warn: true }));
  } else if (dive) {
    const bar = selectionBar(dive, selection, lr);
    if (bar.total) main = { text: bar.total, strong: true };
    else if (bar.waiting) main = { text: bar.waiting, strong: false };
    if (bar.hint) notes = [{ text: bar.hint, warn: false }];
  }

  return (
    <>
      {line && <p className="journey-line">{line}</p>}
      {!lr.connected && (
        <div className="notice notice-warn" role="status">
          <span>{t(onCatalog ? "coll.disconnected" : "tag.disconnected")}</span>
        </div>
      )}
      {onCatalog ? (
        <CatalogClocks c={catalog} dives={dives} />
      ) : (
        dive && <SelectionClocks s={selection} dive={dive} lr={lr} />
      )}
      <div className="card selection-bar apply-bar">
        <div className="selection-text">
          {main && <span className={main.strong ? "strong" : "muted"}>{main.text}</span>}
          {notes.map((n, i) => (
            <span key={i} className={`small ${n.warn ? "warn" : "muted"}`}>
              {n.text}
            </span>
          ))}
          {next.prompt && <span className="muted small">{next.prompt}</span>}
        </div>
        <div className="selection-actions">
          <button type="button" className="btn btn-s" onClick={onBack}>
            <Icon name="back" />
            {t("coll.back")}
          </button>
          <button type="button" className="btn btn-p" disabled={!next.allowed} onClick={onNext}>
            {t("sel.next")}
            <Icon name="arrow" />
          </button>
        </div>
      </div>
    </>
  );
}
