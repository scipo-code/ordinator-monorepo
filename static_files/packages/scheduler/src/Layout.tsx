import { Outlet } from "react-router-dom";
import { SidebarProvider } from "./components/ui/sidebar";
import { AppSidebar } from "@/components/app-sidebar";
import { Toaster } from "@/components/ui/sonner";

export default function Layout() {
  return (
    <SidebarProvider>
      <div className="flex h-screen w-full">
        <AppSidebar />
        <main className="flex flex-col flex-1">
          <Outlet />          {/* ← WorkOrders or Resources appears here */}
        </main>
      </div>
      <Toaster />
    </SidebarProvider>
  );
}
        // <SidebarTrigger className="px-6" />
