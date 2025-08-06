# Test the assign_to_technicians endpoint
# First, get all available technicians to verify the endpoint is working
GET http://127.0.0.1:3000/api/v1/supervisor/all_technicians/DF/supervisor-1

HTTP 200
[Asserts]
jsonpath "$.all_technicians" exists

# Test assigning technicians to a work order activity
POST http://127.0.0.1:3000/api/v1/supervisor/assign_to_technicians/DF/supervisor-1
Content-Type: application/json

{
  "work_order_number": 2100001234,
  "activity_number": 20,
  "technicians": ["tech-001", "tech-002"]
}

HTTP 201
[Asserts]
jsonpath "$.days" exists

# Test getting supervisor main table after assignment
GET http://127.0.0.1:3000/api/v1/supervisor/supervisor_main_table/DF/supervisor-1

HTTP 200
[Asserts]
jsonpath "$.days" exists

# Test assigning additional technicians to the same work order activity
POST http://127.0.0.1:3000/api/v1/supervisor/assign_to_technicians/DF/supervisor-1
Content-Type: application/json

{
  "work_order_number": 2100001234, 
  "activity_number": 20,
  "technicians": ["tech-003"]
}

HTTP 201
[Asserts]
jsonpath "$.days" exists