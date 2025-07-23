import { useParams } from "react-router-dom"



export default function WorkorderOverview() {
  const { workorder } = useParams<{ workorder: string }>();


  
  return <h1>Hello {workorder}</h1>
}

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

