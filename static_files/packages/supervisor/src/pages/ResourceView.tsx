import { Button } from "@/components/ui/button";
import { Calendar } from "@/components/ui/calendar";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { NaiveDateDto, SupervisorAllAvailableTechnicians, TechnicianAvailability, useDays, useTechnicianAvailability } from "@scipo-code/shared";
import { format } from "date-fns";
import { ChevronLeft, ChevronRight } from "lucide-react";
import 'react-day-picker/dist/style.css';
import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { Badge } from "@/components/ui/badge";


export default function ResourceView() {
  const { asset } = useParams();
  const { data: days, isLoading: isDaysLoading } = useDays();

  // ISSUE #000 The Supervisor ID does not have any meaningful functionality and will have to be fixed
  // in the future!
  const supervisorId = "main";
  const { data: availableTechnicians, isLoading: isTechniciansLoading } = useTechnicianAvailability(asset || "", supervisorId);
  const [currentDayIndex, setCurrentDayIndex] = useState(0);
  const [selectedWorkCenters, setSelectedWorkCenters] = useState<string[]>([]);
  const [hasUserInteracted, setHasUserInteracted] = useState(false);

  const workCenters = availableTechnicians ? [...new Set(availableTechnicians.all_technicians.flatMap(tech => tech.resources) || [])] : [];

  useEffect(() => {
    if (workCenters.length > 0 && selectedWorkCenters.length === 0 && !hasUserInteracted) {
      setSelectedWorkCenters([workCenters[0]]);
    }
  }, [selectedWorkCenters.length, workCenters, hasUserInteracted]);

  const toggleWorkCenter = (workCenter: string) => {
    setSelectedWorkCenters(prev =>
      prev.includes(workCenter)
        ? prev.filter(wc => wc !== workCenter)
        : [...prev, workCenter]
    );
  };

  const clearSelection = () =>  {
    setHasUserInteracted(true);
    setSelectedWorkCenters([]);

  }

  console.log(selectedWorkCenters);
  if (!asset) return <div>Asset not found</div>;
  if (isDaysLoading || isTechniciansLoading) return <div>Loading...</div>;
  if (!days || !availableTechnicians) {
    return <div>Error loading data</div>;
  };

  const weekDays = days.slice(currentDayIndex, currentDayIndex + 14);



  const navigatePeriod = (direction: 'prev' | 'next') => {
    setCurrentDayIndex(prev => {
      const newIndex = direction === 'next'? prev + 8 : prev - 8;
      return Math.max(0, Math.min(newIndex, (days.length || 0) - 7));
    });
  };

  const canGoPrevPeriod = currentDayIndex > 0;
  const canGoNextPeriod = currentDayIndex + 7 < (days.length || 0);

  return (
    <div className="p-4 flex flex-col flex-1 min-h-0 overflow-hidden">
      <div className="flex items-center justify-between mb-4 shrink-0">
        <h2 className="text-2xl font-bold">Resource Availability</h2>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={() => navigatePeriod('prev')} disabled={!canGoPrevPeriod}>
            <ChevronLeft className="h-4 w-4" />
          </Button>
          <span className="font-medium px-4">
            {days[currentDayIndex]} - {days[Math.min(days.length - 1, currentDayIndex + 14)]}
          </span>
          <Button variant="outline" size="sm" onClick={() => navigatePeriod('next')} disabled={!canGoNextPeriod}>
            <ChevronRight className="h-4 w-4" />
          </Button>
        </div>
      </div>

      <div className="flex gap-4">
        <div className="flex-1 min-h-0">
        {selectedWorkCenters ? (
            <GanttView
              technicians={availableTechnicians.all_technicians.filter(tech => tech.resources.some(resource => selectedWorkCenters.includes(resource)))}
              weekDays={weekDays}
              workCenters={selectedWorkCenters}
            /> ) : (
            <div className="flex-1 flex items-center justify-center text-gray-500">
              Please select a work center to view technicians
            </div>
            )
          }
        </div>
        <div className="w-60 flex-shrink-0">
          <WorkCenterSidebar workCenters={workCenters}
            selectedWorkCenters={selectedWorkCenters}
            onToggle={toggleWorkCenter}
            onClear={clearSelection}
            technicians={availableTechnicians}
            />
        </div>
      </div>
    </div>
  );
}

