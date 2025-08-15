import { describe, expect, it } from "vitest";
import { fetchDays } from "../days.ts";

describe("fetchDays", () => {
  it("should fetch days successfully", async () => {
    const result = await fetchDays();

    expect(Array.isArray(result)).toBe(true);
    expect(result[0]).toBe("2025-01-13");
  });
});
