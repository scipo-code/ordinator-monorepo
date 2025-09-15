import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef } from "react";
import { fetchSolution } from "../api/solution_status.ts";

export type queryKeys = string;

export function useVersionChangeDetector(
  url: string,
  keysToInvalidate: queryKeys[],
  intervalMs = 5000,
) {
  const queryClient = useQueryClient();
  const lastVersionRef = useRef<bigint | null>(null);

  const query = useQuery({
    queryKey: ["change-detector", url],
    queryFn: () => fetchSolution(url),
    enabled: !!url && url !== "",
    refetchInterval: intervalMs,
    refetchIntervalInBackground: true,
    notifyOnChangeProps: ["data", "error"],
  });

  useEffect(() => {
    if (query.data?.version) {
      const currentVersion = query.data.version;

      if (
        lastVersionRef.current !== null &&
        currentVersion !== lastVersionRef.current
      ) {
        console.log(
          `Version changed from ${lastVersionRef.current} to ${currentVersion}`,
        );
        keysToInvalidate.forEach((key) => {
          queryClient.invalidateQueries({ queryKey: [key] });
        });
      }

      lastVersionRef.current = currentVersion;
    }
  }, [query.data?.version, keysToInvalidate, queryClient]);

  return query;
}

export function useCurrentVersion(
  url: string,
  intervalMs = 1000,
) {
  const query = useQuery({
    queryKey: ["version", url],
    queryFn: () => fetchSolution(url),
    enabled: !!url && url !== "",
    refetchInterval: intervalMs,
    refetchIntervalInBackground: true,
  });

  return query;
}
