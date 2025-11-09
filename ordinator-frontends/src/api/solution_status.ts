import type { SolutionVersionDto } from "@/types/dto/SolutionVersionDto.ts";
import { apiConfig } from "./config.ts";

export async function fetchSolution(
  endpoint: string,
): Promise<SolutionVersionDto> {
  const res = await fetch(`${apiConfig.baseUrl}/${endpoint}`);

  if (!res.ok) {
    throw new Error(await res.text());
  }

  return (await res.json()) as SolutionVersionDto;
}
