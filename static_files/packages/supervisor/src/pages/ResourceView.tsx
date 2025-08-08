import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { addDays, format, isWithinInterval, startOfDay } from "date-fns";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { useState } from "react";

interface TechnicianAvailability {
  id: string;
  name: string;
  workCenter: string;
  availabilities: {
    startDate: Date;
    endDate: Date;
    type: "available" | "scheduled" | "unavailable";
    description?: string;
  }[];
}

const mockData: TechnicianAvailability[] = [
  {
    id: "tech-1",
    name: "John Smith",
    workCenter: "Mechanical",
    availabilities: [
      {
        startDate: new Date(2025, 0, 6, 8, 0),
        endDate: new Date(2025, 0, 6, 16, 0),
        type: "available"
      },
      {
        startDate: new Date(2025, 0, 7, 8, 0),
        endDate: new Date(2025, 0, 7, 12, 0),
        type: "scheduled",
        description: "Pump maintenance"
      },
      {
        startDate: new Date(2025, 0, 8, 8, 0),
        endDate: new Date(2025, 0, 8, 16, 0),
        type: "available"
      }
    ]
  },
  {
    id: "tech-2", 
    name: "Sarah Johnson",
    workCenter: "Mechanical",
    availabilities: [
      {
        startDate: new Date(2025, 0, 6, 9, 0),
        endDate: new Date(2025, 0, 6, 17, 0),
        type: "scheduled",
        description: "Compressor repair"
      },
      {
        startDate: new Date(2025, 0, 7, 8, 0),
        endDate: new Date(2025, 0, 7, 16, 0),
        type: "available"
      },
      {
        startDate: new Date(2025, 0, 8, 8, 0),
        endDate: new Date(2025, 0, 8, 12, 0),
        type: "unavailable",
        description: "Training"
      }
    ]
  },
  {
    id: "tech-3",
    name: "Mike Chen",
    workCenter: "Electrical",
    availabilities: [
      {
        startDate: new Date(2025, 0, 6, 8, 0),
        endDate: new Date(2025, 0, 6, 16, 0),
        type: "available"
      },
      {
        startDate: new Date(2025, 0, 7, 10, 0),
        endDate: new Date(2025, 0, 7, 14, 0),
        type: "scheduled",
        description: "Panel upgrade"
      },
      {
        startDate: new Date(2025, 0, 8, 8, 0),
        endDate: new Date(2025, 0, 8, 16, 0),
        type: "available"
      }
    ]
  },
  {
    id: "tech-4",
    name: "Lisa Davis",
    workCenter: "Electrical",
    availabilities: [
      {
        startDate: new Date(2025, 0, 6, 8, 0),
        endDate: new Date(2025, 0, 6, 12, 0),
        type: "scheduled",
        description: "Motor testing"
      },
      {
        startDate: new Date(2025, 0, 7, 8, 0),
        endDate: new Date(2025, 0, 7, 16, 0),
        type: "available"
      },
      {
        startDate: new Date(2025, 0, 8, 14, 0),
        endDate: new Date(2025, 0, 8, 16, 0),
        type: "unavailable",
        description: "Meeting"
      }
    ]
  }
];

export default function ResourceView() {
  const [currentWeekStart, setCurrentWeekStart] = useState(startOfDay(new Date(2025, 0, 6))); // Monday Jan 6, 2025
  
  const weekDays = Array.from({ length: 7 }, (_, i) => addDays(currentWeekStart, i));
  
  const workCenters = [...new Set(mockData.map(tech => tech.workCenter))];
  
  const navigateWeek = (direction: 'prev' | 'next') => {
    setCurrentWeekStart(prev => addDays(prev, direction === 'next' ? 7 : -7));
  };
  
  return (
    <div className="p-4 flex flex-col flex-1 min-h-0 overflow-hidden">
      <div className="flex items-center justify-between mb-4 shrink-0">
        <h2 className="text-2xl font-bold">Resource Availability</h2>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={() => navigateWeek('prev')}>
            <ChevronLeft className="h-4 w-4" />
          </Button>
          <span className="font-medium px-4">
            {format(currentWeekStart, "MMM d")} - {format(addDays(currentWeekStart, 6), "MMM d, yyyy")}
          </span>
          <Button variant="outline" size="sm" onClick={() => navigateWeek('next')}>
            <ChevronRight className="h-4 w-4" />
          </Button>
        </div>
      </div>

      <Tabs defaultValue={workCenters[0]} className="flex flex-col flex-1 min-h-0">
        <TabsList className="shrink-0">
          {workCenters.map(workCenter => (
            <TabsTrigger key={workCenter} value={workCenter}>
              {workCenter}
              <Badge variant="secondary" className="ml-2">
                {mockData.filter(tech => tech.workCenter === workCenter).length}
              </Badge>
            </TabsTrigger>
          ))}
        </TabsList>

        {workCenters.map(workCenter => (
          <TabsContent key={workCenter} value={workCenter} className="flex-1 min-h-0">
            <GanttView 
              technicians={mockData.filter(tech => tech.workCenter === workCenter)}
              weekDays={weekDays}
            />
          </TabsContent>
        ))}
      </Tabs>
    </div>
  );
}

