import { ColDef, ICellRendererParams, } from 'ag-grid-community';
import { useMemo } from "react";
import { DropdownMenu,
   DropdownMenuContent,
   DropdownMenuItem,
   DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu.tsx';
import { MoreHorizontal } from 'lucide-react';

import { useNavigate } from 'react-router-dom';
import { PeriodStatusICellRenderer } from "@scipo-code/shared";
import { SchedulingData } from "@scipo-code/shared";



interface ActionMenuParams extends ICellRendererParams<SchedulingData> {
   context: {
      asset: string
   }
}


const ActionMenu: React.FC<ActionMenuParams> = (({ data, context }) => {
   if (!data) return null;
   const navigate = useNavigate();

   const goToEditWorkorder = (e: React.MouseEvent) => {
      e.stopPropagation();

      if (data?.work_order_number) {
         navigate(`/${context.asset}/dashboard/${data.work_order_number}`);
      }
   }

   return (
      <DropdownMenu>
         <DropdownMenuTrigger asChild>
            <button onClick={(e) => e.stopPropagation()} className='p-1'>
               <MoreHorizontal size={14} />
            </button>
         </DropdownMenuTrigger>
         <DropdownMenuContent side='right' align='start' onClick={(e) => e.stopPropagation()}>
            <DropdownMenuItem onClick={goToEditWorkorder}>
               Edit Scheduling
            </DropdownMenuItem>
         </DropdownMenuContent>
      </DropdownMenu>
   );
});



export function useTableColDefs(): ColDef<SchedulingData>[] {
  return useMemo((() => {
    const base: ColDef[] = [
      {
        field: 'suggested_scheduled_period',
        headerName: 'suggested scheduled period',
        pinned: "left",
        width: 130,
      },
      {
        field: 'scheduled_start_date',
        headerName: 'scheduled start date',
        pinned: "left",
        width: 130,
      },
      {
        field: 'work_order_number',
        headerName: 'work order number',
        pinned: "left",
        width: 130,
      },
      {
        field: 'period_status',
        headerName: 'period status',
        cellStyle: {textAlign: "center"},
        pinned: "left",
        width: 130,
        cellRenderer: PeriodStatusICellRenderer,
      },
      {
        field: 'action',
        headerName: "",
        width: 60,
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
         headerName: 'work order type',
         width: 100,
      },
      {
         field: 'main_work_ctr',
         headerName: 'main work ctr',
         width: 100,
      },
      {
         field: 'operation_work_center',
         headerName: 'operation work center',
         width: 100,
      },
      {
         field: 'description_work_order',
         headerName: 'description work order',
         width: 100,
      },
      {
         field: 'operation_short_text',
         headerName: 'operation short text',
         width: 100,
      },
      {
         field: 'material_status',
         headerName: 'material status',
         width: 100,
      },
      {
         field: 'system_status',
         headerName: 'system status',
         width: 100,
      },
      {
         field: 'user_status',
         headerName: 'user status',
         width: 100,
      },
      {
         field: 'work',
         width: 100,
      },
      {
         field: 'actual_work',
         headerName: 'actual work',
         width: 100,
      },
      {
         field: 'unloading_point',
         headerName: 'unloading point',
         width: 100,
      },
      {
         field: 'basic_start_date',
         headerName: 'basic start date',
         width: 100,
      },
      {
         field: 'basic_finish_date',
         headerName: 'basic finish date',
         width: 100,
      },
      {
         field: 'earliest_start_date',
         headerName: 'earliest start date',
         width: 100,
      },
      {
         field: 'earliest_finish_date',
         headerName: 'earliest finish date',
         width: 100,
      },
      {
         field: 'earliest_allowed_start_date',
         headerName: 'earliest allowed start date',
         width: 100,
      },
      {
         field: 'latest_allowed_finish_date',
         headerName: 'latest allowed finish date',
         width: 100,
      },
      {
         field: 'activity',
         width: 100,
      },
      {
         field: 'functional_location',
         headerName: 'functional location',
         width: 100,
      },
      {
         field: 'description_operation',
         headerName: 'description operation',
         width: 100,
      },
      {
         field: 'subnetwork_of',
         headerName: 'subnetwork of',
         width: 100,
      },
      {
         field: 'system_condition',
         headerName: 'system condition',
         width: 100,
      },
      {
         field: 'maintenance_plan',
         headerName: 'maintenance plan',
         width: 100,
      },
      {
         field: 'planner_group',
         headerName: 'planner group',
         width: 100,
      },
      {
         field: 'maintenance_plant',
         headerName: 'maintenance plant',
         width: 100,
      },
      {
         field: 'pm_collective',
         headerName: 'pm collective',
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
