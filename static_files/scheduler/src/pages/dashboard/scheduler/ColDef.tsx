import { ColDef, ICellRendererParams, } from 'ag-grid-community';
import { memo, useCallback, useMemo } from "react";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu.tsx';
import { MoreHorizontal } from 'lucide-react';

import { SingleRowDto } from "../../../../../../crates/ordinator-contracts/bindings/SingleRowDto.ts";
export type SchedulingData = SingleRowDto & {action: string | null};


interface ActionMenuParams extends ICellRendererParams<SchedulingData> {
   context: {
      onAssign: (row: SchedulingData) => void;
   };
}


const ActionMenu: React.FC<ActionMenuParams> = memo((({ data, context }) => {
  const handleSelect = useCallback(() => {
     if (data) context.onAssign(data);
  }, [data,context]);
  
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button onClick={(e) => e.stopPropagation} className='p-1'>
          <MoreHorizontal size={14} />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent side='right' align='start' onClick={(e) => e.stopPropagation()}>
        <DropdownMenuItem onClick={handleSelect}>Accept {data?.suggested_scheduled_period}</DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
  
}));
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
        field: 'action',
        headerName: "",
        minWidth: 15,
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
