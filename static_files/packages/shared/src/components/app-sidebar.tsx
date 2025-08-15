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

// Menu items.
// TODO: The assets available should reflect only what the user has access to.

export type SidebarItem = {
  title: string,
  url: string,
  icon: React.ComponentType,
}

export function AppSidebar({ items } : {items: SidebarItem[] }) {
  const { asset } = useParams();
  return (
    <Sidebar collapsible="none" className="min-h-screen">
      <SidebarHeader>
        <span className="font-bold">{asset ? asset : "Select asset"}</span>
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


