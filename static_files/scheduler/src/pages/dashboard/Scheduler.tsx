import React, { useMemo } from 'react';
import { AgGridReact } from 'ag-grid-react';
import { ColDef, } from 'ag-grid-community';
import { useParams, useNavigate } from 'react-router-dom';
import { useQuery } from "@tanstack/react-query";

import { SchedulerWorkOrderDto } from "../../../../../crates/ordinator-contracts/bindings/SchedulerWorkOrderDto.ts";
import { SingleRowDto } from "../../../../../crates/ordinator-contracts/bindings/SingleRowDto.ts";

type SingleRowDtoWithOptions = SingleRowDto & {"menu": string};

// import { agGridThemeLight } from "./theme";
// https://www.youtube.com/watch?v=TrKlKF5au6c&ab_channel=m6io
/** snake_case → "Title Case" */
const toTitle = (s: string): string =>
  s.replace(/_/g, " ").replace(/\w\S*/g, w => w[0].toUpperCase() + w.slice(1));


export const useSchedulerColumns = () =>
  useMemo<ColDef<SingleRowDto>[]>(() => {
    const fields: (keyof SingleRowDto)[] = [
      "scheduled_period",
      "scheduled_start_date",
      "priority",
      "revision",
      "work_order_type",
      "main_work_ctr",
      "operation_work_center",
      "work_order_number",
      "description_work_order",
      "operation_short_text",
      "material_status",
      "system_status",
      "user_status",
      "work",
      "actual_work",
      "unloading_point",
      "basic_start_date",
      "basic_finish_date",
      "earliest_start_date",
      "earliest_finish_date",
      "earliest_allowed_start_date",
      "latest_allowed_finish_date",
      "activity",
      "functional_location",
      "description_operation",
      "subnetwork_of",
      "system_condition",
      "maintenance_plan",
      "planner_group",
      "maintenance_plant",
      "pm_collective",
      "room",
    ];
    return fields.map<ColDef<SingleRowDto>>(field => ({
      field,
      headerName: toTitle(field),
      pinned: ["scheduled_period", "work_order_number"].includes(field)
        ? "left"
        : undefined,
      width: 150,
    }));
  }, []);
  
const fetchWorkOrders = async (
  asset: string,
): Promise<SingleRowDto[]> => {
  const res = await fetch(
    `/api/v1/scheduler/scheduler/work_orders_with_scheduling/${asset}`,
  );

  if (!res.ok) {
    // propagate a rejected promise so React Query sets "error"
    const body = await res.text();
    throw new Error(`(${res.status}) ${body}`);
  }

  // The Rust DTO is `Vec<SingleRowDto>` => serialises to a bare JSON array
  return (await res.json()) as SchedulerWorkOrderDto;
};

// const onReschedule = (row: SingleRowDto) => {
//   // TODO Do something
//   console.log("Reschedule workorder:", row.work_order_number);
// }


const Scheduler: React.FC = () => {
  const { asset } = useParams<{ asset: string }>();
  const navigate = useNavigate();
  
  /* Safeguard against missing /asset */
  React.useEffect(() => {
    if (!asset) navigate("/dashboard/DF");
  }, [asset, navigate]);

  const columnDefs = useSchedulerColumns();
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
          // theme={agGridThemeLight}
          columnDefs={columnDefs}
          defaultColDef={{
            resizable: true,
            sortable: true,
            filter: true,
            flex: 1,
            minWidth: 100
          }}

          // NOTE This is an enterprise feature costing 999 USD pr developer.
          // getContextMenuItems={(
          //   params: GetContextMenuItemsParams<SingleRowDto>
          // ) => {
          //   const defaultItems = [
          //     "copy",
          //     "copyWithHeaders",
          //     "export"
          //   ];

          //   const row = params.node?.data;
          //   if (!row) return defaultItems;
            
          //   const customItems: any[] = [
          //     {
          //       name: "Reschedule",
          //       action: () => onReschedule(row),
          //       icon: `<i class="fas fa-calendar-alt"></i>`
          //     },
          //     {
          //       name: "Lock In Period",
          //       action: () => {
          //         console.log("Lock: ", row.work_order_number);
          //       }
          //     }
          //   ];

          //   return [...defaultItems, "separator", ...customItems];
          // }}
        />
      </div>
    </div>
  );
};

export default Scheduler;
