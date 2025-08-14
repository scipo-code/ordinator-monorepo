import { apiConfig } from "./config.ts";
export async function fetchResources(): Promise<string[]> {
  const res = await fetch(`${apiConfig.baseUrl}/api/v1/resources`);

  if (!res.ok) {
    throw new Error(await res.text());
  }

  return (await res.json());
}
