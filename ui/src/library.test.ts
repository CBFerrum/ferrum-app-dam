import { describe, expect, it } from "vitest";
import { ending, storedUrl } from "./Home";
import { player, previewUrl } from "./View";

describe("the library", () => {
  it("lists rights ending within the window, soonest first", () => {
    const l = ending(
      [
        { id: "later", usage_expires_on: "2027-02-01" },
        { id: "b", usage_expires_on: "2026-11-20" },
        { id: "a", usage_expires_on: "2026-11-10" },
        { id: "gone", usage_expires_on: "2026-10-01" },
        { id: "forever" },
      ],
      "2026-11-04",
    );
    expect(l.map((x) => x.id)).toEqual(["a", "b"]);
  });
});

describe("thumbnails", () => {
  it("are read only from the platform's stored files", () => {
    expect(storedUrl("/files/0190f1a2-3b4c-7d5e-8f60-123456789abc")).toBe("/files/0190f1a2-3b4c-7d5e-8f60-123456789abc");
    expect(storedUrl("https://elsewhere.test/x.png")).toBeNull();
    expect(storedUrl("/files/../api/x")).toBeNull();
    expect(storedUrl(null)).toBeNull();
  });
});

describe("an asset seen whole", () => {
  it("is played, read or shown only for the kinds the preview route answers", () => {
    expect(player("video/mp4")).toBe("video");
    expect(player("Audio/MPEG; charset=binary")).toBe("audio");
    expect(player("application/pdf")).toBe("pdf");
    expect(player("image/webp")).toBe("image");
    for (const t of ["text/html", "image/svg+xml", "application/zip", "", null]) expect(player(t)).toBeNull();
    expect(previewUrl("a/b")).toBe("/api/app/dam/preview/a%2Fb");
  });
});
