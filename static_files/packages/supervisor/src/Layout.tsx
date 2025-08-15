import { Outlet, useParams } from "react-router-dom";
import { SidebarProvider } from "./components/ui/sidebar";
import { AppSidebar, SidebarItem } from "@scipo-code/shared";
import { Toaster } from "@/components/ui/sonner";
import { useMemo } from "react";
import { CalendarDays, ClipboardList, Settings, Users } from "lucide-react";

export default function Layout() {
  const { asset } = useParams();
  

  const items = useMemo((): SidebarItem[] =>  {
    return [
      {
        title: "Schedule",
        url: `/${asset}/schedule`,
        icon: CalendarDays,
      },
      {
        title: "Frozen Plan",
        url: `/${asset}/frozen_plan`,
        icon: ClipboardList,
      },
      {
        title: "Resources",
        url: `/${asset}/resources`,
        icon: Users,
      },
      {
        title: "Settings",
        url: "#",
        icon: Settings,
      },
    ] as SidebarItem[];
  }, [asset]
  )
  
  return (
    <SidebarProvider>
      <div className="flex h-screen w-full">
        <AppSidebar items={items} />
        <main className="flex flex-col flex-1">
          <Outlet />          {/* ← WorkOrders or Resources appears here */}
        </main>
      </div>
      <Toaster />
    </SidebarProvider>
  );
}
        // <SidebarTrigger className="px-6" />
