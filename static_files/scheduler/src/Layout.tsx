import { Outlet } from "react-router-dom";
import { SidebarProvider, SidebarTrigger } from "./components/ui/sidebar";
import { AppSidebar } from "@/components/app-sidebar";

export default function Layout() {
  return (
    <SidebarProvider>
      <AppSidebar />
      <main className="flex-1">
        <SidebarTrigger className="px-6" />
        <Outlet />          {/* ← WorkOrders or Resources appears here */}
      </main>
    </SidebarProvider>
  );
}
