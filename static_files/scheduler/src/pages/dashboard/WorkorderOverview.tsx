import React, { useState, useMemo, useEffect } from 'react';
import { AgGridReact } from 'ag-grid-react';
import { ColDef, ICellRendererParams } from 'ag-grid-community';
import { useParams, useNavigate } from 'react-router-dom';
// import { agGridThemeLight } from "./theme";
// https://www.youtube.com/watch?v=TrKlKF5au6c&ab_channel=m6io
interface WorkOrder
 {
  id: string;
  title: string;
  description: string;
  status: 'pending' | 'in_progress' | 'completed' | 'cancelled';
  priority: 'low' | 'medium' | 'high';
  startDate?: Date;
  endDate?: Date;
  assignedTo?: string;
}

// Mock data - in real app this would be fetched based on asset
const mockWorkOrders: Record<string, WorkOrder[]> = {
  'DF': [
    {
      id: 'WO-001',
      title: 'Annual Maintenance',
      description: 'Regular maintenance of production line A',
      status: 'in_progress',
      priority: 'high',
      startDate: new Date('2024-03-15'),
      endDate: new Date('2024-03-20'),
      assignedTo: 'John Doe',
    },
    {
      id: 'WO-002',
      title: 'Equipment Calibration',
      description: 'Calibrate measuring instruments',
      status: 'pending',
      priority: 'medium',
      startDate: new Date('2024-03-25'),
      assignedTo: 'Jane Smith',
    }
  ],
  'TL': [
    {
      id: 'WO-003',
      title: 'Safety Inspection',
      description: 'Quarterly safety equipment inspection',
      status: 'completed',
      priority: 'high',
      startDate: new Date('2024-03-01'),
      endDate: new Date('2024-03-02'),
      assignedTo: 'Mike Johnson',
    },
    {
      id: 'WO-004',
      title: 'Software Update',
      description: 'Update control system software',
      status: 'pending',
      priority: 'low',
      startDate: new Date('2024-04-01'),
      assignedTo: 'Sarah Wilson',
    }
  ]
};

const WorkorderOverview: React.FC = () => {
  const { asset } = useParams<{ asset: string }>();
  const navigate = useNavigate();
  const [workOrders, setWorkOrders] = useState<WorkOrder[]>([]);

  useEffect(() => {
    if (!asset) {
      navigate('/dashboard/DF'); // Default to DF if no asset selected
      return;
    }
    setWorkOrders(mockWorkOrders[asset] || []);
  }, [asset, navigate]);

  const columnDefs = useMemo<ColDef[]>(() => [
    {
       field: 'id',
       headerName: 'ID',
       sortable: true,
       filter: true,
       width: 120,
       pinned: 'left' as const,
    },
    { field: 'title', headerName: 'Title', sortable: true, filter: true },
    { field: 'description', headerName: 'Description', sortable: true, filter: true },
    { 
      field: 'status', 
      headerName: 'Status', 
      sortable: true, 
      filter: true,
      cellStyle: params => {
        const status = params.value as WorkOrder['status'];
        const colors: Record<WorkOrder['status'], string> = {
          pending: '#ffd700',
          in_progress: '#87ceeb',
          completed: '#90ee90',
          cancelled: '#ff6b6b'
        };
        return { backgroundColor: colors[status] ?? 'white' };
      }
    },
    { 
      field: 'priority', 
      headerName: 'Priority', 
      sortable: true, 
      filter: true,
      cellStyle: params => {
        const priority = params.value as WorkOrder['priority'];
        const colors: Record<WorkOrder['priority'], string> = {
          low: '#90ee90',
          medium: '#ffd700',
          high: '#ff6b6b'
        };
        return { backgroundColor: colors[priority] ?? 'white' };
      }
    },
    { 
      field: 'startDate', 
      headerName: 'Start Date', 
      sortable: true, 
      filter: true,
      valueFormatter: params => params.value ? new Date(params.value).toLocaleDateString() : 'Not set'
    },
    { 
      field: 'endDate', 
      headerName: 'End Date', 
      sortable: true, 
      filter: true,
      valueFormatter: params => params.value ? new Date(params.value).toLocaleDateString() : 'Not set'
    },
    { field: 'assignedTo', headerName: 'Assigned To', sortable: true, filter: true },
    {
      headerName: 'Actions',
      field: 'actions',
      sortable: false,
      filter: false,
      cellRenderer: (params: ICellRendererParams<WorkOrder>) => {
        if (!params.data) return null;          // or use params.data!.id below
        const { id } = params.data;
        return (
          <div>
            <button onClick={() => handleEdit(id)} style={{ marginRight: '8px' }}>
              Edit
            </button>
            <button onClick={() => handleDelete(id)} style={{ color: 'red' }}>
              Delete
            </button>
          </div>
        );
      }
    }
  ], []);

  const handleEdit = (id: string) => {
    // This would typically trigger a modal or navigation to edit form
    console.log('Edit work order:', id);
  };

  const handleDelete = (id: string) => {
    // This would typically send a delete request to the backend
    console.log('Delete work order:', id);
  };


  if (!asset) {
    return null;
  }

  return (
    <div className="p-4">
      <h2 className="text-2xl font-bold mb-4">Work Orders - {asset}</h2>
      <div className="w-full h-[600px]">
        <AgGridReact
          rowData={workOrders}
          // theme={agGridThemeLight}
          columnDefs={columnDefs}
          defaultColDef={{
            resizable: true,
            sortable: true,
            filter: true,
            flex: 1,
            minWidth: 100
          }}
        />
      </div>
    </div>
  );
};

export default WorkorderOverview;
