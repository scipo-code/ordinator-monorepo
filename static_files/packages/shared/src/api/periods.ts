import type { PeriodDto } from "../../../../crates/ordinator-contracts/bindings/PeriodDto.ts";

export default async function fetchPeriods(): Promise<PeriodDto[]> {
  const res = await fetch(
    "/api/v1/periods",
  );

  if (!res.ok) {
    const body = await res.text();
    throw new Error(`(${res.status}) ${body}`);
  }

  return (await res.json()) as PeriodDto[];
}
