import type { NaiveDateDto } from "@/types/dto/NaiveDateDto.ts";
import { apiConfig } from "./config.ts";

export async function fetchDays(): Promise<NaiveDateDto[]> {
  const res = await fetch(`${apiConfig.baseUrl}/api/v1/days`);

  if (!res.ok) {
    throw new Error(await res.text());
  }

  return (await res.json()) as NaiveDateDto[];
}
