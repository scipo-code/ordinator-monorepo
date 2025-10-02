import { Outlet, useParams } from "react-router-dom";
import { SidebarProvider } from "./components/ui/sidebar";
import { SidebarItem } from "@scipo-code/shared";
import { AppSidebar } from "@/components/app-sidebar";
import { Toaster } from "@/components/ui/sonner";
import { Calendar, Home, Search, Settings, Users } from "lucide-react";
import { useMemo } from "react";

export default function Layout() {
  const { asset } = useParams();
  const items = useMemo((): SidebarItem[] => {
    
    return [
        {
          title: "Scheduler",
          url: `${asset}/dashboard`,
          icon: Home,
        },
        {
          title: "Workorders",
          url: "#",
          icon: Search,
        },
        {
          title: "Resources",
          url: `${asset}/resources`,
          icon: Users,
        },
        {
          title: "Calendar",
          url: "#",
          icon: Calendar,
        },
        {
          title: "Search",
          url: "#",
          icon: Search,
        },
        {
          title: "Settings",
          url: "#",
          icon: Settings,
        },
      ] as SidebarItem[];
  }, [asset])

  return (
    <SidebarProvider>
      <div className="flex h-screen w-full">
        <AppSidebar items={items} />
        <main className="flex flex-col flex-1 min-w-0">
          <Outlet />          {/* ← WorkOrders or Resources appears here */}
        </main>
      </div>
      <Toaster />
    </SidebarProvider>
  );
}
        // <SidebarTrigger className="px-6" />
