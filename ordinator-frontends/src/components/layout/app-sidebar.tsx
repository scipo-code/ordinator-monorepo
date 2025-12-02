import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar"
import { CalendarDays, ClipboardList, Home, Users } from "lucide-react";
import { Link, useParams } from "react-router-dom"
import { type SidebarItem } from "@/types/sidebar";


export function AppSidebar({ operator, ...props }: { operator: "scheduler" | "supervisor" | "technician" } & React.ComponentProps<typeof Sidebar>) {
  const { asset } = useParams<{ asset: string }>();
  const sidebarItems = {

    scheduler: [
        {
          title: "Scheduler",
          url: `/scheduler/${asset}/dashboard`,
          icon: Home,
        },
        {
          title: "Resources",
          url: `/scheduler/${asset}/resources`,
          icon: Users,
        },
      ],
      supervisor: [
        {
          title: "Schedule",
          url: `/supervisor/${asset}/schedule`,
          icon: CalendarDays,
        },
        {
          title: "Frozen Plan",
          url: `/supervisor/${asset}/frozen_plan`,
          icon: ClipboardList,
        },
        {
          title: "Resources",
          url: `/supervisor/${asset}/resources`,
          icon: Users,
        },
      ],
      technician: [
        {
          title: "Dashboard",
          url: `/technician/${asset}/dashboard`,
          icon: CalendarDays,
        },
      ]
  };

  const items = sidebarItems[operator] as SidebarItem[] || [];



  
  return (
    <Sidebar collapsible="none" {...props}>
      <SidebarHeader className="flex items-center gap-3 p-4">
        <div className="flex items-center gap-2">
          <img src="./ordinator-logo.svg" alt="Ordinator" className="h-8 w-8" />
          <span className="font-semibold text-lg">Ordinator</span>
        </div>
        {asset && (
          <div className="px-2 py-1 bg-blue-100 text-gray-700 text-xs rounded-md font-medium">
            Asset: {asset}
          </div>
        )}
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupContent>
            <SidebarMenu>
              {items.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton asChild>
                    <Link to={item.url}>
                      <item.icon />
                      <span>{item.title}</span>
                    </Link>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
    </Sidebar>
  )
}



