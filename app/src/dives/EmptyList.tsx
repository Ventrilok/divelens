// What the dive list shows when the filters hide every dive.

import { hasFilters, NO_FILTERS, type Filters } from "./filters";
import { t } from "../i18n";

interface Props {
  filters: Filters;
  onChange: (f: Filters) => void;
}

/** The message, and the button that clears the filters when some are set. */
export function EmptyMessage({ filters, onChange }: Props) {
  return (
    <>
      <p className="muted">{t("filter.none")}</p>
      {hasFilters(filters) && (
        <button type="button" className="btn btn-s" onClick={() => onChange(NO_FILTERS)}>
          {t("filter.clearAll")}
        </button>
      )}
    </>
  );
}

/** The message in place of the list. */
export function EmptyList({ filters, onChange }: Props) {
  return (
    <div className="card list-card list-empty">
      <EmptyMessage filters={filters} onChange={onChange} />
    </div>
  );
}
