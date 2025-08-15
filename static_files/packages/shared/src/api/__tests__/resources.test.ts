import { describe, expect, it } from "vitest";
import { fetchResources } from "../resources.ts";

describe("fetchResources", () => {
  it("should fetch resources successfully", async () => {
    const result = await fetchResources();

    expect(Array.isArray(result)).toBe(true);
    expect(result[0]).toBe("MTN-MECH");
  });
});
