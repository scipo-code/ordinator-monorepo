import { apiConfig } from "./config.ts";
import { CreateTechnicianDto } from "../types/CreateTechnicianDto.ts";
import { NaiveDateDto } from "../types/NaiveDateDto.ts";
import { SupervisorAllAvailableTechnicians } from "../types/SupervisorAllAvailableTechnicians.ts";
import { SupervisorMainTableDto } from "../types/SupervisorMainTableDto.ts";

export async function fetchMainTable(
  asset: string,
  daily_id: string,
  day?: NaiveDateDto,
): Promise<SupervisorMainTableDto> {
  const params = new URLSearchParams();

  if (day) {
    params.append("day", day);
  }

  const url =
    `${apiConfig.baseUrl}/api/v1/daily/daily_main_table/${asset}/${daily_id}?`;
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
  daily_id: string,
): Promise<SupervisorAllAvailableTechnicians> {
  const url =
    `${apiConfig.baseUrl}/api/v1/daily/technician_availability/${asset}/${daily_id}`;
  const res = await fetch(url);

  if (!res.ok) {
    const body = await res.text();
    throw new Error(`(${res.status}) ${body}`);
  }

  return (await res.json()) as SupervisorAllAvailableTechnicians;
}

export async function addTechnician(
  asset: string,
  daily_id: string,
  technician: CreateTechnicianDto,
): Promise<string> {
  const url =
    `${apiConfig.baseUrl}/api/v1/daily/add_technician/${
      encodeURIComponent(asset)
    }` +
    `/${encodeURIComponent(daily_id)}`;

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
