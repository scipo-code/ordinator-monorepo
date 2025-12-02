
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { Toaster } from "@/components/ui/sonner";
import { AppSidebar } from "./app-sidebar";
import { Outlet } from "react-router-dom";

interface LayoutProps {
  children: React.ReactNode;
  operator: 'scheduler' | 'supervisor' | 'technician';
}

      // style={
      //   {
      //     "--sidebar-width": "calc(var(--spacing) * 72)",
      //     "--header-height": "calc(var(--spacing) * 12)",
      //   } as React.CSSProperties
      // }
export function Layout({children, operator }: LayoutProps) {
  return (
    <SidebarProvider
      >
      <AppSidebar className="bg-inherit" operator={operator} variant="inset"/>
      <SidebarInset className="overflow-auto max-h-screen">
        {children}
      </SidebarInset>
      <Toaster />
    </SidebarProvider>
  );
}
          // <Header variant={variant} />
        // <SidebarTrigger className="px-6" />

export function RoutingLayout({ operator }: { operator: 'scheduler' | 'supervisor' | 'technician'}) {
  return (
    <Layout operator={operator} >
      <Outlet />
    </Layout>
  )
}

