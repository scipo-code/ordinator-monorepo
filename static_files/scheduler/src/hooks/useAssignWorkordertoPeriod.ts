import { useMutation, useQueryClient } from "@tanstack/react-query";
import type { PeriodDto } from "../../../../crates/ordinator-contracts/bindings/PeriodDto.ts";
import { assignWorkordertoPeriod } from "../api/workorders.ts";

export function useAssignWorkorderToPeriod() {
  const qc = useQueryClient();

  return (
    useMutation({
      mutationFn: (params: {
        asset: string;
        workorder: string;
        period: PeriodDto;
      }) =>
        assignWorkordertoPeriod(
          params.asset,
          params.workorder,
          params.period,
        ),

      onSuccess: () => {
        qc.invalidateQueries({ queryKey: ["workOrders"] });
      },
    })
  );
}
