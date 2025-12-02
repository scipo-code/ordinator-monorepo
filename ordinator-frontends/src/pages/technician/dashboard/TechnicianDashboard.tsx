import { SiteHeader } from "@/components/layout/site-header";
import { SectionCards } from "./section-cards";
import { ChartAreaInteractive } from "./chart-area";
import { DataTable } from "./data-table";

import data from "./data.json";

export default function TechnicianDashboard() {
  return (
    <>
      <SiteHeader header="Dashboard" className="bg-white"/>
        <div className="flex-1">
          <div className="@container/main flex flex-1 flex-col gap-2 bg-white">
            <div className="flex flex-col gap-4 py-4 md:gap-6 md:py-6">
              <SectionCards />
              <div className="px-4 lg:px-6">
                <ChartAreaInteractive />
              </div>
              <DataTable data={data} />
            </div>
          </div>
        </div>
    </>
  )
}
