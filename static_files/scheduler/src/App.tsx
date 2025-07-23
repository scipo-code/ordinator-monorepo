import { HashRouter, Route, Routes } from "react-router-dom";
// import AssetDashboard from "./pages/AssetDashboard"; 
import Scheduler from "./pages/dashboard/Scheduler";
import "./App.css";
import Layout from "./Layout";
import { AllCommunityModule, ModuleRegistry } from 'ag-grid-community'; 

// Register all Community features
ModuleRegistry.registerModules([AllCommunityModule]);

function App() {
  return (
    <HashRouter>
        <Routes>
          <Route path="/" element={<Layout />}>
            <Route path="/dashboard/:asset" element={<Scheduler />} />
          </Route>
        </Routes>
    </HashRouter>
  );
}

            // <Route path="/dashboard/:asset/resources" element={<ResourceOverview />} />
export default App;
