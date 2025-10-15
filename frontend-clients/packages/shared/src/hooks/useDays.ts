import { useQuery } from "@tanstack/react-query";
import { fetchDays } from "../api/days.ts";

export const useDays = () => {
  return useQuery({
    queryKey: ["days"],
    queryFn: fetchDays,
    staleTime: Infinity,
  });
};
