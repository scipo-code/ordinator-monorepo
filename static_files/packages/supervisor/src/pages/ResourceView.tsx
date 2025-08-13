import { Button } from "@/components/ui/button";
import { Calendar } from "@/components/ui/calendar";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { addTechnician, NaiveDateDto, SupervisorAllAvailableTechnicians, TechnicianAvailability, useDays, useResources, useTechnicianAvailability } from "@scipo-code/shared";
import { format } from "date-fns";
import { ChevronsUpDown, ChevronDownIcon, ChevronLeft, ChevronRight, X, Check } from "lucide-react";
import 'react-day-picker/dist/style.css';
import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { Badge } from "@/components/ui/badge";
import { Label } from "@/components/ui/label";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Input } from "@/components/ui/input";
import { toast } from "sonner";
import { cn } from "@/lib/utils"
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command"
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from "@/components/ui/form";
import * as z from "zod";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { useQueryClient } from "@tanstack/react-query";
import { ScrollArea } from "@/components/ui/scroll-area";


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
    <div className="p-4 flex flex-col flex-1 min-h-0 border ">
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

      <div className="flex flex-1 min-h-0 gap-2 overflow-hidden">
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
        <div className="w-60">
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
        <Button variant="outline" size="sm" disabled={selectedWorkCenters.length === 0 ? true : false} onClick={onClear}>Clear Selection</Button>
      </CardHeader>
      <CardContent className="flex flex-col gap-2 h-[420px]">
        <ScrollArea className="flex-1">
        {workCenters.map(wc => (
          <div
            role="button"
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
        </ScrollArea>
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
    <Card className="flex-1 min-h-0 flex flex-col h-full">
      <CardHeader className="shrink-0">
        <div className="flex items-center justify-between">
        <CardTitle className="text-lg">
          Weekly Availability
         </CardTitle>
         <AddTechnicianDialog/>
        </div>
      </CardHeader>
      <CardContent className="flex-1 min-h-0 overflow-auto p-4">
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


const formTechnicianSchema = z.object({
  id: z.string().min(1, "Technician ID is required"),
  start: z.string().min(1, "Start date is required"),
  finish: z.string().min(1, "Finish date is required"),
  resources_string: z.array(z.string()).min(1, "At least one resource is required"),
}).refine((data) => {
  const startDate = new Date(data.start);
  const finishDate = new Date(data.finish);
  return finishDate > startDate
}, {
    message: "Finish date must be later than start date",
    path: ["finish"]
  })

function AddTechnicianDialog() {
  const { asset } = useParams();
  const { data: resources } = useResources();
  const [ _selectedResources, setSelectedResources ] = useState<string[]>([]);
  const [open, setOpen] = useState(false);
  const queryClient = useQueryClient();


  const form = useForm<z.infer<typeof formTechnicianSchema>>({
    resolver: zodResolver(formTechnicianSchema),
    defaultValues: {
      id: "",
      start: "",
      finish: "",
      resources_string: [],
    }
  })


  const onSubmit = async (values: z.infer<typeof formTechnicianSchema>) => {
    try {
      await addTechnician(asset!, "main", values);
      form.reset();
      toast(`Technician, ${values.id}, has been added.`);
      setSelectedResources([])
      setOpen(false);

      queryClient.invalidateQueries({queryKey: ["technicianAvailability"]});
    } catch (error) {
      toast(`Failed to add technician: ${error}`);
    }
  };

  

  if (resources?.length === 0 || !resources) {
    return (
      <p className="text-red-300">Error fetching Resources from server</p>
    )
  }

  
  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button variant="outline" size="sm">Add Technician</Button>
      </DialogTrigger>

      <DialogContent>
        <DialogHeader>
          <DialogTitle>Add Technician</DialogTitle>
        </DialogHeader>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <FormField
              control={form.control}
              name="id"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Technician ID</FormLabel>
                  <FormControl>
                    <Input {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="start"
              render={({ field }) => (
                <FormItem>
                  <FormControl>
                    <Calendar24
                      label="Start Date"
                      value="field.value"
                      onChange={field.onChange}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="finish"
              render={({ field }) => (
                <FormItem>
                  <FormControl>
                    <Calendar24
                      label="Finish Date"
                      value="field.value"
                      onChange={field.onChange}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="resources_string"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Resources</FormLabel>
                  <FormControl>
                    <div className="gap-2">
                      <ResourcesCombobox
                        resources={resources}
                        selectedResources={field.value}
                        onSelect={(resource) => {
                          const newResources = field.value.includes(resource)
                            ? field.value.filter(r => r !== resource)
                            : [...field.value, resource];
                          field.onChange(newResources);
                        }}
                      />
                      <AddedResources
                        selectedResources={field.value}
                        onToggle={(resource) => {
                          const newResources = field.value.filter(r => r !== resource);
                          field.onChange(newResources);
                        }}
                      />
                    </div>
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <Button type="submit">Add technician</Button>
          </form>
        </Form>

      </DialogContent>
    </Dialog>
  )
}

function ResourcesCombobox({resources, selectedResources, onSelect}: {resources: string[], selectedResources: string[], onSelect: (value: string) => void}) {
  const [open, setOpen] = useState(false)

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          role="combobox"
          aria-expanded={open}
          className="w-[200px] justify-between"
        >
          Select Resources
          <ChevronsUpDown className="opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[200px] p-0">
        <Command>
          <CommandInput placeholder="Search framework..." className="h-9" />
          <CommandList>
            <CommandEmpty>No resource found.</CommandEmpty>
            <CommandGroup>
              {resources.map((resource) => (
                <CommandItem
                  key={resource}
                  value={resource}
                  onSelect={(currentValue) => {
                    onSelect(currentValue)
                    setOpen(false)
                  }}
                >
                  {resource}
                  <Check
                    className={cn(
                      "ml-auto",
                      selectedResources.includes(resource) ? "opacity-100" : "opacity-0"
                    )}
                  />
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  )
}

function AddedResources({selectedResources, onToggle}: {selectedResources: string[], onToggle: (value: string) => void}) {
  return (
    <div className="p-2 flex flex-wrap gap-2 overflow-auto rounded-md border max-h-[200px] min-h-[40px]">
      {selectedResources.map(r => (
        <div key={r} className="gap-2">
          <Badge variant="secondary" className="gap-2 mb-1" key={r} onClick={() => onToggle(r)}>
            <p>{r}</p>
            <Button variant="ghost" size="sm">
              <X className="h-3 w-3" />
            </Button>
          </Badge>
        </div>
      ))}
    </div>
  )
}




function Calendar24({label, value, onChange}: {label: string, value: string, onChange: (value: string) => void}) {
  const [open, setOpen] = useState(false)
  const [date, setDate] = useState<Date | undefined>(
    value ? new Date(value) : undefined
  )
  const [time, setTime] = useState(value ? value.split('T')[1]?.split('Z')[0] || "07:00:00" : "07:00:00");

  const updateDateTime = (newDate?: Date, newTime?: string) => {
    if (newDate && newTime) {
      // Format: YYYY-MM-DDTHH:MM:SSZ Example: 2025-04-08T07:00:00Z
      const isoString = `${newDate.toISOString().split('T')[0]}T${newTime}Z`;
      onChange(isoString);
    }
  }

  return (
      <div className="flex gap-4">
        <div className="flex flex-col gap-3">
          <Label htmlFor="date-picker" className="px-1">
            {label}
          </Label>
          <Popover open={open} onOpenChange={setOpen}>
            <PopoverTrigger asChild>
              <Button
                variant="outline"
                id="date-picker"
                className="w-32 justify-between font-normal"
              >
                {date ? date.toLocaleDateString() : "Select date"}
                <ChevronDownIcon />
              </Button>
            </PopoverTrigger>
            <PopoverContent className="w-auto overflow-hidden p-0" align="start">
              <Calendar
                mode="single"
                selected={date}
                captionLayout="dropdown"
                onSelect={(newDate) => {
                  setDate(newDate)
                  updateDateTime(newDate, time)
                  setOpen(false)
                }}
              />
            </PopoverContent>
          </Popover>
        </div>
        <div className="flex flex-col gap-3">
          <Label htmlFor="time-picker" className="px-1">
            Time
          </Label>
          <Input
            type="time"
            value={time}
            step="1"
            onChange={(e) => {
              setTime(e.target.value)
              updateDateTime(date, e.target.value)
            }}
            className="bg-background appearance-none [&::-webkit-calendar-picker-indicator]:hidden [&::-webkit-calendar-picker-indicator]:appearance-none"
          />
        </div>
      </div>
  )
}
