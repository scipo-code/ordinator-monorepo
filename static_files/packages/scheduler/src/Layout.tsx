import { Outlet } from "react-router-dom";
import { SidebarProvider} from "./components/ui/sidebar";
import { AppSidebar } from "@/components/app-sidebar";

export default function Layout() {
  return (
    <SidebarProvider>
      <AppSidebar />
      <main className="flex-1 flex flex-col min-h-0 overflow-hidden">
        <Outlet />          {/* ← WorkOrders or Resources appears here */}
      </main>
    </SidebarProvider>
  );
}
        // <SidebarTrigger className="px-6" />
