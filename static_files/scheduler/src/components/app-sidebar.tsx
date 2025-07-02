import { Calendar, ChevronDown, Home, Inbox, Search, Settings } from "lucide-react"

import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  // SidebarTrigger,
} from "@/components/ui/sidebar"
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "./ui/dropdown-menu"
import { useNavigate, useParams, useLocation } from "react-router-dom"
import { useEffect, useState } from "react"

import { AssetNames } from "../../../../crates/ordinator-contracts/bindings/AssetNames.ts";
// Menu items.
const items = [
  {
    title: "Scheduler",
    url: "/dashboard/:asset",
    icon: Home,
  },
  {
    title: "Workorders",
    url: "/dashboard/:asset/workorders",
    icon: Search,
  },
  {
    title: "Resources",
    url: "/dashboard/:asset/resources",
    icon: Inbox,
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
]
// TODO: The assets should be fetched not hardcoded.
// TODO: The assets available should reflect only what the user has access to.
export function AppSidebar() {
  const navigate = useNavigate();
  const location = useLocation();
  const [workspace, setWorkspace] = useState<string | null>(null);
  const [assets, setAssets] = useState<AssetNames[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const { asset } = useParams<{ asset: string}>();

  useEffect(() => {
    setIsLoading(true);
    fetch("/api/v1/assets")
      .then((res) => {
        if (!res.ok) throw new Error(`HTTP: ${res.status}`)
        return res.json()
      })
      .then((data: AssetNames[])=> {
        setAssets(data)
        setIsLoading(false)
      })
      .catch((err) => {
        console.error("failed to load assets", err)
        setError("Could not load assets")
        setIsLoading(false)
      })
  }, [])

  useEffect(() => {
    if (asset) {
      setWorkspace(asset);
    } else {
      setWorkspace("Select Workspace");
    }
  }, [asset])

  const handleSelectAsset = (ws: string) => {
    // If we're already on a dashboard page, keep the current view
    const currentPath = location.pathname;
    if (currentPath.includes('/dashboard/')) {
      const newPath = currentPath.replace(/\/dashboard\/[^/]+/, `/dashboard/${ws}`);
      navigate(newPath);
    } else {
      navigate(`/dashboard/${ws}`);
    }
    setWorkspace(ws);
  }

  // const handleNavigation = (url: string) => {
  //   if (!asset) {
  //     // If no asset is selected, show a message or handle it appropriately
  //     alert('Please select a workspace first');
  //     return;
  //   }
  //   navigate(url.replace(':asset', asset));
  // }

  return (
    <Sidebar collapsible="icon">
      <SidebarContent>
        <SidebarMenu>
          <SidebarMenuItem>
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <SidebarMenuButton>
                  <span className="font-bold px-2">{workspace}</span>
                  <ChevronDown className="ml-auto" />
                </SidebarMenuButton>
              </DropdownMenuTrigger>


            <DropdownMenuContent className="w[-radix--popper-anchor-width]">
              {isLoading && (
                <DropdownMenuItem disabled>
                  <span>Loading...</span>
                </DropdownMenuItem>
              )}
              {error && (
                <DropdownMenuItem disabled>
                  <span>{error}</span>
                </DropdownMenuItem>
              )}
              {!error &&
               !isLoading &&
               assets.map((a) => (
                 <DropdownMenuItem key={a} onClick={() => handleSelectAsset(a)}>
                   <span>{a}</span>
                 </DropdownMenuItem>
               ))}

              {
              !error &&
              !isLoading &&
              assets.length === 0 && (
              
                <DropdownMenuItem disabled>
                  <span>"No workspaces found"</span>
                </DropdownMenuItem>
            )
            }
              </DropdownMenuContent>
            
            </DropdownMenu>
          </SidebarMenuItem>
        </SidebarMenu>
                
        <SidebarGroup>
          <SidebarGroupContent>
            <SidebarMenu>
              {items.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton asChild>
                    <a href={item.url}>
                      <item.icon />
                      <span>{item.title}</span>
                    </a>
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

