import { useMutation, useQueryClient } from "@tanstack/react-query";
import { PeriodDto } from "@scipo-code/shared";
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
