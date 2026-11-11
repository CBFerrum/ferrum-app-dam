// One asset seen whole: its video or sound played, its PDF read, its picture shown, from its file as
// it stands, read with the person's own rights (`/api/app/dam/preview/{asset}`). Its record, with
// what may be done with it, is a link away.

import { useQuery } from "@tanstack/react-query";
import { useParams } from "react-router";
import { api, docPath, t as tr, isolate } from "@ferrum/client";
import { ButtonLink, LoadFailed, PageHead, ReaderSkeleton, StatusBadge, placed } from "@ferrum/kit";
import type { Row } from "./Home";

const shown: Record<string, "video" | "audio" | "pdf" | "image"> = {
  "application/pdf": "pdf",
  "video/mp4": "video",
  "video/webm": "video",
  "video/ogg": "video",
  "video/quicktime": "video",
  "audio/mpeg": "audio",
  "audio/mp4": "audio",
  "audio/ogg": "audio",
  "audio/wav": "audio",
  "audio/webm": "audio",
  "audio/flac": "audio",
  "audio/aac": "audio",
  "image/png": "image",
  "image/jpeg": "image",
  "image/gif": "image",
  "image/webp": "image",
  "image/avif": "image",
};

/** How a page shows a file of this type by itself (the kinds the preview route answers), or nothing. */
export function player(contentType: unknown): "video" | "audio" | "pdf" | "image" | null {
  const t = typeof contentType === "string" ? contentType.split(";")[0].trim().toLowerCase() : "";
  return shown[t] ?? null;
}

export const previewUrl = (asset: string) => `/api/app/dam/preview/${encodeURIComponent(asset)}`;

export function View() {
  const { id = "" } = useParams();
  const q = useQuery({ queryKey: ["dam", "asset", id], queryFn: () => api.get<Row>(docPath("MediaAsset", id)) });
  const back = { to: "/", label: tr("The library") };
  if (q.isPending) return (
    <>
      <PageHead title={tr("Asset")} back={back} />
      <ReaderSkeleton />
    </>
  );
  if (q.isError) return (
    <>
      <PageHead title={tr("Asset")} back={back} />
      <LoadFailed error={q.error} onRetry={() => void q.refetch()} />
    </>
  );
  const a = q.data;
  const title = String(a.title);
  const how = player(a.content_type);
  const src = previewUrl(a.id);
  const facts = [a.kind, a.license, a.usage_expires_on ? tr("last day {usage_expires_on}", { usage_expires_on: String(a.usage_expires_on) }) : null, a.width && a.height ? `${String(a.width)} × ${String(a.height)}` : null].filter(Boolean).join(" · ");
  return (
    <>
      <PageHead
        title={isolate(title)}
        back={back}
        description={facts ? isolate(facts) : undefined}
        actions={
          <>
            <StatusBadge value={a.status} />
            <ButtonLink to={`/e/MediaAsset/${a.id}`}>{tr("Open the record")}</ButtonLink>
          </>
        }
      />
      <section className="card card-pad" aria-label={tr("The file")}>
        {how === "video" ? <video controls preload="metadata" src={src} className="mx-auto max-h-[75vh] max-w-full" /> : null}
        {how === "audio" ? <audio controls preload="metadata" src={src} className="w-full" /> : null}
        {how === "pdf" ? <iframe title={title} src={src} className="h-[75vh] w-full border-0" /> : null}
        {how === "image" ? <img src={src} alt={title} className="mx-auto max-h-[75vh] max-w-full object-contain" /> : null}
        {how ? null : <p className="muted">{placed(tr("A page does not show a file of type {p1} by itself: open the record to download it."), { p1: isolate(String(a.content_type || "unknown")) })}</p>}
      </section>
    </>
  );
}
