import { SingleRowDto } from "./SingleRowDto.ts";

export type SchedulingData = SingleRowDto & { action: string | null };
