// One asset seen whole: its video or sound played, its PDF read, its picture shown, from its file as
// it stands, read with the person's own rights (`/api/app/dam/preview/{asset}`). Its record, with
// what may be done with it, is a link away.

import { useQuery } from "@tanstack/react-query";
import { Link, useParams } from "react-router";
import { api, docPath, t as tr } from "@ferrum/client";
import { Badge, ErrorState, PageLoading, useShellTitle } from "@ferrum/kit";
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
  useShellTitle(q.data ? String(q.data.title) : "Asset");
  if (q.isPending) return <PageLoading />;
  if (q.isError) return <ErrorState error={q.error} onRetry={() => void q.refetch()} />;
  const a = q.data;
  const title = String(a.title);
  const how = player(a.content_type);
  const src = previewUrl(a.id);
  return (
    <div className="np-page stack">
      <header className="row" style={{ justifyContent: "space-between" }}>
        <h2>
          {title} <Badge tone="neutral">{String(a.status ?? "")}</Badge>
        </h2>
        <Link to={`/e/MediaAsset/${a.id}`}>{tr("Open the record")}</Link>
      </header>
      <section className="card card-pad stack" aria-label={tr("The file")}>
        {how === "video" ? <video controls preload="metadata" src={src} style={{ maxWidth: "100%", maxHeight: "75vh" }} /> : null}
        {how === "audio" ? <audio controls preload="metadata" src={src} /> : null}
        {how === "pdf" ? <iframe title={title} src={src} style={{ width: "100%", height: "75vh", border: 0 }} /> : null}
        {how === "image" ? <img src={src} alt={title} style={{ maxWidth: "100%", maxHeight: "75vh", objectFit: "contain" }} /> : null}
        {how ? null : <p className="faint">{tr("A page does not show a file of type")} {String(a.content_type || "unknown")} {tr("by itself: open the record to download it.")}</p>}
      </section>
      <p className="faint">
        {[a.kind, a.license, a.usage_expires_on ? `last day ${String(a.usage_expires_on)}` : null, a.width && a.height ? `${String(a.width)} × ${String(a.height)}` : null]
          .filter(Boolean)
          .join(" · ")}
      </p>
    </div>
  );
}
