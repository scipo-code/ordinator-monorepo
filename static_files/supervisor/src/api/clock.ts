type SystemClock = string;

export default async function fetchSystemClock(): Promise<SystemClock> {
  const res = await fetch("/api/v1/system_clock");

  if (!res.ok) {
    throw new Error(await res.text());
  }

  return (await res.json()) as SystemClock;
}
