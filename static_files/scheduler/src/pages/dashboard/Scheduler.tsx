import React, { useCallback, useEffect } from 'react';
import { AgGridReact } from 'ag-grid-react';
import { useParams, Navigate } from 'react-router-dom';
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { SchedulingData, useTableColDefs } from './scheduler/ColDef';
import { fetchPeriods, fetchWorkOrders, useAssignWorkorderToPeriod } from './scheduler/ApiHandlers';
import { PeriodDto } from '../../../../../crates/ordinator-contracts/bindings/PeriodDto';


// import { agGridThemeLight } from "./theme";
// https://www.youtube.com/watch?v=TrKlKF5au6c&ab_channel=m6io
/** snake_case → "Title Case" */

const Scheduler: React.FC = () => {
  const { asset } = useParams<{ asset: string }>();
  const columns = useTableColDefs();
    
  /* Safeguard against missing /asset */
  if (!asset) {
    return <Navigate to="/dashboard/DF" replace />;
  }

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


  const {
    data: periods = [],
    // isLoading: periodsLoading,
    // isError: periodsError,
  } = useQuery({
    queryKey: ["periods"],
    queryFn: fetchPeriods,
    staleTime: Infinity,
  });

  const qc = useQueryClient();
  useEffect(() => {
    qc.prefetchQuery({
      queryKey: ["periods"],
      queryFn: fetchPeriods
    });
  }, [qc]);

  
  const assignMutation = useAssignWorkorderToPeriod();
  const handleAssignPeriod = useCallback(
    (row: SchedulingData, period?: PeriodDto) => {
      const chosenPeriod = period ?? (row.suggested_scheduled_period as PeriodDto | undefined);

      if (!chosenPeriod) return;
      
      assignMutation.mutate({
        asset,
        workorder: row.work_order_number,
        period: chosenPeriod,
      })
    },
    [asset, assignMutation],
  )

  console.log(periods);

  
  if (!asset) return null;


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
      <h2 className="text-2xl font-bold mb-4 shrink-0">Work Orders - {asset}</h2>
      <div className="flex-1 min-h-0">
        <AgGridReact
          className='h-full w-full'
          rowData={workOrders}
          columnDefs={columns}
          context={
          {
            onAssignPeriod: handleAssignPeriod,
            periods: periods
          }
        }
          defaultColDef={{
            resizable: true,
            sortable: true,
            filter: true,
            flex: 1,
          }}
        />
      </div>
    </div>
  );
};

export default Scheduler;
