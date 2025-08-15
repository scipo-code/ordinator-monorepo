import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { fetchSolution } from "../api/solution_status.ts";
export type queryKeys = string;

export function useVersionChangeDetector(
  url: string,
  keysToInvalidate: queryKeys[],
  intervalMs = 5000,
) {
  const queryClient = useQueryClient();
  const [lastVersion, setLastVersion] = useState<null | bigint>(null);

  const query = useQuery({
    queryKey: ["change-detector", url],
    queryFn: () => fetchSolution(url),
    enabled: !!url && url !== "",
    refetchInterval: intervalMs,
    refetchIntervalInBackground: true,
  });

  useEffect(() => {
    if (query.data?.version) {
      const currentVersion = query.data.version;

      if (lastVersion !== null && currentVersion !== lastVersion) {
        keysToInvalidate.forEach((key) => {
          queryClient.invalidateQueries({ queryKey: [key] });
        });
      }
      setLastVersion(currentVersion);
    }
  }, [query.data?.version, lastVersion, keysToInvalidate, queryClient]);

  return query;
}
