import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { DailyLoad, useDailyLoadings } from "@scipo-code/shared";
import { DailyLoadingDto } from "@scipo-code/shared";
import { useState } from "react";
import { useParams } from "react-router-dom";

import 'react-day-picker/dist/style.css';
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Button } from "@/components/ui/button";
import { Calendar } from "@/components/ui/calendar";
import { ScrollArea } from "@/components/ui/scroll-area";



export default function ResourceView() {
  const { asset } = useParams<{asset: string}>();
  const { data: resourcesData, error } = useDailyLoadings(asset || "");
  const [ selectedResources, setSelectedResources ] = useState<Resource[]>([]);
  const [ dates, setDates ] = useState<{from: Date, to: Date}>({
    from: new Date(),
    to: new Date(Date.now() + 14 * 24 * 3600 * 1000)
  });


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
    <div className="w-full h-full p-4">
      <Card className="h-full">
        <CardHeader>
          <div className="flex items-start justify-between">
            <CardTitle>{asset?.toUpperCase()}</CardTitle>
            <DatePicker dates={dates} setDates={setDates}/>
          </div>
        </CardHeader>
        <CardContent className="h-full overflow-hidden">
          <div className="flex justify-between items-start gap-4 h-full">
            <ResourceLoadings resourcesData={resourcesData} selectedResources={selectedResources} dates={dates}/>
            <div className="h-full w-60">
              <ResourceSidebar
                resources={resources}
                selectedResources={selectedResources}
                onToggle={toggleResource}
                toggleAll={toggleAllResources}
                onClear={clearResources}
              />
            </div>
          </div>
        </CardContent>
      </Card>
    </div>

  )
}

function DatePicker({dates, setDates}: {dates: {from: Date, to: Date}, setDates: (dates: {from: Date, to: Date}) => void}) {
  const [ open, setOpen ] = useState(false);
  
  return (
    <div className="flex flex-col gap-3">
      <Label htmlFor="date" className="px-1">
        Pick Dates
      </Label>
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
    </div>
  )
}

function ResourceLoadings({ resourcesData, selectedResources, dates}: {
  resourcesData: DailyLoadingDto,
  selectedResources: Resource[],
  dates: {from: Date, to: Date}
}) {

  if (!resourcesData || selectedResources.length === 0) {
    return (
      <div className="flex-1 min-h-0 h-full flex items-center justify-center">
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
    <div className="flex-1 min-h-0 h-full">
      <ScrollArea className="flex-1 h-full overflow-y-clip">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead key="Date" className="sticky left-0 bg-background">Date</TableHead>
              {Object.keys(filteredData).map((resourceName) => (
                <TableHead key={resourceName}>{resourceName}</TableHead>
              ))}
            </TableRow>
          </TableHeader>
          <TableBody>
            {allDays.map(day => (
              <TableRow key={day}>
                <TableCell className="sticky left-0 bg-background">{day}</TableCell>
                {Object.entries(filteredData).map(([resourceName, dailyLoads]) => {
                  const dayData = dailyLoads.find(load => load.day === day);
                  return <TableCell key={resourceName}>{dayData?.work || 0}</TableCell>
                })}
              </TableRow>
            ))}

          </TableBody>


        </Table>
      </ScrollArea>
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
        <div className="flex gap-2">
          <Button variant="outline" className="flex-1" size="sm" disabled={selectedResources.length === 0 ? true : false} onClick={onClear}>Clear</Button>
          <Button variant="outline" className="flex-1" size="sm" disabled={selectedResources.length !== 0} onClick={toggleAll}>All</Button>
        </div>
      </CardHeader>
      <CardContent className="flex flex-col gap-2 h-full">
        <ScrollArea className="flex-1">
          <div className="flex flex-col gap-1">
            {resources.map(res => (
              <div
                role="button"
                key={res}
                className={`flex justify-between p-2 gap-2 rounded cursor-pointer hover:bg-gray-100 ${
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

