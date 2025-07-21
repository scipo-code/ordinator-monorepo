import { ColDef, ICellRendererParams, } from 'ag-grid-community';
import { useMemo } from "react";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu.tsx';
import { MoreHorizontal } from 'lucide-react';

import { SingleRowDto } from "../../../../../../crates/ordinator-contracts/bindings/SingleRowDto.ts";
export type SchedulingData = SingleRowDto & {action: string | null};

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
export function useTableColDefs(): ColDef<SchedulingData>[] {
  return useMemo((() => {
    const base: ColDef[] = [
      {
        field: 'scheduled_period',
        pinned: "left",
        minWidth: 80,
      },
      {
        field: 'scheduled_start_date',
        pinned: "left",
        minWidth: 110,
      },
      {
        field: 'work_order_number',
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
         minWidth: 100,
      },
      {
         field: 'main_work_ctr',
         minWidth: 100,
      },
      {
         field: 'operation_work_center',
         minWidth: 100,
      },
      {
         field: 'description_work_order',
         minWidth: 100,
      },
      {
         field: 'operation_short_text',
         minWidth: 100,
      },
      {
         field: 'material_status',
         minWidth: 100,
      },
      {
         field: 'system_status',
         minWidth: 100,
      },
      {
         field: 'user_status',
         minWidth: 100,
      },
      {
         field: 'work',
         minWidth: 100,
      },
      {
         field: 'actual_work',
         minWidth: 100,
      },
      {
         field: 'unloading_point',
         minWidth: 100,
      },
      {
         field: 'basic_start_date',
         minWidth: 100,
      },
      {
         field: 'basic_finish_date',
         minWidth: 100,
      },
      {
         field: 'earliest_start_date',
         minWidth: 100,
      },
      {
         field: 'earliest_finish_date',
         minWidth: 100,
      },
      {
         field: 'earliest_allowed_start_date',
         minWidth: 100,
      },
      {
         field: 'latest_allowed_finish_date',
         minWidth: 100,
      },
      {
         field: 'activity',
         minWidth: 100,
      },
      {
         field: 'functional_location',
         minWidth: 100,
      },
      {
         field: 'description_operation',
         minWidth: 100,
      },
      {
         field: 'subnetwork_of',
         minWidth: 100,
      },
      {
         field: 'system_condition',
         minWidth: 100,
      },
      {
         field: 'maintenance_plan',
         minWidth: 100,
      },
      {
         field: 'planner_group',
         minWidth: 100,
      },
      {
         field: 'maintenance_plant',
         minWidth: 100,
      },
      {
         field: 'pm_collective',
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
