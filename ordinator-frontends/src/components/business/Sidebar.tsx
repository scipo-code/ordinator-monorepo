import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import 'react-day-picker/dist/style.css';
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { SupervisorAllAvailableTechnicians } from "@/types/dto/SupervisorAllAvailableTechnicians";

export function ResourceSidebar({ resources, selectedResources, onToggle, toggleAll, onClear, technicians}: {
  resources: string[],
  selectedResources: string[],
  onToggle: (value: string) => void,
  toggleAll: () => void,
  onClear: () => void,
  technicians: SupervisorAllAvailableTechnicians,
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Select Resources</CardTitle>
        <div className="flex gap-2">
          <Button variant="outline" className="flex-1" size="sm" disabled={selectedResources.length === 0 ? true : false} onClick={onClear}>Clear</Button>
          <Button variant="outline" className="flex-1" size="sm" disabled={selectedResources.length === resources.length} onClick={toggleAll}>All</Button>
        </div>
      </CardHeader>
      <CardContent className="flex flex-col gap-2 h-[420px]">
        <ScrollArea className="flex-1">
        {resources.map(res => (
          <div
            role="button"
            key={res}
            className={`flex justify-between p-2 rounded cursor-pointer hover:bg-gray-100 ${
              selectedResources.includes(res) ? 'bg-blue-100' : ''
            }`}
            onClick={() => onToggle(res)}
          >
            <span>{res}</span>
            <Badge>{technicians.all_technicians.filter(tech => tech.resources.includes(res)).length}</Badge>
          </div>
        ))}
        </ScrollArea>
      </CardContent>
    </Card>
  )
}
