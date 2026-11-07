// The library: assets waiting for review, approved assets whose rights end within 30 days, and the
// approved assets by collection, each with its thumbnail. Each opens to be seen whole.

import { useQuery } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { Link } from "react-router";
import { api, docPath, fetchBlob, todayIso, t } from "@ferrum/client";
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

/** A stored file's address (`/files/{id}`), or nothing. */
export function storedUrl(v: unknown): string | null {
  return typeof v === "string" && /^\/files\/[0-9a-f-]{36}$/i.test(v) ? v : null;
}

/** An asset's thumbnail, read with the person's own rights (it is seen by whoever sees the asset). */
function Thumbnail({ asset }: { asset: Row }) {
  const url = storedUrl(asset.thumbnail);
  const blob = useQuery({ queryKey: ["dam", "thumbnail", url], enabled: !!url, staleTime: Infinity, queryFn: ({ signal }) => fetchBlob(url!, signal) });
  const [src, setSrc] = useState<string>();
  useEffect(() => {
    if (!blob.data) return;
    const u = URL.createObjectURL(blob.data);
    setSrc(u);
    return () => URL.revokeObjectURL(u);
  }, [blob.data]);
  if (!url) return null;
  return src ? <img src={src} alt="" width={48} height={48} style={{ objectFit: "cover", borderRadius: 4, verticalAlign: "middle" }} /> : <span style={{ display: "inline-block", width: 48 }} />;
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
  const soon = ending(q.data.approved, todayIso());
  return (
    <div className="np-page">
      <header className="row" style={{ justifyContent: "space-between" }}>
        <h2>{t("The library")}</h2>
        <Link to="/e/MediaAsset/new">{t("New asset")}</Link>
      </header>
      <div className="np-columns">
        <section className="card card-pad stack" aria-label={t("Waiting for review")}>
          <h3>{t("Waiting for review ({length})", { length: q.data.review.length })}</h3>
          <ul className="stack">
            {q.data.review.map((a) => (
              <li key={a.id} className="row" style={{ gap: 8 }}>
                <Thumbnail asset={a} />
                <span>
                  <Link to={`/view/${a.id}`}>{String(a.title)}</Link> {a.review_note ? <span className="faint">· {String(a.review_note)}</span> : null}
                </span>
              </li>
            ))}
          </ul>
        </section>
        <section className="card card-pad stack" aria-label={t("Rights ending")}>
          <h3>{t("Rights ending in 30 days ({length})", { length: soon.length })}</h3>
          <ul className="stack">
            {soon.map((a) => (
              <li key={a.id}>
                <Link to={`/view/${a.id}`}>{String(a.title)}</Link> {t("· last day")} {String(a.usage_expires_on)}
              </li>
            ))}
          </ul>
        </section>
        <section className="card card-pad stack" aria-label={t("Approved")}>
          <h3>{t("Approved ({length})", { length: q.data.approved.length })}</h3>
          <ul className="stack">
            {q.data.approved.map((a) => (
              <li key={a.id} className="row" style={{ gap: 8 }}>
                <Thumbnail asset={a} />
                <span>
                  <Link to={`/view/${a.id}`}>{String(a.title)}</Link> <Badge tone="neutral">{String(a.kind ?? "")}</Badge>{" "}
                  {a.license === "Editorial Only" ? <Badge tone="amber">{t("Editorial only")}</Badge> : null}
                  {a.has_location ? <Badge tone="amber">{t("Records where it was taken")}</Badge> : null}
                </span>
              </li>
            ))}
          </ul>
        </section>
      </div>
    </div>
  );
}
