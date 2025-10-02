import { describe, expect, it } from "vitest";
import {
  addTechnician,
  fetchMainTable,
  fetchTechnicianAvailability,
} from "../supervisor.ts";
import { CreateTechnicianDto } from "../../types/CreateTechnicianDto.ts";

describe("supervisor API", () => {
  describe("fetchMainTable", () => {
    it("should fetch main table without day parameter", async () => {
      // ISSUE #000 supervisor_id = main should be fixed!
      const result = await fetchMainTable("Test", "main");

      expect(Object.keys(result.days)[0]).toBe("2025-01-13");
      expect(Object.keys(result.days).length).toBe(14);
    });

    it("should fetch main table with day parameter", async () => {
      const day = "2025-01-14";
      const result = await fetchMainTable("Test", "main", day);

      expect(Object.keys(result.days)[0]).toBe(day);
    });

    it("should throw error with status when response is not ok", async () => {
      await expect(fetchMainTable("pump1", "sup123")).rejects.toThrow(
        /^\(500\)/,
      );
    });
  });

  describe("fetchTechnicianAvailability", () => {
    it("should fetch technician availability successfully", async () => {
      // ISSUE #000 supervisor_id = main should be fixed!
      const result = await fetchTechnicianAvailability("Test", "main");

      expect(
        result.all_technicians.filter((tech) => tech.id === "TEST_OP-001-01")
          .length,
      ).toBe(1);
    });

    it("should throw error with status when response is not ok", async () => {
      const result = fetchTechnicianAvailability("pump1", "sup123");

      await expect(result).rejects.toThrow(
        /^\(500\)/,
      );
    });
  });

  describe("addTechnician", () => {
    it("should add technician successfully", async () => {
      const technician: CreateTechnicianDto = {
        id: "John-Doe",
        resources_string: ["MTN-ELEC", "MTN-ROUS"],
        start: "2025-01-13T07:00:00Z",
        finish: "2025-01-15T07:00:00Z",
      };

      // ISSUE #000 supervisor_id = main should be fixed!
      const result = await addTechnician("Test", "sup123", technician);

      expect(result).toBe(
        `"Technician ${technician.id} successfully created and started"`,
      );
    });

    it("should not add technician because start date is later than finish date", async () => {
      const technician: CreateTechnicianDto = {
        id: "John-Doe",
        resources_string: ["MTN-ELEC", "MTN-ROUS"],
        start: "2025-01-17T07:00:00Z",
        finish: "2025-01-15T07:00:00Z",
      };

      // ISSUE #000 supervisor_id = main should be fixed!
      const result = addTechnician("Test", "sup123", technician);

      await expect(result).rejects.toThrow(
        '{"error":"Start date later than finish date."}',
      );
    });

    // it("should throw error because technician already fails", async () => {
    //   const technician: CreateTechnicianDto = {
    //     id: "John-Doe",
    //     resources_string: ["MTN-ELEC", "MTN-ROUS"],
    //     start: "2025-01-13T07:00:00Z",
    //     finish: "2025-01-15T07:00:00Z",
    //   };

    //   // ISSUE #000 supervisor_id = main should be fixed!
    //   const result = addTechnician("Test", "sup123", technician);

    //   await expect(result).rejects.toThrow(
    //     `""Technician ${technician.id} already exists and is running""`,
    //   );
    // });
  });
});
