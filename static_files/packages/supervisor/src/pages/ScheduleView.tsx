import React from 'react';
import { AgGridReact } from 'ag-grid-react';
import { useParams } from 'react-router-dom';
import { useQuery } from "@tanstack/react-query";
import { parseISO, format } from 'date-fns';
import { fetchWorkOrders } from '@/api/workorders';
import fetchSystemClock from '@/api/clock';
import { ColDef } from 'ag-grid-community';
import { useMemo } from "react";

import { SingleRowDto } from "../../../../crates/ordinator-contracts/bindings/SingleRowDto.ts";
import { PeriodStatusICellRenderer } from '@/components/ui/period_status_badge.tsx';
import fetchPeriods from '@/api/periods.ts';

export type SchedulingData = SingleRowDto;



export function useTableColDefs(): ColDef<SchedulingData>[] {
  return useMemo((() => {
    const base: ColDef[] = [
      {
        field: 'suggested_scheduled_period',
        headerName: 'suggested scheduled period',
        pinned: "left",
        minWidth: 80,
      },
      {
        field: 'scheduled_start_date',
        headerName: 'scheduled start date',
        pinned: "left",
        minWidth: 110,
      },
      {
        field: 'work_order_number',
        headerName: 'work order number',
        pinned: "left",
        minWidth: 130,
      },
      {
        field: 'period_status',
        headerName: 'period status',
        pinned: "left",
        minWidth: 20,
        cellRenderer: PeriodStatusICellRenderer,
      },
      {
         field: 'priority',
         minWidth: 100,
      },
      {
         field: 'revision',
         minWidth: 100,
      },
      {
         field: 'work_order_type',
         headerName: 'work order type',
         minWidth: 100,
      },
      {
         field: 'main_work_ctr',
         headerName: 'main work ctr',
         minWidth: 100,
      },
      {
         field: 'operation_work_center',
         headerName: 'operation work center',
         minWidth: 100,
      },
      {
         field: 'description_work_order',
         headerName: 'description work order',
         minWidth: 100,
      },
      {
         field: 'operation_short_text',
         headerName: 'operation short text',
         minWidth: 100,
      },
      {
         field: 'material_status',
         headerName: 'material status',
         minWidth: 100,
      },
      {
         field: 'system_status',
         headerName: 'system status',
         minWidth: 100,
      },
      {
         field: 'user_status',
         headerName: 'user status',
         minWidth: 100,
      },
      {
         field: 'work',
         minWidth: 100,
      },
      {
         field: 'actual_work',
         headerName: 'actual work',
         minWidth: 100,
      },
      {
         field: 'unloading_point',
         headerName: 'unloading point',
         minWidth: 100,
      },
      {
         field: 'basic_start_date',
         headerName: 'basic start date',
         minWidth: 100,
      },
      {
         field: 'basic_finish_date',
         headerName: 'basic finish date',
         minWidth: 100,
      },
      {
         field: 'earliest_start_date',
         headerName: 'earliest start date',
         minWidth: 100,
      },
      {
         field: 'earliest_finish_date',
         headerName: 'earliest finish date',
         minWidth: 100,
      },
      {
         field: 'earliest_allowed_start_date',
         headerName: 'earliest allowed start date',
         minWidth: 100,
      },
      {
         field: 'latest_allowed_finish_date',
         headerName: 'latest allowed finish date',
         minWidth: 100,
      },
      {
         field: 'activity',
         minWidth: 100,
      },
      {
         field: 'functional_location',
         headerName: 'functional location',
         minWidth: 100,
      },
      {
         field: 'description_operation',
         headerName: 'description operation',
         minWidth: 100,
      },
      {
         field: 'subnetwork_of',
         headerName: 'subnetwork of',
         minWidth: 100,
      },
      {
         field: 'system_condition',
         headerName: 'system condition',
         minWidth: 100,
      },
      {
         field: 'maintenance_plan',
         headerName: 'maintenance plan',
         minWidth: 100,
      },
      {
         field: 'planner_group',
         headerName: 'planner group',
         minWidth: 100,
      },
      {
         field: 'maintenance_plant',
         headerName: 'maintenance plant',
         minWidth: 100,
      },
      {
         field: 'pm_collective',
         headerName: 'pm collective',
         minWidth: 100,
      },
      {
         field: 'room',
         minWidth: 100,
      }
    ];
    return base;
  }), [])
};


// import { agGridThemeLight } from "./theme";
// https://www.youtube.com/watch?v=TrKlKF5au6c&ab_channel=m6io
/** snake_case → "Title Case" */

const SchedulerView: React.FC = () => {
  const { asset } = useParams<{ asset: string }>();
  const columns = useTableColDefs();
    
  /* Fetch with React Query */
  const {
    data: systemclock,
  } = useQuery({
    queryKey: ["systemclock"],
    queryFn: fetchSystemClock,
    staleTime: Infinity,
  });

  const {
     data: periods = [],
  } = useQuery({
     queryKey: ["periods"],
     queryFn: fetchPeriods,
     staleTime: Infinity,
  });
  
  if (!periods) return;

  const query_periods = [0,1].map(x => periods[x]);


  const {
    data: workOrders = [],          // default ↠ empty array
    isLoading,
    isError,
    error,
  } = useQuery({
    queryKey: ["workOrders", asset, query_periods],
    // don’t run until we have a real asset
    enabled: !!asset && !!periods,
    queryFn: () => fetchWorkOrders(asset!, query_periods),
    retry: 2,                       // exponential-backoff retries
    staleTime: 60_000,              // cache 1 min
  });


  if (!asset) return null;
  if (!systemclock) return;

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
      <h2 className="text-2xl font-bold mb-4 shrink-0">Work Orders - {asset}: {format(parseISO(systemclock), "PPP p")}</h2>
      <div className="flex-1 min-h-0">
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
            flex: 1,
          }}
        />
      </div>
    </div>
  );
};

export default SchedulerView;
