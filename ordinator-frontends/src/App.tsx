import { HashRouter, Route, Routes } from "react-router-dom";
import "./App.css";
import { AllCommunityModule, ModuleRegistry } from 'ag-grid-community'; 
import { SchedulerRoutes } from "./routes/scheduler-routes";
import { SupervisorRoutes } from "./routes/supervisor-routes";
import { TechnicianRoutes } from "./routes/technician-routes";
import LoginPage from "./pages/auth/Login";

// Register all Community features
ModuleRegistry.registerModules([AllCommunityModule]);

function App() {
  return (
      <HashRouter>
          <Routes>
            <Route path="/" element={<LoginPage />} />
            <Route path="/login" element={<LoginPage />} />
            <Route path="/scheduler/*" element={<SchedulerRoutes />} />
            <Route path="/supervisor/*" element={<SupervisorRoutes />} />
            <Route path="/technician/*" element={<TechnicianRoutes />} />
          </Routes>
      </HashRouter>
  );
}
            // <Route path="/login" element={<LoginPage />} />
            // <Route path="/technician" element={<TechnicianRoutes />} />
            // <Route path="/" element={<RoleBasedRedirect />} />
export default App;
