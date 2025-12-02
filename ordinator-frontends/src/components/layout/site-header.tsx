import { Separator } from "@/components/ui/separator"
import { LogOut } from "lucide-react"
import { Button } from "../ui/button"
import { cn } from "@/lib/utils"

interface SiteHeaderProps {
  header: string,
  className?: string,
}

export function SiteHeader({header, className}: SiteHeaderProps) {
  return (
    <header className={
      cn("flex h-(--header-height) shrink-0 items-center gap-2 border-b transition-[width,height] ease-linear group-has-data-[collapsible=icon]/sidebar-wrapper:h-(--header-height) mt-2 rounded-t-lg", className)
    }>
      <div className="flex w-full items-center gap-1 px-4 lg:gap-2 lg:px-6">
        <h1 className="text-base font-bold">{header}</h1>
        <Separator
          orientation="vertical"
          className="mx-2 data-[orientation=vertical]:h-4"
        />
        <div className="ml-auto flex items-center gap-2">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => {
              
            }}
            title="Logout"
            />
          <LogOut className="h-4 w-4" />
        </div>
      </div>
    </header>
  )
}

