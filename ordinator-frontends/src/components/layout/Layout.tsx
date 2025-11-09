
import { SidebarProvider } from "@/components/ui/sidebar";
import { Toaster } from "@/components/ui/sonner";
import { AppSidebar } from "./app-sidebar";
import { Outlet } from "react-router-dom";

interface LayoutProps {
  children: React.ReactNode;
  variant: 'scheduler' | 'supervisor' | 'technician';
}

export function Layout({children, variant }: LayoutProps) {
  return (
    <SidebarProvider>
      <div className="flex h-screen w-full">
        <AppSidebar variant={variant} />
        <main className="flex flex-col flex-1 min-w-0">
          {children}
        </main>
      </div>
      <Toaster />
    </SidebarProvider>
  );
}
          // <Header variant={variant} />
        // <SidebarTrigger className="px-6" />

export function RoutingLayout({ variant }: { variant: 'scheduler' | 'supervisor' | 'technician'}) {
  return (
    <Layout variant={variant} >
      <Outlet />
    </Layout>
  )
}

