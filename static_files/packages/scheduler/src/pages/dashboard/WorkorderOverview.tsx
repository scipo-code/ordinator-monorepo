import { assignWorkordertoPeriod, fetchWorkorderInfo } from "@/api/workorders";
import { useParams } from "react-router-dom"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { WorkOrderInfoWithSchedulingDto } from "@scipo-code/shared";
import { Badge } from "@/components/ui/badge";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { PeriodStatusBadge } from "@scipo-code/shared";
import { fetchPeriods } from "@scipo-code/shared";
import { PeriodDto } from "@scipo-code/shared";
import { PeriodAssignment } from "@/components/PeriodAssignment";
import { useCallback } from "react";
import { TacticalDayAssignment } from "@/components/TacticalDayAssignment";


function WorkorderCard({
  wo,
  periods,
  asset,
  onAssignPeriod,
  isAssigning
}: {
    wo: WorkOrderInfoWithSchedulingDto,
    periods: PeriodDto[],
    asset: string,
    onAssignPeriod: (workOrderNumber: string, asset: string, period: PeriodDto) => void;
    isAssigning: boolean,
  }) {
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
      
      <div className="space-y-4">
        <div className="flex items-center gap-2">
          <h3 className="font-semibold">Current Suggestion:</h3>
          <PeriodStatusBadge status={wo.period_status} />
          {wo.suggested_scheduled_period && (
            <span className="text-sm text-muted-foreground">
              Suggested: {wo.suggested_scheduled_period}
            </span>
          )}
        </div>
      
        <div>
          <h3 className="font-semibold mb-2">Schedule to period</h3>
          <PeriodAssignment
            periods={periods}
            suggestedPeriod={wo.suggested_scheduled_period}
            workOrderNumber={wo.work_order_number.toString()}
            asset={asset}
            onAssign={onAssignPeriod}
            isAssigning={isAssigning}
          />
        </div>
        <div>
          <h3 className="font-semibold mb-2">Schedule to date (TODO)</h3>
          <TacticalDayAssignment
            periods={periods}
            suggestedPeriod={wo.suggested_scheduled_period}
            workOrderNumber={wo.work_order_number.toString()}
            asset={asset}
            onAssign={onAssignPeriod}
            isAssigning={isAssigning}
          />
        </div>
      </div>
      </CardContent>
    </Card>
  )
  
}


export default function WorkorderOverview() {
  const { asset, workorder } = useParams<{ asset: string, workorder: string }>();
  const queryClient = useQueryClient();

  const {
    data: woInfo, isLoading, error
  } = useQuery({
    queryKey: ['workorderInfo', {workorder, asset}],
    queryFn: () => fetchWorkorderInfo(asset!, workorder!),
    enabled: !!workorder && !!asset,
  })


  const {
    data: periods = [], isLoading: isPeriodsLoading, error: errorPeriod
  } = useQuery({
    queryKey: ['periods'],
    queryFn: fetchPeriods,
    staleTime: Infinity,
  });

  const assignMutation = useMutation({
    mutationFn: ({asset, workorder, period}: {
      asset: string,
      workorder: string,
      period: PeriodDto,
    }) => assignWorkordertoPeriod(asset,workorder, period),

    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ['workorderInfo', {workorder, asset}]
      });
    },

    onError: (error: Error) => {
      console.error("Failed to assign period: ", error);
    },
  });


  const handleAssignPeriod = useCallback((
    workOrderNumber: string,
    assetParam: string,
    Period: PeriodDto,
  ) => {
      assignMutation.mutate({
        asset: assetParam,
        workorder: workOrderNumber,
        period: Period,
      });
    }, [assignMutation]);

  
  
  console.log('Periods Debug:', {
    periods,
    isArray: Array.isArray(periods),
    length: periods?.length,
    type: typeof periods,
    firstItem: periods?.[0],
    keys: periods?.[0] ? Object.keys(periods[0]) : 'no first item'
  });

  if (!workorder || !asset) return null;
  if (isLoading || isPeriodsLoading) {
    return (
      <span>Loading...</span>
    )
  }

  if (error || errorPeriod) {
    return (
      <span className="color-red-600">{(error as Error).message}</span>
    )
    
  }
  
  return (
    <div className="px-4">
      <WorkorderCard
         wo={woInfo!}
         periods={periods}
         asset={asset}
         onAssignPeriod={handleAssignPeriod}
         isAssigning={assignMutation.isPending}
      />

      {assignMutation.error && (
        <div className="max-w-4xl mt-4 p-4 bg-red-50 border border-red-200 rounded">
          <p className="text-red-800">
            Failed to assign period: {assignMutation.error.message}
          </p>
        </div>
      )}


      {assignMutation.isSuccess && (
        <div className="max-w-4xl mt-4 p-4 bg-green-50 border border-green-200 rounded">
          <p className="text-green-800">Period assigned successfully!</p>
        </div>
      )}
    </div>

  )}




