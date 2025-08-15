import { HashRouter, Navigate, Route, Routes } from "react-router-dom";
// import ResourceView from "./pages/ResourceView"; 
import "./App.css";
import Layout from "./Layout";
// import PlanningView from "./pages/PlanningView";
import { AllCommunityModule, ModuleRegistry } from 'ag-grid-community'; 
import ScheduleView from "./pages/ScheduleView";
import MainTable from "./pages/MainTable";
import ResourceView from "./pages/ResourceView";

// Register all Community features
ModuleRegistry.registerModules([AllCommunityModule]);

function App() {
  return (
      <HashRouter>
          <Routes>
            <Route path="/" element={<Layout />}>
              <Route path="/:asset/" element={<Navigate to="schedule" relative="path" />} />
              <Route path="/:asset/schedule" element={<MainTable />} />
              <Route path="/:asset/frozen_plan" element={<ScheduleView />} />
              <Route path="/:asset/resources" element={<ResourceView/>} />
            </Route>
          </Routes>
      </HashRouter>
  );
}
export default App;
