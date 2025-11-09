import type { PeriodDto } from "@/types/dto/PeriodDto.ts";
import { apiConfig } from "./config.ts";

export async function fetchPeriods(): Promise<PeriodDto[]> {
  const res = await fetch(
    `${apiConfig.baseUrl}/api/v1/periods`,
  );

  if (!res.ok) {
    const body = await res.text();
    throw new Error(`(${res.status}) ${body}`);
  }

  return (await res.json()) as PeriodDto[];
}
