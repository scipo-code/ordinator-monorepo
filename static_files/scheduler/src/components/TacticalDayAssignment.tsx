import { useState } from "react";
import { PeriodDto } from "../../../../crates/ordinator-contracts/bindings/PeriodDto";
import { Popover, PopoverContent, PopoverTrigger } from "./ui/popover";
import { Button } from "./ui/button";
import { Check, ChevronsUpDown } from "lucide-react";
import { Command, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList } from "./ui/command";
import { cn } from "@/lib/utils";






interface PeriodAssignmentProps {
  periods: PeriodDto[],
  suggestedPeriod?: PeriodDto,
  workOrderNumber: string,
  asset: string,
  onAssign: (workOrderNumber: string, asset: string, period: PeriodDto) => void;
  isAssigning?: boolean,
};

export function TacticalDayAssignment({
  // periods,
  // suggestedPeriod,
  // workOrderNumber,
  // asset,
  // onAssign,
  // isAssigning = false,
}: PeriodAssignmentProps) {
  const [open, setOpen ] = useState(false);
  // const [selectedDay, setSelectedDay] = useState<PeriodDto | null>(() => {
  //   if (selectedDay) {
  //     return periods.find(p => p == suggestedPeriod) || null;
  //   }
  //   return null;
  // })

  const selectedDay = "todo";


  const handleAssign = () => {
    if (selectedDay) {
      console.log("Assign to date");
    }
  };

  return (
    <div className="flex items-center gap-2">
      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger asChild>
          <Button
            variant="outline"
            role="combobox"
            aria-expanded={open}
            className="w-[300px] justify-between"
          >
            {selectedDay ? selectedDay : "Select date..."}
            <ChevronsUpDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-[300px] p-0">
         <Command>
           <CommandInput placeholder="Search periods..." />
           <CommandEmpty>No day found.</CommandEmpty>
             <CommandList>
               <CommandGroup>
                     <CommandItem
                       key="todo"
                       value="todo"
                     >
                       <Check
                         className={cn(
                           "opacity-0 mr-2 h-4 w-4",
                         )}
                        />
                        <span className="font-medium">Date</span>
                      </CommandItem>
               </CommandGroup>
            </CommandList>
           </Command>
         </PopoverContent>
       </Popover>
       <Button
         variant="default"
         onClick={handleAssign}
         >
         Assign Date
       </Button>
     </div>
  );
}
