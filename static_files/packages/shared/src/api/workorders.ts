import { PeriodDto } from "../types/PeriodDto.ts";
import type { WorkOrderInfoWithSchedulingDto } from "../types/WorkOrderInfoWithSchedulingDto.ts";
import type { SchedulingData } from "../types/SchedulingData.ts";

export async function fetchWorkOrders(
  asset: string,
  periods?: string[],
): Promise<SchedulingData[]> {
  const params = new URLSearchParams();

  if (periods) {
    periods.forEach((p) => params.append("periods", p));
  }

  const url = `/api/v1/scheduler/work_orders_with_scheduling/${asset}?`;
  const res = await fetch(
    url + params.toString(),
  );

  if (!res.ok) {
    // propagate a rejected promise so React Query sets "error"
    const body = await res.text();
    throw new Error(`(${res.status}) ${body}`);
  }

  // The Rust DTO is `Vec<SingleRowDto>` => serialises to a bare JSON array
  return (await res.json()) as SchedulingData[];
}

export async function assignWorkordertoPeriod(
  asset: string,
  workorder: string,
  period: PeriodDto,
): Promise<string> {
  const url = `/api/v1/scheduler/${encodeURIComponent(asset)}` +
    `/assign_work_order_to_period/${encodeURIComponent(workorder)}` +
    `/${encodeURIComponent(period)}`;

  const res = await fetch(url, { method: "POST" });

  if (!res.ok) {
    throw new Error(await res.text());
  }

  return res.text();
}

export async function fetchWorkorderInfo(
  asset: string,
  work_order_number: string,
): Promise<WorkOrderInfoWithSchedulingDto> {
  const res = await fetch(
    `/api/v1/scheduler/work_orders_with_scheduling/${asset}/${work_order_number}`,
  );

  if (!res.ok) {
    throw new Error(await res.text());
  }

  return (await res.json()) as WorkOrderInfoWithSchedulingDto;
}
