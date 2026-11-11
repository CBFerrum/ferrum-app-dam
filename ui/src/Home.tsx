// The library: assets waiting for review, approved assets whose rights end within 30 days, and the
// approved assets by collection, each with its thumbnail. Each opens to be seen whole.

import { useQuery } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { api, docPath, fetchBlob, formatValue, todayIso, t, isolate } from "@ferrum/client";
import { Badge, ButtonLink, HomeSkeleton, LoadFailed, PageHead, PageStack, Queue, QueueGrid, QueueItem, useShellTitle } from "@ferrum/kit";

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

/** The queues show this many rows each; the rest is "and N more", linking to the list. */
const LIMIT = 5;

/** A day as the person reads it ("20 Nov 2026"); empty when there is none. */
const day = (v: unknown) => formatValue({ type: "date" }, v);

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
  return src ? <img src={src} alt="" /> : <span aria-hidden="true" />;
}

export function Home() {
  useShellTitle(t("Digital Assets"));
  const q = useQuery({
    queryKey: ["dam", "library"],
    queryFn: async () => {
      const [review, approved] = await Promise.all([rows("MediaAsset", [["status", "eq", "In Review"]]), rows("MediaAsset", [["status", "eq", "Approved"]], { sort: "title asc" })]);
      return { review, approved };
    },
  });
  const create = (
    <ButtonLink to="/e/MediaAsset/new" variant="primary" icon="plus">
      {t("New asset")}
    </ButtonLink>
  );
  if (q.isPending) return (
    <PageStack>
      <PageHead title={t("The library")} />
      <HomeSkeleton queues={3} />
    </PageStack>
  );
  if (q.isError) return (
    <PageStack>
      <PageHead title={t("The library")} actions={create} />
      <LoadFailed error={q.error} onRetry={() => void q.refetch()} />
    </PageStack>
  );
  const { review, approved } = q.data;
  const soon = ending(approved, todayIso());
  const summary = [review.length && t("{n} waiting for review", { n: review.length }), soon.length && t("{n} rights ending soon", { n: soon.length })].filter(Boolean).join(" · ");
  return (
    <PageStack>
      <PageHead title={t("The library")} description={summary ? isolate(summary) : undefined} actions={create} />
      <QueueGrid>
        {soon.length > 0 && (
          <Queue title={t("Rights ending in 30 days")} count={soon.length} tone="attention" more={{ to: "/e/MediaAsset", count: soon.length - LIMIT }}>
            {soon.slice(0, LIMIT).map((a) => (
              <QueueItem key={a.id} to={`/view/${a.id}`} title={isolate(String(a.title))} meta={t("last day {usage_expires_on}", { usage_expires_on: day(a.usage_expires_on) })} lead={<Thumbnail asset={a} />} />
            ))}
          </Queue>
        )}
        <Queue title={t("Waiting for review")} count={review.length} more={{ to: "/e/MediaAsset", count: review.length - LIMIT }}>
          {review.slice(0, LIMIT).map((a) => (
            <QueueItem key={a.id} to={`/view/${a.id}`} title={isolate(String(a.title))} meta={a.review_note ? isolate(String(a.review_note)) : undefined} lead={<Thumbnail asset={a} />} />
          ))}
        </Queue>
        <Queue title={t("Approved")} count={approved.length} more={{ to: "/e/MediaAsset", count: approved.length - LIMIT }}>
          {approved.slice(0, LIMIT).map((a) => (
            <QueueItem
              key={a.id}
              to={`/view/${a.id}`}
              title={isolate(String(a.title))}
              meta={isolate(String(a.kind ?? ""))}
              lead={<Thumbnail asset={a} />}
              trailing={
                <>
                  {a.license === "Editorial Only" ? <Badge tone="amber">{t("Editorial only")}</Badge> : null}
                  {a.has_location ? <Badge tone="amber">{t("Records where it was taken")}</Badge> : null}
                </>
              }
            />
          ))}
        </Queue>
      </QueueGrid>
    </PageStack>
  );
}
