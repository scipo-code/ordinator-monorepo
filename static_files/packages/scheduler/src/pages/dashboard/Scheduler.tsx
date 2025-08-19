import React from 'react';
import { AgGridReact } from 'ag-grid-react';
import { useParams } from 'react-router-dom';
import { useQuery } from "@tanstack/react-query";
import { useTableColDefs } from './scheduler/ColDef';
import { parseISO, format } from 'date-fns';
import { fetchWorkOrders, useSystemClock } from "@scipo-code/shared";
import { useVersionChangeDetector } from '@scipo-code/shared';
import { StagnationDot } from '@scipo-code/shared';


// import { agGridThemeLight } from "./theme";
// https://www.youtube.com/watch?v=TrKlKF5au6c&ab_channel=m6io
/** snake_case → "Title Case" */

const Scheduler: React.FC = () => {
  const { asset } = useParams<{ asset: string }>();
  const columns = useTableColDefs();
    
  /* Fetch with React Query */
  const {
    data: workOrders = [],          // default ↠ empty array
    isLoading,
    isError,
    error,
  } = useQuery({
    queryKey: ["workOrders", asset],
    // don’t run until we have a real asset
    enabled: !!asset,
    queryFn: () => fetchWorkOrders(asset!),
    retry: 2,                       // exponential-backoff retries
    staleTime: 60_000,              // cache 1 min
  });


  const query = useVersionChangeDetector(
    asset ? `api/v1/solution_status/${encodeURIComponent(asset)}/tactical`: "",
    ["workOrders"],
    1000
  );
  

  const { data: systemclock } = useSystemClock();

  if (!asset) return null;
  if (!systemclock)
    return (
      <div className="p-4">
        <h2 className="text-2xl font-bold mb-4">Work Orders – {asset}</h2>
        <p className="text-red-600">
          Could not fetch system clock. Server is non-responding.
        </p>
      </div>
      
    );

  // Handling Scheduling data varians
  if (isLoading)
    return (
      <div className="p-4">
        <h2 className="text-2xl font-bold mb-4">Work Orders – {asset}</h2>
        <p className="text-gray-700">Loading…</p>
      </div>
    );

  if (isError)
    return (
      <div className="p-4">
        <h2 className="text-2xl font-bold mb-4">Work Orders – {asset}</h2>
        <p className="text-red-600">
          Couldn’t fetch work orders.<br />
          <code>{(error as Error).message}</code>
        </p>
      </div>
    );

  return (
    <div className="p-4 flex flex-col flex-1 min-h-0 overflow-hidden">
      <div className="flex justify-between">
        <h2 className="text-2xl font-bold mb-4 shrink-0">Work Orders - {asset}: {format(parseISO(systemclock), "PPP p")}</h2>
        <div className='flex items-center gap-2 px-2 py-1 text-gray-700 text-xs rounded-md font-medium'>
          Version: {query.data?.version}
          <StagnationDot stagnations={query.data?.stagnation_iterations} />
        </div>
      </div>
        <AgGridReact
          className='h-full w-full'
          rowData={workOrders}
          columnDefs={columns}
          context={{
            asset: asset
            
          }}
          defaultColDef={{
            resizable: true,
            sortable: true,
            filter: true,
          }}
        />
    </div>
  );
};

export default Scheduler;
