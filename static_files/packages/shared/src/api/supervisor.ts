import { NaiveDateDto } from "../types/NaiveDateDto.ts";
import { SupervisorAllAvailableTechnicians } from "../types/SupervisorAllAvailableTechnicians.ts";
import { SupervisorMainTableDto } from "../types/SupervisorMainTableDto.ts";

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
    `/api/v1/supervisor/supervisor_main_table/${asset}/${supervisor_id}?`;
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
    `/api/v1/supervisor/technician_availability/${asset}/${supervisor_id}`;
  const res = await fetch(url);

  if (!res.ok) {
    const body = await res.text();
    throw new Error(`(${res.status}) ${body}`);
  }

  return (await res.json()) as SupervisorAllAvailableTechnicians;
}
