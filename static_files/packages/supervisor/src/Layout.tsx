import { Outlet } from "react-router-dom";
import { SidebarProvider } from "./components/ui/sidebar";
import { AppSidebar } from "@/components/app-sidebar";
import { Toaster } from "@/components/ui/sonner";

export default function Layout() {
  return (
    <SidebarProvider>
      <AppSidebar />
      <main className="flex flex-col h-screen">
        <Outlet />          {/* ← WorkOrders or Resources appears here */}
      </main>
      <Toaster />
    </SidebarProvider>
  );
}
        // <SidebarTrigger className="px-6" />
