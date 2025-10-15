import { describe, expect, it } from "vitest";
import { fetchPeriods } from "../periods.ts";

describe("fetchPeriods", () => {
  it("should fetch periods successfully", async () => {
    const result = await fetchPeriods();

    expect(Array.isArray(result)).toBe(true);
    expect(result[0]).toBe("2025-W3-4");
  });
});
