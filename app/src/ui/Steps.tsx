// Top of each step of the journey (spec F5): the five steps in a full-width
// frame, joined by a line like a checkout. A step already done can be clicked
// to go back to it, when `onGo` is given (never from the summary, never while
// writing); a step skipped (the clocks, for "no dive") is greyed.

import type { ReactNode } from "react";
import { t, type Key } from "../i18n";
import type { Step, StepState } from "../journey/state";
import { Icon } from "./Icon";

const labels: Key[] = ["journey.steps.photos", "journey.steps.dives", "journey.steps.clocks", "journey.steps.review", "steps.summary"];

export function Steps({ states, onGo }: { states: StepState[]; onGo?: (step: Step) => void }) {
  return (
    <ol className="card flow">
      {labels.map((key, i) => {
        const n = (i + 1) as Step;
        const state = states[i] ?? "next";
        const label: ReactNode = (
          <>
            <span className="flow-n">{state === "done" ? <Icon name="check" size={12} /> : n}</span>
            <span className="flow-label">{t(key)}</span>
          </>
        );
        return (
          <li
            key={key}
            className={`flow-step flow-${state}`}
            aria-current={state === "on" ? "step" : undefined}
            aria-disabled={state === "skipped" ? true : undefined}
          >
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
  );
}
