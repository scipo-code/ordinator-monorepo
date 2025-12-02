import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { useEffect, useRef, useState } from "react";
import { useParams } from "react-router-dom";

import 'react-day-picker/dist/style.css';
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Button } from "@/components/ui/button";
import { Calendar } from "@/components/ui/calendar";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useDailyLoadings } from "@/hooks";
import type { DailyLoadingDto } from "@/types/dto/DailyLoadingDto";
import type { DailyLoad } from "@/types/dto/DailyLoad";
import { SiteHeader } from "@/components/layout/site-header";



export default function ResourceView() {
  const { asset } = useParams<{asset: string}>();
  const { data: resourcesData, error } = useDailyLoadings(asset || "");
  const [ selectedResources, setSelectedResources ] = useState<Resource[]>([]);
  const [ dates, setDates ] = useState<{from: Date, to: Date}>({
    from: new Date("2025-01-13"), //new Date(),
    to: new Date(new Date("2025-01-13").getTime() + 31 * 24 * 3600 * 1000)
  });

  const hasInitialized =useRef(false);

  useEffect(() => {
    if (resourcesData?.resources && !hasInitialized.current) {
      const allResources = Object.keys(resourcesData.resources);
      setSelectedResources(allResources);
      hasInitialized.current = true;
    }
  }, [resourcesData]);

  if (!asset) return (
    <div className="flex items-center">
      <div className="border p-6">
        <p>Asset not found</p>
      </div>
    </div>
  );
  if (!resourcesData || error) return (
    <div className="flex items-center">
      <div className="border p-6">
        <p>Could not load resources</p>
        {error && <p>Error: {error.message}</p>}
      </div>
    </div>
  );


  const resources = Object.keys(resourcesData?.resources);

  const toggleResource = (resource: Resource) => {
    setSelectedResources(prev =>
      prev.includes(resource) ? prev.filter(res => res != resource) : [...prev, resource]
  )
  }

  const toggleAllResources = () => {
    setSelectedResources(resources);
  }
  const clearResources = () => {
    setSelectedResources([]);
  }
  
  return (
    <>
    <SiteHeader header="Available Resources" />
      <Card className="h-full flex flex-col">
        <CardContent className="flex flex-1 min-h-0">
          <div className="flex-1 min-w-0 min-h-0">
            <ResourceLoadings resourcesData={resourcesData} selectedResources={selectedResources} dates={dates}/>
          </div>
          <div className="w-60 flex flex-col gap-2 p-2">
            <DatePicker dates={dates} setDates={setDates}/>
            <ResourceSidebar
              resources={resources}
              selectedResources={selectedResources}
              onToggle={toggleResource}
              toggleAll={toggleAllResources}
              onClear={clearResources}
            />
          </div>
        </CardContent>
      </Card>
    </>
  )
}

function DatePicker({dates, setDates}: {dates: {from: Date, to: Date}, setDates: (dates: {from: Date, to: Date}) => void}) {
  const [ open, setOpen ] = useState(false);
  
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button variant="outline" id="date" className="w-60 justify-between font-normal">
          {dates.from.toLocaleDateString()} - {dates.to.toLocaleDateString()}
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-auto overflow-hidden p-0" align="start">
        <Calendar mode="range" selected={dates} captionLayout="dropdown" onSelect={(range) => {
          if (range?.from && range?.to)
          setDates({from: range.from, to: range.to});
        }} />
      </PopoverContent>
    </Popover>
  )
}

function ResourceLoadings({ resourcesData, selectedResources, dates}: {
  resourcesData: DailyLoadingDto,
  selectedResources: Resource[],
  dates: {from: Date, to: Date}
}) {

  if (!resourcesData || selectedResources.length === 0) {
    return (
      <div className="flex flex-1 h-full items-center justify-center">
        <p className="text-muted-foreground">Select resources to view data</p>
      </div>
    )
  }

  const data = resourcesData.resources;
  const filteredData = selectedResources.reduce((acc, resource) => {
    if (data[resource]) {
      acc[resource] = data[resource]
    }
      return acc;
    
  }, {} as Record<string, DailyLoad[]>)

  
  const allDays = Object.values(data)[0]?.map(load => load.day).filter(day => {
    let dayAsDate  = new Date(day).setHours(0,0,0,0); // naivedate
    let fromDate = new Date(dates.from).setHours(0,0,0,0); // naivedate
    let toDate = new Date(dates.to).setHours(0,0,0,0); // naivedate
    return dayAsDate >= fromDate && dayAsDate <= toDate;
    }) || [];

  return (
    <div className="h-full w-full overflow-auto border rounded">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead key="Date" className="sticky left-0 bg-background z-10">Date</TableHead>
            {Object.keys(filteredData).map((resourceName) => (
              <TableHead className="sticky top-0 bg-background z-20" key={resourceName}>{resourceName}</TableHead>
            ))}
          </TableRow>
        </TableHeader>
        <TableBody>
          {allDays.map(day => (
            <TableRow key={day}>
              <TableCell className="sticky left-0 bg-background z-10 whitespace-nowrap">{day}</TableCell>
              {Object.entries(filteredData).map(([resourceName, dailyLoads]) => {
                const dayData = dailyLoads.find(load => load.day === day);
                return <TableCell className="whitespace-nowrap" key={resourceName}>{dayData?.work || 0}</TableCell>
              })}
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  )
}


export type Resource = string;

export function ResourceSidebar({ resources, selectedResources, onToggle, toggleAll, onClear}: {
  resources: Resource[],
  selectedResources: Resource[],
  onToggle: (value: Resource) => void,
  toggleAll: () => void,
  onClear: () => void,
}) {
  return (
    <Card className="h-full">
      <CardHeader>
        <CardTitle>Select Resources</CardTitle>
        <div className="flex gap-2 justify-between">
          <Button variant="outline" className="flex-1" size="sm" disabled={selectedResources.length === 0 ? true : false} onClick={onClear}>Clear</Button>
          <Button variant="outline" className="flex-1" size="sm" disabled={selectedResources.length !== resources.length ? false : true} onClick={toggleAll}>All</Button>
        </div>
      </CardHeader>
      <CardContent className="flex-1 flex flex-col min-h-0">
        <ScrollArea className="flex-1">
          <div className="flex flex-col gap-1">
            {resources.map(res => (
              <div
                role="button"
                key={res}
                className={`w-full flex justify-between p-2 rounded cursor-pointer hover:bg-gray-100 ${
                  selectedResources.includes(res) ? 'bg-blue-100' : ''
                }`}
                onClick={() => onToggle(res)}
              >
                <span>{res}</span>
              </div>
            ))}

          </div>
        </ScrollArea>
      </CardContent>
    </Card>
  )
}

