import { useQuery } from "@tanstack/react-query";
import { fetchTechnicianAvailability } from "../api/supervisor.ts";

export const useTechnicianAvailability = (
  asset: string,
  supervisorId: string,
) => {
  return useQuery({
    queryKey: ["technicianAvailability", asset, supervisorId],
    enabled: !!asset && !!supervisorId,
    queryFn: () => fetchTechnicianAvailability(asset, supervisorId),
    retry: 2,
    staleTime: 60_000,
  });
};
