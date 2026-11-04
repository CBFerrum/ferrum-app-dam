// The library: assets waiting for review, approved assets whose rights end within 30 days, and the
// approved assets by collection. Each opens its record.

import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router";
import { api, docPath } from "@ferrum/client";
import { Badge, ErrorState, PageLoading, useShellTitle } from "@ferrum/kit";

export interface Row {
  id: string;
  [k: string]: unknown;
}

const rows = (entity: string, filters: unknown[], extra: Record<string, string | number> = {}) =>
  api.get<{ rows: Row[] }>(docPath(entity), { filters: JSON.stringify(filters), limit: 500, ...extra }).then((r) => r.rows);

/** Approved assets whose last day falls between `today` and `days` days on, soonest first. */
export function ending(list: Row[], today: string, days = 30): Row[] {
  const last = new Date(Date.parse(today) + days * 86_400_000).toISOString().slice(0, 10);
  return list
    .filter((a) => typeof a.usage_expires_on === "string" && a.usage_expires_on >= today && a.usage_expires_on <= last)
    .sort((a, b) => String(a.usage_expires_on).localeCompare(String(b.usage_expires_on)));
}

export function Home() {
  useShellTitle("Digital Assets");
  const q = useQuery({
    queryKey: ["dam", "library"],
    queryFn: async () => {
      const [review, approved] = await Promise.all([rows("MediaAsset", [["status", "eq", "In Review"]]), rows("MediaAsset", [["status", "eq", "Approved"]], { sort: "title asc" })]);
      return { review, approved };
    },
  });
  if (q.isPending) return <PageLoading />;
  if (q.isError) return <ErrorState error={q.error} onRetry={() => void q.refetch()} />;
  const soon = ending(q.data.approved, new Date().toISOString().slice(0, 10));
  return (
    <div className="np-page">
      <header className="row" style={{ justifyContent: "space-between" }}>
        <h2>The library</h2>
        <Link to="/e/MediaAsset/new">New asset</Link>
      </header>
      <div className="np-columns">
        <section className="card card-pad stack" aria-label="Waiting for review">
          <h3>Waiting for review ({q.data.review.length})</h3>
          <ul className="stack">
            {q.data.review.map((a) => (
              <li key={a.id}>
                <Link to={`/e/MediaAsset/${a.id}`}>{String(a.title)}</Link> {a.review_note ? <span className="faint">· {String(a.review_note)}</span> : null}
              </li>
            ))}
          </ul>
        </section>
        <section className="card card-pad stack" aria-label="Rights ending">
          <h3>Rights ending in 30 days ({soon.length})</h3>
          <ul className="stack">
            {soon.map((a) => (
              <li key={a.id}>
                <Link to={`/e/MediaAsset/${a.id}`}>{String(a.title)}</Link> · last day {String(a.usage_expires_on)}
              </li>
            ))}
          </ul>
        </section>
        <section className="card card-pad stack" aria-label="Approved">
          <h3>Approved ({q.data.approved.length})</h3>
          <ul className="stack">
            {q.data.approved.map((a) => (
              <li key={a.id}>
                <Link to={`/e/MediaAsset/${a.id}`}>{String(a.title)}</Link> <Badge tone="neutral">{String(a.kind ?? "")}</Badge> {a.license === "Editorial Only" ? <Badge tone="amber">Editorial only</Badge> : null}
              </li>
            ))}
          </ul>
        </section>
      </div>
    </div>
  );
}
