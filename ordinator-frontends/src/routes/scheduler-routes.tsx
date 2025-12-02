import { RoutingLayout } from "@/components/layout/Layout";
import SchedulerDashboard from "@/pages/scheduler/dashboard/SchedulerDashboard";
import WorkOrderOverview from "@/pages/scheduler/dashboard/WorkOrderOverview";
import ResourceView from "@/pages/scheduler/ResourceView";
import { Navigate, Route, Routes } from "react-router-dom";




export function SchedulerRoutes() {
  return (
      <Routes>
        <Route path=":asset" element={<RoutingLayout operator="scheduler" />}>
          <Route index element={<Navigate to="dashboard" relative="path" />} />
          <Route path="dashboard" element={<SchedulerDashboard />} />
          <Route path="dashboard/:workorder" element={<WorkOrderOverview/>} />
          <Route path="resources" element={<ResourceView />} />
        </Route>
      </Routes>
        
  )
}
        // <Route path=":asset/" element={<Navigate to="schedule" relative="path" />} />
