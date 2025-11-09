import { useQuery } from "@tanstack/react-query";
import { fetchMainTable } from "../api/supervisor.ts";
import type { NaiveDateDto } from "@/types/dto/NaiveDateDto.ts";

export const useSupervisorMainTable = (
  asset: string,
  supervisorId: string,
  day?: NaiveDateDto,
) => {
  return useQuery({
    queryKey: ["supervisorMainTable", asset, supervisorId, day],
    enabled: !!asset && !!supervisorId,
    queryFn: () => fetchMainTable(asset, supervisorId, day),
    retry: 2,
    staleTime: 60_000,
  });
};
