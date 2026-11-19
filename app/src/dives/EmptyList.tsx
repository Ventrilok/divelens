// What the dive list shows when the filters hide every dive.

import { hasFilters, NO_FILTERS, type Filters } from "./filters";
import { t } from "../i18n";

export function EmptyList({ filters, onChange }: { filters: Filters; onChange: (f: Filters) => void }) {
  return (
    <div className="card list-card list-empty">
      <p className="muted">{t("filter.none")}</p>
      {hasFilters(filters) && (
        <button type="button" className="btn btn-s" onClick={() => onChange(NO_FILTERS)}>
          {t("filter.clearAll")}
        </button>
      )}
    </div>
  );
}
