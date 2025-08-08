import { NaiveDateDto } from "../types/NaiveDateDto.ts";
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
