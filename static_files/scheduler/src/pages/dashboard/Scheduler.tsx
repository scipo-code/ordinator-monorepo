import React, {memo, useCallback, useMemo } from 'react';
import { AgGridReact } from 'ag-grid-react';
import { ColDef, ICellRendererParams, } from 'ag-grid-community';
import { useParams, useNavigate } from 'react-router-dom';
import { useQuery } from "@tanstack/react-query";

import { SingleRowDto } from "../../../../../crates/ordinator-contracts/bindings/SingleRowDto.ts";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu.tsx';
import { MoreHorizontal } from 'lucide-react';

type SchedulingData = SingleRowDto & {action: string | null};

// import { agGridThemeLight } from "./theme";
// https://www.youtube.com/watch?v=TrKlKF5au6c&ab_channel=m6io
/** snake_case → "Title Case" */


const ActionMenu: React.FC<ICellRendererParams<SchedulingData>> = memo((({ data }) => {
  const handleSelect = useCallback(() => {
    if (data) console.log(data.work_order_number);
  }, [data]);
  
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button onClick={(e) => e.stopPropagation} className='p-1'>
          <MoreHorizontal size={14} />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent side='right' align='start' onClick={(e) => e.stopPropagation()}>
        <DropdownMenuItem onClick={handleSelect}>Accept</DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
  
}));
  
function useTableColDefs(): ColDef<SchedulingData>[] {
  return useMemo((() => {
    const base: ColDef[] = [
      {
        field: 'scheduled_period',
        pinned: "left",
        width: 120,
      },
      {
        field: 'scheduled_start_date',
        pinned: "left",
        width: 120,
      },
      {
        field: 'work_order_number',
        pinned: "left",
        width: 130,
      },
      {
        field: 'action',
        headerName: "",
        width: 15,
        cellStyle: {textAlign: "center"},
        sortable: false,
        filter: false,
        editable: false,
        suppressSizeToFit: true,
        pinned: "left",
        cellRenderer: ActionMenu,         
      },
      {
         field: 'priority',
         width: 100,
      },
      {
         field: 'revision',
         width: 100,
      },
      {
         field: 'work_order_type',
         width: 100,
      },
      {
         field: 'main_work_ctr',
         width: 100,
      },
      {
         field: 'operation_work_center',
         width: 100,
      },
      {
         field: 'description_work_order',
         width: 100,
      },
      {
         field: 'operation_short_text',
         width: 100,
      },
      {
         field: 'material_status',
         width: 100,
      },
      {
         field: 'system_status',
         width: 100,
      },
      {
         field: 'user_status',
         width: 100,
      },
      {
         field: 'work',
         width: 100,
      },
      {
         field: 'actual_work',
         width: 100,
      },
      {
         field: 'unloading_point',
         width: 100,
      },
      {
         field: 'basic_start_date',
         width: 100,
      },
      {
         field: 'basic_finish_date',
         width: 100,
      },
      {
         field: 'earliest_start_date',
         width: 100,
      },
      {
         field: 'earliest_finish_date',
         width: 100,
      },
      {
         field: 'earliest_allowed_start_date',
         width: 100,
      },
      {
         field: 'latest_allowed_finish_date',
         width: 100,
      },
      {
         field: 'activity',
         width: 100,
      },
      {
         field: 'functional_location',
         width: 100,
      },
      {
         field: 'description_operation',
         width: 100,
      },
      {
         field: 'subnetwork_of',
         width: 100,
      },
      {
         field: 'system_condition',
         width: 100,
      },
      {
         field: 'maintenance_plan',
         width: 100,
      },
      {
         field: 'planner_group',
         width: 100,
      },
      {
         field: 'maintenance_plant',
         width: 100,
      },
      {
         field: 'pm_collective',
         width: 100,
      },
      {
         field: 'room',
         width: 100,
      }
    ];
    return base;
  }), [])
};
  
const fetchWorkOrders = async (
  asset: string,
): Promise<SchedulingData[]> => {
  const res = await fetch(
    `/api/v1/scheduler/scheduler/work_orders_with_scheduling/${asset}`,
  );

  if (!res.ok) {
    // propagate a rejected promise so React Query sets "error"
    const body = await res.text();
    throw new Error(`(${res.status}) ${body}`);
  }

  // The Rust DTO is `Vec<SingleRowDto>` => serialises to a bare JSON array
  return (await res.json()) as SchedulingData[];
};

// const onReschedule = (row: SingleRowDto) => {
//   // TODO Do something
//   console.log("Reschedule workorder:", row.work_order_number);
// }


const Scheduler: React.FC = () => {
  const { asset } = useParams<{ asset: string }>();
  const navigate = useNavigate();
  const columns = useTableColDefs();
  
  /* Safeguard against missing /asset */
  React.useEffect(() => {
    if (!asset) navigate("/dashboard/DF");
  }, [asset, navigate]);

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

  if (!asset) return null;

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
