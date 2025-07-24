import { fetchWorkorderInfo } from "@/api/workorders";
import { useParams } from "react-router-dom"
import { useQuery } from "@tanstack/react-query";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { WorkOrderInfoWithScheduling } from "../../../../../crates/ordinator-contracts/bindings/WorkOrderInfoWithScheduling";
import { Badge } from "@/components/ui/badge";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { PeriodStatusBadge } from "@/components/ui/period_status_badge";


function WorkorderCard({ wo }: { wo: WorkOrderInfoWithScheduling  }) {
  const renderFlag = (label: string, active: boolean) => 
    <Badge className="mr-1" variant={active ? "default" : "secondary"}>{label}</Badge>;
  

  return (
    <Card className="w-full max-w-4xl p-4">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          Workorder: {wo.work_order_number}
          {renderFlag("SCH", wo.sch)}
          {renderFlag("AWSC", wo.awsc)}
          {renderFlag("Vendor", wo.vendor)}
        </CardTitle>
        <CardDescription>
          <div className="flex flex-col space-y-0.5">
            <span>Main Workcenter: {wo.main_work_center}</span>
            <span>Functional Location: {wo.functional_location}</span>
          </div>
        </CardDescription>
      </CardHeader>
      <CardContent>
        <h3 className="mb-2 font-semibold">Operations</h3>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Activity</TableHead>
              <TableHead>Work Remaining</TableHead>
              <TableHead>Work Center</TableHead>
              <TableHead>Number of People</TableHead>
              <TableHead>Unloading Point Period</TableHead>
              <TableHead>Unloading Point Code</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
              {wo.operations.map((a) => (
                <TableRow key={a.activity}>
                  <TableCell className="text-right">{a.activity}</TableCell>
                  <TableCell className="text-right">{a.work_remaining}</TableCell>
                  <TableCell>{a.work_center}</TableCell>
                  <TableCell className="text-right">{a.number_of_people}</TableCell>
                  <TableCell>{a.unloading_point_period}</TableCell>
                  <TableCell>{a.unloading_point_string}</TableCell>
                </TableRow>
              ))}
          </TableBody>
        </Table>
      <br />
      
      <h3 className="font-semibold">Schedule to period</h3>
      <PeriodStatusBadge status={wo.period_status} />   <span>{wo.suggested_scheduled_period}</span>
      </CardContent>
    </Card>
  )
  
}


export default function WorkorderOverview() {
  const { asset, workorder } = useParams<{ asset: string, workorder: string }>();

  const {
    data: woInfo, isLoading, error
  } = useQuery({
    queryKey: ['workorderInfo', {workorder, asset}],
    queryFn: () => fetchWorkorderInfo(asset!, workorder!),
    enabled: !!workorder && !!asset,
  })

  console.log(woInfo?.work_order_number);

  if (!workorder || !asset) return null;
  if (isLoading) {
    return (
      <span>Loading...</span>
    )
  }

  if (error) {
    return (
      <span className="color-red-600">{(error as Error).message}</span>
    )
    
  }
  
  return (
    <WorkorderCard wo={woInfo!} />
  )}

  // const assignMutation = useAssignWorkorderToPeriod();
  // const handleAssignPeriod = useCallback(
  //   (row: SchedulingData, period?: PeriodDto) => {
  //     const chosenPeriod = period ?? (row.suggested_scheduled_period as PeriodDto | undefined);

  //     if (!chosenPeriod) return;
      
  //     assignMutation.mutate({
  //       asset,
  //       workorder: row.work_order_number,
  //       period: chosenPeriod,
  //     })
  //   },
  //   [asset, assignMutation],
  // )


  // const {
  //   data: periods = [],
  //   // isLoading: periodsLoading,
  //   // isError: periodsError,
  // } = useQuery({
  //   queryKey: ["periods"],
  //   queryFn: fetchPeriods,
  //   staleTime: Infinity,
  // });

