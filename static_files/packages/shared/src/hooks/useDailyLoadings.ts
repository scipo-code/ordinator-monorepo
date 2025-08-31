import { useQuery } from "@tanstack/react-query";
import { fetchDailyLoadings } from "../api/daily_loadings.ts";

export const useDailyLoadings = (
  asset: string,
) => {
  return useQuery({
    queryKey: ["resources", asset],
    enabled: !!asset,
    queryFn: () => fetchDailyLoadings(asset),
    retry: 2,
    staleTime: 0,
  });
};
