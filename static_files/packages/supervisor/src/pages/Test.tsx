import { Button } from "@/components/ui/button"
import { Calendar } from "@/components/ui/calendar"
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover"
import { CalendarIcon } from "lucide-react"
import 'react-day-picker/dist/style.css';
// Commentar
// Commentar
// Commentar
function CalendarPopover() {


  
  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
        >
          <CalendarIcon/>
        </Button>
      </PopoverTrigger>
      <PopoverContent>
        <Calendar mode="range" numberOfMonths={1}/>
      </PopoverContent>
    </Popover>
        
  )
}

function CalendarPopover() {


  
  return (
        <Calendar mode="range" numberOfMonths={1}/>
  )
}


export default function Test() {
  return <CalendarPopover /> 
}
