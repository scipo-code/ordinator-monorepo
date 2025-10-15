import { apiConfig } from "./config.ts";
type SystemClock = string;

export async function fetchSystemClock(): Promise<SystemClock> {
  const res = await fetch(`${apiConfig.baseUrl}/api/v1/system_clock`);

  if (!res.ok) {
    throw new Error(await res.text());
  }

  return (await res.json()) as SystemClock;
}
