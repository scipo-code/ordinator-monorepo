import { apiConfig } from "./config.ts";
import { NaiveDateDto } from "../types/NaiveDateDto.ts";

export async function fetchDays(): Promise<NaiveDateDto[]> {
  const res = await fetch(`${apiConfig.baseUrl}/api/v1/days`);

  if (!res.ok) {
    throw new Error(await res.text());
  }

  return (await res.json()) as NaiveDateDto[];
}
