import { useQuery } from "@tanstack/react-query";
import { fetchTechnicianAvailability } from "../api/daily.ts";

export const useTechnicianAvailability = (
  asset: string,
  dailyId: string,
) => {
  return useQuery({
    queryKey: ["technicianAvailability", asset, dailyId],
    enabled: !!asset && !!dailyId,
    queryFn: () => fetchTechnicianAvailability(asset, dailyId),
    retry: 2,
    staleTime: 60_000,
  });
};
