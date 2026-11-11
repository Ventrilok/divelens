import { DASH } from "../format";
import { t } from "../i18n";

/** Rating as stars; a dash when unrated. */
export function Stars({ rating }: { rating: number | null }) {
  if (!rating) {
    return (
      <span className="muted" aria-label={t("rating.none")}>
        {DASH}
      </span>
    );
  }
  return (
    <span className="stars" aria-label={t("rating.stars", { n: rating })}>
      {"★".repeat(rating)}
      <span className="stars-off">{"★".repeat(5 - rating)}</span>
    </span>
  );
}
