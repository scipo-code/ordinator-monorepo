import type { SchedulingData } from "../pages/ScheduleView.tsx";

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