function WorkCenterSidebar({ workCenters, selectedWorkCenters, onToggle, onClear, technicians}: {
  workCenters: string[],
  selectedWorkCenters: string[],
  onToggle: (value: string) => void,
  onClear: () => void,
  technicians: SupervisorAllAvailableTechnicians,
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Select Work Centers</CardTitle>
        <Button variant="outline" size="sm" onClick={onClear}>clear</Button>
      </CardHeader>
      <CardContent className="flex flex-col gap-2">
        {workCenters.map(wc => (
          <div
            key={wc}
            className={`flex justify-between p-2 rounded cursor-pointer hover:bg-gray-100 ${
              selectedWorkCenters.includes(wc) ? 'bg-blue-100' : ''
            }`}
            onClick={() => onToggle(wc)}
          >
            <span>{wc}</span>
            <Badge>{technicians.all_technicians.filter(tech => tech.resources.includes(wc)).length}</Badge>
          </div>
        ))}
      </CardContent>
    </Card>
  )
}


interface GanttViewProps {
  technicians: TechnicianAvailability[];
  weekDays: NaiveDateDto[];
  workCenters: string[];
}

function GanttView({ technicians, weekDays, workCenters }: GanttViewProps) {
  return (
    <Card className="flex-1 min-h-0 flex flex-col">
      <CardHeader className="shrink-0">
        <div className="flex items-center justify-between">
        <CardTitle className="text-lg">
          Weekly Availability
         </CardTitle>
         <AddTechnicianDialog/>
        </div>
      </CardHeader>
      <CardContent className="flex-1 min-h-0 overflow-auto">
        <div>
          {/* Day headers */}
          <div className="grid grid-cols-15 gap-1 mb-2 sticky top-0 bg-white z-10">
            <div className="font-medium p-2">Tech</div>
            {weekDays.map(day => (
              <div key={day} className="font-medium p-2 text-center border rounded">
                <div>{format(day, "EEE")}</div>
                <div className="text-sm text-gray-500">{format(day, "MMM d")}</div>
              </div>
            ))}
          </div>
          
          {/* Technician rows */}
          {workCenters.map(wc => (
            <div key={wc}>
              <div className="flex rounded justify-center bg-blue-300 mb-2">
                <span className="font-medium">{wc}</span>
              </div>
              {technicians.filter(tech => tech.resources.includes(wc)).map(technician => (
                <TechnicianRow key={technician.id} technician={technician} weekDays={weekDays} />
              ))}
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}

          // {workCenters.map(wc => (
          //   <div key={wc}
          //     <div className="flex rounded justify-center bg-blue-300 mb-2">
          //       <span className="font-medium">{wc}</span>
          //     </div>
          //     {technicians.filter(tech => tech.resources.includes(wc)).map(technician => (
          //       <TechnicianRow key={technician.id} technician={technician} weekDays={weekDays} />
          //     ))}
          //   </div>
          // ))}
interface TechnicianRowProps {
  technician: TechnicianAvailability;
  weekDays: NaiveDateDto[];
}

function TechnicianRow({ technician, weekDays }: TechnicianRowProps) {
  return (
    <div className="grid grid-cols-15 gap-1 mb-2">
      <div className="p-2 font-medium border rounded bg-gray-50">
        {technician.id}
      </div>
      {weekDays.map(day => (
        <DayCell key={day} stringDay={day} technician={technician} />
      ))}
    </div>
  );
}

interface DayCellProps {
  stringDay: NaiveDateDto;
  technician: TechnicianAvailability;
}

function DayCell({ stringDay, technician }: DayCellProps) {
  const isAvailable = stringDay >= technician.start && stringDay <= technician.end;
  
  
  return (
    <div className={`flex justify-center border rounded p-2 min-h-[60px] ${isAvailable ? 'bg-green-100' : 'bg-white'}`}>
      {isAvailable ? 'Available' : null }
    </div>
  );
}



function AddTechnicianDialog() {
  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button variant="outline" size="sm">Add Technician</Button>
      </DialogTrigger>

      <DialogContent>
        <DialogHeader>
          <DialogTitle>Add Technician</DialogTitle>
        </DialogHeader>
        <Calendar mode="range" numberOfMonths={1}/>
      </DialogContent>
    </Dialog>
  )
}



// function CalendarPopover() {


  
//   return (
//     <Popover>
//       <PopoverTrigger asChild>
//         <Button
//           variant="outline"
//         >
//           <CalendarIcon/>
//         </Button>
//       </PopoverTrigger>
//       <PopoverContent>
//       </PopoverContent>
//     </Popover>
        
//   )
// }