interface GanttViewProps {
  technicians: TechnicianAvailability[];
  weekDays: Date[];
}

function GanttView({ technicians, weekDays }: GanttViewProps) {
  return (
    <Card className="flex-1 min-h-0 flex flex-col">
      <CardHeader className="shrink-0">
        <CardTitle className="text-lg">Weekly Schedule</CardTitle>
      </CardHeader>
      <CardContent className="flex-1 min-h-0 overflow-auto">
        <div className="min-w-[800px]">
          {/* Day headers */}
          <div className="grid grid-cols-8 gap-1 mb-2 sticky top-0 bg-white z-10">
            <div className="font-medium p-2">Technician</div>
            {weekDays.map(day => (
              <div key={day.toISOString()} className="font-medium p-2 text-center border rounded">
                <div>{format(day, "EEE")}</div>
                <div className="text-sm text-gray-500">{format(day, "MMM d")}</div>
              </div>
            ))}
          </div>
          
          {/* Technician rows */}
          {technicians.map(technician => (
            <TechnicianRow key={technician.id} technician={technician} weekDays={weekDays} />
          ))}
        </div>
      </CardContent>
    </Card>
  );
}

interface TechnicianRowProps {
  technician: TechnicianAvailability;
  weekDays: Date[];
}

function TechnicianRow({ technician, weekDays }: TechnicianRowProps) {
  return (
    <div className="grid grid-cols-8 gap-1 mb-2">
      <div className="p-2 font-medium border rounded bg-gray-50">
        {technician.name}
      </div>
      {weekDays.map(day => (
        <DayCell key={day.toISOString()} day={day} technician={technician} />
      ))}
    </div>
  );
}

interface DayCellProps {
  day: Date;
  technician: TechnicianAvailability;
}

function DayCell({ day, technician }: DayCellProps) {
  const dayStart = startOfDay(day);
  const dayEnd = addDays(dayStart, 1);
  
  const dayAvailabilities = technician.availabilities.filter(availability => 
    isWithinInterval(availability.startDate, { start: dayStart, end: dayEnd }) ||
    isWithinInterval(availability.endDate, { start: dayStart, end: dayEnd }) ||
    (availability.startDate < dayStart && availability.endDate > dayEnd)
  );
  
  return (
    <div className="border rounded p-1 min-h-[60px] bg-white">
      {dayAvailabilities.map((availability, index) => (
        <AvailabilityBlock key={index} availability={availability} />
      ))}
    </div>
  );
}

interface AvailabilityBlockProps {
  availability: TechnicianAvailability['availabilities'][0];
}

function AvailabilityBlock({ availability }: AvailabilityBlockProps) {
  const getTypeColor = () => {
    switch (availability.type) {
      case "available": return "bg-green-200 text-green-800 border-green-300";
      case "scheduled": return "bg-blue-200 text-blue-800 border-blue-300";
      case "unavailable": return "bg-red-200 text-red-800 border-red-300";
    }
  };
  
  return (
    <div className={`text-xs p-1 mb-1 rounded border ${getTypeColor()} hover:opacity-80 cursor-pointer`}>
      <div className="font-medium capitalize">{availability.type}</div>
      <div className="text-xs">
        {format(availability.startDate, "HH:mm")} - {format(availability.endDate, "HH:mm")}
      </div>
      {availability.description && (
        <div className="text-xs opacity-75 truncate">
          {availability.description}
        </div>
      )}
    </div>
  );
}
