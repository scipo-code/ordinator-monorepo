import { useQuery } from "@tanstack/react-query";
import { fetchSystemClock } from "../api";

export const useSystemClock = () => {
  return useQuery({
    queryKey: ["systemclock"],
    queryFn: fetchSystemClock,
    staleTime: Infinity,
  });
};