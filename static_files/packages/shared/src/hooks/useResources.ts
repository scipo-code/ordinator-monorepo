import { useQuery } from "@tanstack/react-query";
import { fetchResources } from "../api/resources.ts";

export const useResources = () => {
  return useQuery({
    queryKey: ["resources"],
    queryFn: fetchResources,
    staleTime: Infinity,
  });
};
