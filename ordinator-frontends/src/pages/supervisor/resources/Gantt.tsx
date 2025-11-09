import type { NaiveDateDto } from "@/types/dto/NaiveDateDto";
import type { TechnicianAvailability } from "@/types/dto/TechnicianAvailability";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { format } from "date-fns";
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from "@/components/ui/form";
import * as z from "zod";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { useQueryClient } from "@tanstack/react-query";
import { useParams } from "react-router-dom";
import { useResources } from "@/hooks";
import { useState } from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { Badge } from "@/components/ui/badge";
import { Check, ChevronDownIcon, ChevronsUpDown, X } from "lucide-react";
import { Label } from "@/components/ui/label";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Calendar } from "@/components/ui/calendar";
import { Command, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList } from "@/components/ui/command";
import { cn } from "@/lib/utils";
import { addTechnician } from "@/api/supervisor";

interface GanttViewProps {
  groupedTechnicians: Map<string, TechnicianAvailability[]>;
  weekDays: NaiveDateDto[];
}

export function GanttView({ groupedTechnicians, weekDays }: GanttViewProps) {
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


          {Array.from(groupedTechnicians.entries()).map(([resourceCombo, techs]) => (
            <div key={resourceCombo}>
              <div className="bg-blue-300 mb-2 text-center font-medium rounded">
                {resourceCombo} ({techs.length})
              </div>
            {techs.map(tech => (
              <TechnicianRow key={tech.id} technician={tech} weekDays={weekDays} />
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

export function TechnicianRow({ technician, weekDays }: TechnicianRowProps) {
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
                      <div className="mb-2">
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
                      </div>
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
          <CommandInput placeholder="Search Resource..." className="h-9" />
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
