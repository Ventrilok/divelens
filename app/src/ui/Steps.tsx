// Top of each Create Collections step: the title, then the three steps in a
// full-width frame, joined by a line like a checkout. A step already done can
// be clicked to go back to it, when `onGo` is given (never from the summary).

import type { ReactNode } from "react";
import { t, type Key } from "../i18n";
import { Icon } from "./Icon";

const steps: Key[] = ["steps.dives", "steps.adjust", "steps.summary"];

export function Steps({ current, onGo }: { current: 1 | 2 | 3; onGo?: (step: number) => void }) {
  return (
    <div className="flow-head">
      <h1>{t("coll.title")}</h1>
      <ol className="card flow" aria-label={t("coll.title")}>
      {steps.map((key, i) => {
        const n = i + 1;
        const state = n < current ? "done" : n === current ? "on" : "next";
        const label: ReactNode = (
          <>
            <span className="flow-n">{state === "done" ? <Icon name="check" size={12} /> : n}</span>
            <span className="flow-label">{t(key)}</span>
          </>
        );
        return (
          <li key={key} className={`flow-step flow-${state}`} aria-current={state === "on" ? "step" : undefined}>
            {state === "done" && onGo ? (
              <button type="button" className="flow-link" onClick={() => onGo(n)}>
                {label}
              </button>
            ) : (
              <span className="flow-link">{label}</span>
            )}
          </li>
        );
      })}
      </ol>
    </div>
  );
}
