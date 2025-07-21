import { SchedulingData } from "./ColDef";
import { PeriodDto } from "../../../../../../crates/ordinator-contracts/bindings/PeriodDto.ts";
import { useMutation, useQueryClient } from "@tanstack/react-query";


export const fetchWorkOrders = async (
  asset: string,
): Promise<SchedulingData[]> => {
  const res = await fetch(
    `/api/v1/scheduler/work_orders_with_scheduling/${asset}`,
  );

  if (!res.ok) {
    // propagate a rejected promise so React Query sets "error"
    const body = await res.text();
    throw new Error(`(${res.status}) ${body}`);
  }

  // The Rust DTO is `Vec<SingleRowDto>` => serialises to a bare JSON array
  return (await res.json()) as SchedulingData[];
};



async function assignWorkordertoPeriod(
  asset: string,
  period: PeriodDto,
  workorder: string,
): Promise<string> {
  const url = `/api/v1/scheduler/${encodeURIComponent(asset)}` +
                       `/assign_work_order_to_period/${encodeURIComponent(workorder.toString())}` +
                       `/${encodeURIComponent(period)}`;

  const res = await fetch(url, { method: "POST" });

  if (!res.ok) {
    throw new Error(await res.text());
  }

  return res.text();
}

export function useAssignWorkorderToPeriod() {
  const qc = useQueryClient();

  return(
    useMutation({
      mutationFn: (params: {
        asset: string;
        period: PeriodDto;
        workorder: string;
      }) => assignWorkordertoPeriod(
          params.asset,
          params.period,
          params.workorder,
        ),

        onSuccess: () => {
          qc.invalidateQueries({ queryKey: ["workOrders"] });
        },
      
    })
    )
}


