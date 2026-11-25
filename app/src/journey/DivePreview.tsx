// The preview by dive of step 3 (spec F5), the same on either path: one row
// per dive with its photos, its cameras and what its photos get per photo,
// then the block about the photos outside the dives. The texts come from
// clocks.ts.

import { formatInt, t, tn } from "../i18n";
import { DASH, formatRowSpan } from "../ui/format";
import type { OutsideBlock, PreviewRow } from "./clocks";

export function DivePreview({ rows, outside }: { rows: PreviewRow[]; outside: OutsideBlock | null }) {
  return (
    <section className="card coll-table" aria-labelledby="h-dives">
      <h2 id="h-dives">{t("coll.byDive")}</h2>
      <div className="coll-table-scroll">
        <table className="tbl">
          <thead>
            <tr>
              <th className="num">{t("col.number")}</th>
              <th>{t("col.site")}</th>
              <th>{t("coll.col.dive")}</th>
              <th className="num">{t("coll.col.photos")}</th>
              <th>{t("coll.col.cameras")}</th>
              <th>{t("coll.col.perPhoto")}</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.dive.id} className={r.warn ? "row-warn" : ""}>
                <td className="num">{r.dive.number ?? DASH}</td>
                <td className="site-name">{r.dive.site.name ?? DASH}</td>
                <td>{formatRowSpan(r.dive.start, r.dive.end)}</td>
                <td className={`num strong ${r.warn ? "warn" : ""}`}>{r.photos !== null ? formatInt(r.photos) : "…"}</td>
                <td className="muted">{r.cameras > 0 ? tn("coll.cameras", r.cameras) : DASH}</td>
                <td className={r.warn ? "warn" : ""}>{r.perPhoto}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {outside && (
        <div className="clocks-outside">
          <span className="clocks-outside-title">{outside.title}</span>
          {outside.lines.map((line, i) => (
            <span key={i} className="muted small">
              {line}
            </span>
          ))}
          {outside.cameras && <span className="muted small">{outside.cameras}</span>}
          {outside.how && <span className="small clocks-outside-how">{outside.how}</span>}
        </div>
      )}
    </section>
  );
}
