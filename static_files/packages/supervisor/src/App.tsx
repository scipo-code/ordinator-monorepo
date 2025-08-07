import { HashRouter, Route, Routes } from "react-router-dom";
// import ResourceView from "./pages/ResourceView"; 
import "./App.css";
import Layout from "./Layout";
// import PlanningView from "./pages/PlanningView";
import { AllCommunityModule, ModuleRegistry } from 'ag-grid-community'; 
import ScheduleView from "./pages/ScheduleView";
import MainTable from "./pages/MainTable";

// Register all Community features
ModuleRegistry.registerModules([AllCommunityModule]);

function App() {
  return (
      <HashRouter>
          <Routes>
            <Route path="/" element={<Layout />}>
              <Route path="/:asset/schedule" element={<ScheduleView />} />
              <Route path="/:asset/workschedule" element={<MainTable/>} />
            </Route>
          </Routes>
      </HashRouter>
  );
}
export default App;
              // <Route path="planning" element={<PlanningView />} />
              // <Route path="resources" element={<ResourceView />} />
