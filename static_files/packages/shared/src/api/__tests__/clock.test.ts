import { describe, expect, it } from "vitest";
import { fetchSystemClock } from "../clock.ts";

describe("fetchSystemClock", () => {
  it("should fetch system clock successfully", async () => {
    const expectedClock = "2025-01-01T07:00:00+00:00";
    const result = await fetchSystemClock();

    expect(typeof result).toBe("string");
    expect(result).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}/);
    expect(result).toBe(expectedClock);
  });
});
