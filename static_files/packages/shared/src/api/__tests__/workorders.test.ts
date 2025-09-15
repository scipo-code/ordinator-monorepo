import { describe, expect, it } from "vitest";
import {
  assignWorkOrderToPeriod,
  fetchWorkorderInfo,
  fetchWorkOrders,
} from "../workorders.ts";
import { PeriodDto } from "../../types/PeriodDto.ts";

describe("workorders API", () => {
  describe("fetchWorkOrders", () => {
    it("should fetch work orders without periods", async () => {
      const result = await fetchWorkOrders("Test");

      expect(result.length > 0).toBe(true);
      expect(result[0]).toHaveProperty("work_order_number");
    });

    it("should fetch work orders with periods", async () => {
      const result = await fetchWorkOrders("Test", ["2025-W3-4"]);

      expect(result.length > 0).toBe(true);
      expect(result[0]).toHaveProperty("work_order_number");
    });

    it("should not return data with wrong period", async () => {
      const result = await fetchWorkOrders("Test", ["2025-W1-4"]);

      expect(result.length).toBe(0);
    });

    it("should throw error with status when response is not ok", async () => {
      await expect(fetchWorkOrders("pump1")).rejects.toThrow(
        /^\(500\)/,
      );
    });
  });

  describe("assignWorkOrderToPeriod", () => {
    it("should assign work order to period successfully", async () => {
      const period: PeriodDto = "2025-W3-4";

      const result = await assignWorkOrderToPeriod(
        "Test",
        "1111990000",
        period,
      );

      expect(result).toBeDefined();
      expect(typeof result).toBe("string");
    });

    it("should assign work order to period successfully", async () => {
      const period: PeriodDto = "2025-W3-4";

      const result = await assignWorkOrderToPeriod(
        "Test",
        "1111990000",
        period,
      );

      expect(result).toBeDefined();
      expect(typeof result).toBe("string");
    });

    it("should error due to wrong workorder", async () => {
      const period: PeriodDto = "2025-W3-4";

      const result = assignWorkOrderToPeriod("Test", "11119900009", period);

      await expect(result).rejects.toThrow(/Could not assign/);
    });
  });

  describe("fetchWorkorderInfo", () => {
    it("should fetch work order info successfully", async () => {
      const result = await fetchWorkorderInfo("Test", "1111990000");

      expect(result.asset).toBe("TEST");
    });
  });
});
