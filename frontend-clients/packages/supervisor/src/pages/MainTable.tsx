
import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Actors, NaiveDateDto, StagnationDot, SupervisorMainTableDto, useDays, useSystemClock } from '@scipo-code/shared';
import { useSupervisorMainTable } from "@scipo-code/shared";
import { format, parseISO } from "date-fns";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";




export default function MainTable() {
  const { asset } = useParams();

  const { data: systemclock } = useSystemClock();
  const { data: days } = useDays();

  const [ selectedDay, setSelectedDay ] = useState<NaiveDateDto | null>(null);
  // ISSUE #000 The Supervisor ID does not have any meaningful functionality and will have to be fixed
  // in the future!
  const supervisorId = "main";
  const { data: mainTableData } = useSupervisorMainTable(asset || "", supervisorId, selectedDay ? selectedDay : undefined );


  useEffect(() => {
    if (!selectedDay && days) {
      setSelectedDay(days[0]);
    }
  }, [days]);
    
  if (!asset) return <div>Asset not found</div>;

  if (!systemclock || !days)
    return (
      <div className="p-4">
        <p className="text-red-600">
          Could not retrieve time environment. Server is non-responding.
        </p>
      </div>
      
    );
  if (!selectedDay) return <div>Loading days...</div>;
  if (!mainTableData) return <div>Loading data...</div>;
  

  
  return (
    <div className="p-4 flex flex-col flex-1 min-h-0 overflow-hidden">
      <h2 className="text-2xl font-bold mb-4 shrink-0">Work Schedule - {asset} : Server clock - {format(parseISO(systemclock), "PPP p")}</h2>
      <div className="flex justify-between items-center">
        <div className="flex items-center gap-2 mb-4 shrink-0">
          <Button
            variant="outline"
            size="sm"
            onClick={() => {
              const currentIndex = days?.indexOf(selectedDay!) || 0;
              if (currentIndex > 0) setSelectedDay(days![currentIndex -1]);
            }}
            disabled={!days || !selectedDay || days.indexOf(selectedDay) === 0}
          >
          <ChevronLeft className="h-4 w-4" />
          </Button>

          <Select value={selectedDay || ""} onValueChange={(value) => setSelectedDay(value as NaiveDateDto)}>
            <SelectTrigger className="w-40">
              <SelectValue placeholder="Select date" />
            </SelectTrigger>
            <SelectContent>
              {days?.map(day => (
                <SelectItem key={day} value={day}>{day}</SelectItem>
              ))}
            </SelectContent>
          </Select>

        
          <Button
            variant="outline"
            size="sm"
            onClick={() => {
              const currentIndex = days?.indexOf(selectedDay!) || 0;
              if (currentIndex < (days?.length || 0) - 1) setSelectedDay(days![currentIndex +1]);
            }}
            disabled={!days || !selectedDay || days.indexOf(selectedDay) === (days?.length || 0) -1}
          >
          <ChevronRight className="h-4 w-4" />
          </Button>
        </div>
        <div className='flex items-center gap-2 px-2 py-1 text-gray-700 text-xs rounded-md font-medium'>
          Solution Stability
          <StagnationDot asset={asset} actor={Actors.supervisor}/>
        </div>
      </div>
      <WorkSchedule data={mainTableData} selectedDay={selectedDay} />

      
    </div>
  )
}

const formatColumnName = (column: string) => {
  return column.replace(/_/g, ' ').replace(/\b\w/g, l => l.toLocaleUpperCase())
}

function WorkSchedule({data, selectedDay}: {data: SupervisorMainTableDto,selectedDay: NaiveDateDto}) {

  if (!data.days || !selectedDay) return <div>Missing Data</div>;

  const inner_data = data.days[selectedDay]?.work_order_activities_per_work_center;

  if (!inner_data ) return <div>Missing Data</div>;

  const firstWorkCenter = Object.values(inner_data).find(rows => rows && rows.length > 0);

  if (!firstWorkCenter || firstWorkCenter.length === 0) {
    return <div>No work order data </div>
  };

  const columns = Object.keys(firstWorkCenter[0]);
  
  return (
    <div className="overflow-auto max-h-[70vh]">
      <Table>
        <TableHeader>
          <TableRow>
            {columns.map(column => (
              <TableHead key={column}>{formatColumnName(column)}</TableHead>
            ))}
          </TableRow>
        </TableHeader>
        {Object.entries(inner_data).map(([workCenterKey, workCenterRows]) => (
          <TableBody key={workCenterKey}>
            <TableRow>
              <TableCell className="bg-blue-100 font-semibold px-2 py-1 text-center" colSpan={columns.length}>{workCenterKey}</TableCell>
            </TableRow>
            {workCenterRows?.map((row, index) => (
              <TableRow key={index} className="hover:bg-gray-100">
                {columns.map(column => (
                  <TableCell key={column}>
                    {String(row[column as keyof typeof row])}
                  </TableCell>
                ))}
              </TableRow>
            ))}
      
          </TableBody>
        ))}
      </Table>
    </div>
  )
}
