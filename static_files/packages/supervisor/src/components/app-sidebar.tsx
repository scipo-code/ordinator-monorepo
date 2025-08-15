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
import { Link, useParams } from "react-router-dom"
import { SidebarItem } from "@scipo-code/shared"



export function AppSidebar({ items } : {items: SidebarItem[] }) {
  const { asset } = useParams();
  return (
    <Sidebar collapsible="none" className="min-h-screen">
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




