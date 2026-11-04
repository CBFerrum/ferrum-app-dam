import { describe, expect, it } from "vitest";
import { ending } from "./Home";

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
