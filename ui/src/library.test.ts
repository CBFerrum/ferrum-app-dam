import { describe, expect, it } from "vitest";
import { ending, storedUrl } from "./Home";

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
