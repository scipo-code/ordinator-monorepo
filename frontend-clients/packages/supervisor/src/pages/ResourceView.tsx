import { Button } from "@/components/ui/button";
import {  TechnicianAvailability, useDays, useTechnicianAvailability } from "@scipo-code/shared";
import {  ChevronLeft, ChevronRight } from "lucide-react";
import 'react-day-picker/dist/style.css';
import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { GanttView, ResourceSidebar as ResourceSidebar } from "@/components/ResourceView";
import { Card, CardContent } from "@/components/ui/card";



const groupTechniciansByResources = (technicians: TechnicianAvailability[]) => {
  const grouped = new Map<string, TechnicianAvailability[]>();

  technicians.forEach(tech => {
    const resourceKey = tech.resources.sort().join(' / ');
    if (!grouped.has(resourceKey)) {
      grouped.set(resourceKey, []);
    }
    grouped.get(resourceKey)!.push(tech)
  })

  return grouped;
}

export default function ResourceView() {
  const { asset } = useParams();
  const { data: days, isLoading: isDaysLoading } = useDays();

  // ISSUE #000 The Daily ID does not have any meaningful functionality and will have to be fixed
  // in the future!
  const dailyId = "main";
  const { data: availableTechnicians, isLoading: isTechniciansLoading } = useTechnicianAvailability(asset || "", dailyId);
  const [currentDayIndex, setCurrentDayIndex] = useState(0);
  const [selectedResources, setSelectedResources] = useState<string[]>([]);
  const [hasUserInteracted, setHasUserInteracted] = useState(false);

  const resources = availableTechnicians ? [...new Set(availableTechnicians.all_technicians.flatMap(tech => tech.resources) || [])] : [];

  useEffect(() => {
    if (resources.length > 0 && selectedResources.length === 0 && !hasUserInteracted) {
      setSelectedResources([resources[0]]);
    }
  }, [selectedResources.length, resources, hasUserInteracted]);

  const toggleWorkCenter = (workCenter: string) => {
    setSelectedResources(prev =>
      prev.includes(workCenter)
        ? prev.filter(wc => wc !== workCenter)
        : [...prev, workCenter]
    );
  };

  const clearSelection = () =>  {
    setHasUserInteracted(true);
    setSelectedResources([]);
  }

  const selectAll = () => {
    setSelectedResources(resources);
  }

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



  const filteredTechnicians = availableTechnicians.all_technicians.filter(tech =>
    tech.resources.some(resource => selectedResources.includes(resource))
  );

  const groupedTechnicians = groupTechniciansByResources(filteredTechnicians);
  
  return (
    <div className="p-4 flex flex-col grow min-h-0">
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

      <div className="flex flex-1 min-h-0 gap-2 overflow-hidden w-full">
        <div className="flex-1 min-h-0">
        {selectedResources.length > 0 ? (
            <GanttView
              groupedTechnicians={groupedTechnicians}
              weekDays={weekDays}
            /> ) : (
            <Card className="flex-1 min-h-0 flex flex-col h-full w-full">
              <CardContent className="flex-1 flex items-center justify-center text-gray-500">
                Please select a work center to view technicians
              </CardContent>
            </Card>
            )
          }
        </div>
        <div className="w-60">
          <ResourceSidebar resources={resources}
            selectedResources={selectedResources}
            onToggle={toggleWorkCenter}
            toggleAll={selectAll}
            onClear={clearSelection}
            technicians={availableTechnicians}
            />
        </div>
      </div>
    </div>
  );
}

