import { fetchSystemClock } from "@/api/clock";
import { useQuery } from "@tanstack/react-query";

export const useSystemClock = () => {
  return useQuery({
    queryKey: ["systemclock"],
    queryFn: fetchSystemClock,
    staleTime: Infinity,
  });
};
