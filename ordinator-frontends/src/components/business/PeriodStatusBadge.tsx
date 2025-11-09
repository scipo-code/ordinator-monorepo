import React from 'react';
import { type ICellRendererParams } from 'ag-grid-community';
import type { PeriodStatus } from '@/types/dto/PeriodStatus';


const PeriodStatusColors: Record<PeriodStatus, {background: string, textColor: string}> = {
   Frozen: { background: '#fcba03', textColor: "text-black"},
   Draft: { background: '#000080', textColor: "text-white"},
   Active: { background: '#adadac', textColor: "text-black"},
   NotScheduled: { background: '#8c1212', textColor: "text-white"},
}

interface PeriodStatusBadgeProps {
  status: PeriodStatus,
  className?: string,
}


export const PeriodStatusBadge: React.FC<PeriodStatusBadgeProps>  = ({ status, className = ''}) => {
  const styles = PeriodStatusColors[status];
  return (
    <span
       className={`inline-block rounded-full px-2.5 py-0.5 text-xs ${className} ${styles.textColor}`}
       style={{
          background: styles.background,
       }}
    >{status}
    </span>
   )
}

interface PeriodStatusBadgeICellRendererProps extends ICellRendererParams<{period_status: PeriodStatus}> {};

export const PeriodStatusICellRenderer: React.FC<PeriodStatusBadgeICellRendererProps> = ({ value }) => {
  if (!value) return null;

  return <PeriodStatusBadge status={value as PeriodStatus} />
}

