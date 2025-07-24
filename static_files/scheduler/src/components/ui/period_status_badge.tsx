import { ICellRendererParams } from 'ag-grid-community';
import { PeriodStatus } from '../../../../../crates/ordinator-contracts/bindings/PeriodStatus.ts';

const PeriodStatusColors: Record<PeriodStatus, string> = {
   Frozen: '#fcba03',
   Draft: '#000080',
   Active: '#ffffff',
   NotScheduled: '#8c1212',
}

interface PeriodStatusBadgeProps {
  status: PeriodStatus,
  className?: string,
}


export const PeriodStatusBadge: React.FC<PeriodStatusBadgeProps>  = ({ status, className = ''}) => {
   return (
      <span
         className={`inline-block rounded-full px-2.5 py-0.5 text-xs text-white ${className}`}
         style={{
            background: PeriodStatusColors[status as PeriodStatus] ?? '#ccc',
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

