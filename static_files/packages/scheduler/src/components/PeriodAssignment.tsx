import { useState } from "react";
import { PeriodDto } from "@scipo-code/shared";
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


export function PeriodAssignment({
  periods,
  suggestedPeriod,
  workOrderNumber,
  asset,
  onAssign,
  isAssigning = false,
}: PeriodAssignmentProps) {
  const [open, setOpen ] = useState(false);
  const [selectedPeriod, setSelectedPeriod] = useState<PeriodDto | null>(() => {
    if (suggestedPeriod) {
      return periods.find(p => p == suggestedPeriod) || null;
    }
    return null;
  })


  const handleAssign = () => {
    if (selectedPeriod) {
      onAssign(workOrderNumber, asset,selectedPeriod as PeriodDto)
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
            {selectedPeriod ? selectedPeriod : "Select period..."}
            <ChevronsUpDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-[300px] p-0">
         <Command>
           <CommandInput placeholder="Search periods..." />
           <CommandEmpty>No period found.</CommandEmpty>
             <CommandList>
               <CommandGroup>
                   {periods.map((period) => (
                     <CommandItem
                       key={period}
                       value={period}
                       onSelect={() => {
                         setSelectedPeriod(period as PeriodDto);
                         setOpen(false);
                       }}
                     >
                       <Check
                         className={cn(
                           "mr-2 h-4 w-4",
                           selectedPeriod === period
                           ? "opacity-100"
                           : "opacity-0"
                         )}
                        />
                        <span className="font-medium">{period}</span>
                      </CommandItem>
                   ))}
               </CommandGroup>
            </CommandList>
           </Command>
         </PopoverContent>
       </Popover>
       <Button
         onClick={handleAssign}
         disabled={!selectedPeriod || isAssigning}
         variant="default"
         >
         {isAssigning ? "Assigning..." : "Assign Period"}
       </Button>
     </div>
  );
}
