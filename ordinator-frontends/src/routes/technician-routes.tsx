import { RoutingLayout } from "@/components/layout/Layout";
import TechnicianDashboard from "@/pages/technician/dashboard/TechnicianDashboard";
import { Route, Routes } from "react-router-dom";


export function TechnicianRoutes() {
  return (
      <Routes>
        <Route path=":asset" element={<RoutingLayout operator="technician" />}>
          <Route index element={<TechnicianDashboard />} />
          <Route path="dashboard" element={<TechnicianDashboard />} />
        </Route>
      </Routes>
        
  )
}
        // <Route path=":asset/" element={<Navigate to="schedule" relative="path" />} />


