import type { NaiveDateDto } from "@/types/dto/NaiveDateDto.ts";
import { apiConfig } from "./config.ts";
import type { SupervisorMainTableDto } from "@/types/dto/SupervisorMainTableDto.ts";
import type { SupervisorAllAvailableTechnicians } from "@/types/dto/SupervisorAllAvailableTechnicians.ts";
import type { CreateTechnicianDto } from "@/types/dto/CreateTechnicianDto.ts";

export async function fetchMainTable(
  asset: string,
  supervisor_id: string,
  day?: NaiveDateDto,
): Promise<SupervisorMainTableDto> {
  const params = new URLSearchParams();

  if (day) {
    params.append("day", day);
  }

  const url =
    `${apiConfig.baseUrl}/api/v1/supervisor/supervisor_main_table/${asset}/${supervisor_id}?`;
  const res = await fetch(
    url + params.toString(),
  );

  if (!res.ok) {
    const body = await res.text();
    throw new Error(`(${res.status}) ${body}`);
  }

  return (await res.json()) as SupervisorMainTableDto;
}

export async function fetchTechnicianAvailability(
  asset: string,
  supervisor_id: string,
): Promise<SupervisorAllAvailableTechnicians> {
  const url =
    `${apiConfig.baseUrl}/api/v1/supervisor/technician_availability/${asset}/${supervisor_id}`;
  const res = await fetch(url);

  if (!res.ok) {
    const body = await res.text();
    throw new Error(`(${res.status}) ${body}`);
  }

  return (await res.json()) as SupervisorAllAvailableTechnicians;
}

export async function addTechnician(
  asset: string,
  supervisor_id: string,
  technician: CreateTechnicianDto,
): Promise<string> {
  const url =
    `${apiConfig.baseUrl}/api/v1/supervisor/add_technician/${
      encodeURIComponent(asset)
    }` +
    `/${encodeURIComponent(supervisor_id)}`;

  const reqOptions = {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(technician),
  };

  const res = await fetch(url, reqOptions);

  if (!res.ok) {
    throw new Error(await res.text());
  }

  return res.text();
}
