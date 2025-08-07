import * as React from 'react';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

// import { SupervisorMainTableDto } from '@scipo-code/shared';



// const data = {
//   days: {
//     20250113: {
//       work_order_by_work_center: {
        
//         MTN_ELEC: {
//           OP_01_001: [
//             {
//               area: null,
//               work_order_number: 1002,
//               activity_number: 10,
//               permit: null,
//               material: "Smat",
//               icc: null,
//               description: "TEST",
//               hours_worked: 0,
//               hours_planned: 5,
//               percentage_complete: 0
//             },
//             {
//               area: null,
//               work_order_number: 1002,
//               activity_number: 10,
//               permit: null,
//               material: "Smat",
//               icc: null,
//               description: "TEST",
//               hours_worked: 0,
//               hours_planned: 5,
//               percentage_complete: 0
//             }
//           ],
//           OP_01_002: [
//             {
//               area: null,
//               work_order_number: 1002,
//               activity_number: 10,
//               permit: null,
//               material: "Smat",
//               icc: null,
//               description: "TEST",
//               hours_worked: 0,
//               hours_planned: 5,
//               percentage_complete: 0
//             },
//           ]
//         },
//         MTN_ROCE: {
//           OP_02_001: [
//             {
//               area: null,
//               work_order_number: 1002,
//               activity_number: 10,
//               permit: null,
//               material: "Smat",
//               icc: null,
//               description: "TEST",
//               hours_worked: 0,
//               hours_planned: 5,
//               percentage_complete: 0
//             },
//           ]
//         }
//       }
//     },
//   }
// };

// const data = [
//   { name: 'John', age: 25, location: 'New York', options: ['Edit', 'Delete'] },
//   { name: 'Jane', age: 30, location: 'London', options: ['Edit', 'Archive'] },
// ];

const groupedData = {
  'MTN_ELEC': [
    { name: 'Work Order 1002', age: 10, location: 'OP_01_001', options: ['Edit', 'Complete'] },
    { name: 'Work Order 1003', age: 5, location: 'OP_01_002', options: ['Edit', 'Start'] },
  ],
  'MTN_ROCE': [
    { name: 'Work Order 1004', age: 8, location: 'OP_02_001', options: ['Edit', 'Review'] },
    { name: 'Work Order 1005', age: 12, location: 'OP_02_002', options: ['Edit', 'Complete'] },
  ],
};



const staticHeaders = [
  { id: 'name', label: 'Name', render: (row) => row.name },
  { id: 'age', label: 'Age', render: (row) => row.age },
  { id: 'location', label: 'Location', render: (row) => row.location },
  { id: 'action', label: 'Action', render: (row) => <button onClick={() => alert(`Action on ${row.name}`)}>Action</button> },
  { id: 'options', label: 'Options', render: (row) => (
      <select>
        {row.options.map((option, idx) => (
          <option key={idx} value={option}>
            {option}
          </option>
        ))}
      </select>
    )
  },
];

const CommonTable = ({ headers, data, groupedData }) => {
  if (groupedData) {
    return (
      <Table>
        <TableHeader>
          <TableRow>
            {headers.map((header) => (
              <TableHead key={header.id}>{header.label}</TableHead>
            ))}
          </TableRow>
        </TableHeader>
        <TableBody>
          {Object.entries(groupedData).map(([workCenter, workCenterData]) => (
            <React.Fragment key={workCenter}>
              <TableRow>
                <TableCell 
                  colSpan={headers.length} 
                  className="bg-blue-100 font-bold text-center py-3"
                >
                  {workCenter}
                </TableCell>
              </TableRow>
              {workCenterData.map((row, index) => (
                <TableRow key={`${workCenter}-${index}`}>
                  {headers.map((header) => (
                    <TableCell key={header.id}>{header.render(row)}</TableCell>
                  ))}
                </TableRow>
              ))}
            </React.Fragment>
          ))}
        </TableBody>
      </Table>
    );
  }

  return (
    <Table>
      <TableHeader>
        <TableRow>
          {headers.map((header) => (
            <TableHead key={header.id}>{header.label}</TableHead>
          ))}
        </TableRow>
      </TableHeader>
      <TableBody>
        {data.map((row, index) => (
          <TableRow key={index}>
            {headers.map((header) => (
              <TableCell key={header.id}>{header.render(row)}</TableCell>
            ))}
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
};

const MainTable: React.FC = () => {

  return (
    <div className="w-full">
      <div className="rounded-md border">
        <CommonTable headers={staticHeaders} groupedData={groupedData} />
      </div>
    </div>
  );
};

export default MainTable;
