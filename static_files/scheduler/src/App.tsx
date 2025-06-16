import { HashRouter, Route, Routes, useParams } from "react-router-dom";
import AssetDashboard from "./pages/AssetDashboard"; 
import WorkOrders from "./pages/dashboard/WorkOrders";
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
    <HashRouter basename="/">
      <Layout>
        <Routes>
          <Route path="/dashboard/:asset" element={<WorkOrders />} />
          <Route path="/dashboard/:asset/resources" element={<Resources />} />
        </Routes>
      </Layout>
    </HashRouter>
  );
}

export default App;
