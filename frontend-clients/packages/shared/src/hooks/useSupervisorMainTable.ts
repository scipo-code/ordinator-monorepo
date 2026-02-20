import { useQuery } from "@tanstack/react-query";
import { fetchMainTable } from "../api/daily.ts";
import { NaiveDateDto } from "../types/NaiveDateDto.ts";

export const useSupervisorMainTable = (
  asset: string,
  dailyId: string,
  day?: NaiveDateDto,
) => {
  return useQuery({
    queryKey: ["dailyMainTable", asset, dailyId, day],
    enabled: !!asset && !!dailyId,
    queryFn: () => fetchMainTable(asset, dailyId, day),
    retry: 2,
    staleTime: 60_000,
  });
};
