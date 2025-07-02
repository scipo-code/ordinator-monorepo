import { HashRouter, Route, Routes, useParams } from "react-router-dom";
import AssetDashboard from "./pages/AssetDashboard"; 
import Scheduler from "./pages/dashboard/Scheduler";
import WorkorderOverview from "./pages/dashboard/WorkorderOverview";
import ResourceOverview from "./pages/dashboard/ResourceOverview";
import { ResourceChart } from "./pages/dashboard/ResourceChart";
import "./App.css";
import Layout from "./Layout";
import { AllCommunityModule, ModuleRegistry } from 'ag-grid-community'; 

// Register all Community features
ModuleRegistry.registerModules([AllCommunityModule]);

function Resources() {
  const { asset } = useParams<{ asset: string }>();
  
  if (!asset) {
    throw new Error("Asset is required");
  }

  return (
    <div className="flex flex-col gap-4">
      <ResourceChart asset={asset} />
      <ResourceOverview asset={asset} />
    </div>
  );
}

function App() {
  return (
    <HashRouter>
        <Routes>
          <Route path="/" element={<Layout />}>
            <Route path="/dashboard/:asset" element={<Scheduler />} />
            <Route path="/dashboard/:asset/workorders" element={<WorkorderOverview />} />
            <Route path="/dashboard/:asset/resources" element={<Resources />} />
          </Route>
        </Routes>
    </HashRouter>
  );
}

export default App;
