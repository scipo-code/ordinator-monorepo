import type { DailyLoadingDto } from "@/types/dto/DailyLoadingDto.ts";
import { apiConfig } from "./config.ts";

export async function fetchDailyLoadings(
  asset: string,
): Promise<DailyLoadingDto> {
  const res = await fetch(
    `${apiConfig.baseUrl}/api/v1/tactical/daily_loadings/${asset}`,
  );

  if (!res.ok) {
    throw new Error(await res.text());
  }

  return (await res.json()) as DailyLoadingDto;
}
