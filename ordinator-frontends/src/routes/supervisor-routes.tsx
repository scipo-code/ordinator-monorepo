import { RoutingLayout } from "@/components/layout/Layout";
import ResourceView from "@/pages/supervisor/resources/ResourceView";
import ScheduleView from "@/pages/supervisor/ScheduleView";
import SupervisorDashboard from "@/pages/supervisor/SupervisorDashboard";
import { Route, Routes } from "react-router-dom";




export function SupervisorRoutes() {
  return (
      <Routes>
        <Route path=":asset" element={<RoutingLayout variant="supervisor" />}>
          <Route index element={<SupervisorDashboard />} />
          <Route path="schedule" element={<SupervisorDashboard />} />
          <Route path="frozen_plan" element={<ScheduleView />} />
          <Route path="resources" element={<ResourceView/>} />
        </Route>
      </Routes>
        
  )
}
        // <Route path=":asset/" element={<Navigate to="schedule" relative="path" />} />

