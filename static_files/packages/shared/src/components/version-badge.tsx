import { useCurrentVersion } from "../hooks";

const StagnationStatus: Record<string, {background: string}> = {
   unstable: { background: 'bg-red-800' },
   converging: { background: 'bg-yellow-800' },
   stable: { background: 'bg-green-800'  },
   none: { background: 'bg-gray-100'},
}


export function StagnationDot({asset}: {asset: string}) {

  const {data: version } = useCurrentVersion(
    asset ? `api/v1/solution_status/${encodeURIComponent(asset)}/tactical`: "",
    1000
  );

  const getStatus = (stagnations?: bigint): string => {
    if (!stagnations) return 'none';
    if (stagnations < 10n ) return 'unstable';
    if (stagnations < 50n ) return 'converging';
    return 'stable';
  }
  

  const statusKey = getStatus(version?.stagnation_iterations);
  const status = StagnationStatus[statusKey]


  
  return (
    <div className={`rounded-full h-[8px] w-[8px] inline-block mr-2 animate-pulse ${status.background}`}></div>
  )
}
