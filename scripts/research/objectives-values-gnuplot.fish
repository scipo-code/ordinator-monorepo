#!/usr/bin/env fish
# This is the script associated with the `test_complete_system` integration test.

rm logging/logs/ordinator.research.log

just run-test &

while test ! -f ./logging/logs/ordinator.research.log; or grep -q not READY ./logging/logs/ordinator.research.log
    echo "ORDINATOR NOT READY"
    sleep 1
end

sleep 30s

pkill master_system_t

jq '
  select(.threadName == "TEST_STRATEGIC" and (.fields | has("objective_value")) ) |
  [ 
  .timestamp, 
  .fields.objective_value.objective_value, 
  .fields.objective_value.urgency[1], 
  .fields.objective_value.resource_penalty[1], 
  .fields.objective_value.clustering_value[1]
  ] |
  @tsv' logging/logs/ordinator.research.log | sed 's/"//g' | sed 's/\\t/\t/g' >./scripts/research/data/strategic.txt

jq '
  select(.threadName == "TEST_TACTICAL" and (.fields | has("objective_value"))) |
  [ 
  .timestamp, 
  .fields.objective_value.objective_value, 
  .fields.objective_value.urgency[1], 
  .fields.objective_value.resource_penalty[1] 
  ] |
  @tsv' logging/logs/ordinator.research.log | sed 's/"//g' | sed 's/\\t/\t/g' >./scripts/research/data/tactical.txt

jq '
  select(.threadName == "TEST_SUPERVISOR" and (.fields | has("objective_value"))) |
  [ 
  .timestamp, 
  .fields.objective_value 
  ] |
  @tsv' logging/logs/ordinator.research.log | sed 's/"//g' | sed 's/\\t/\t/g' >./scripts/research/data/supervisor.txt

jq '
  select(.threadName == "TEST_OP-001-01" and (.fields | has("objective_value"))) |
  [ 
  .timestamp, 
  .fields.objective_value.hands_on_tool_time, 
  .fields.objective_value.assess, 
  .fields.objective_value.assign, 
  .fields.objective_value.total_work_order_activities 
  ] |
  @tsv' logging/logs/ordinator.research.log | sed 's/"//g' | sed 's/\\t/\t/g' >./scripts/research/data/operational-001-01.txt

jq '
  select(.threadName == "TEST_OP-001-02" and (.fields | has("objective_value"))) |
  [ 
  .timestamp, 
  .fields.objective_value.hands_on_tool_time, 
  .fields.objective_value.assess, 
  .fields.objective_value.assign, 
  .fields.objective_value.total_work_order_activities 
  ] |
  @tsv' logging/logs/ordinator.research.log | sed 's/"//g' | sed 's/\\t/\t/g' >./scripts/research/data/operational-001-02.txt

jq '
  select(.threadName == "TEST_OP-002-01" and (.fields | has("objective_value"))) |
  [ 
  .timestamp, 
  .fields.objective_value.hands_on_tool_time, 
  .fields.objective_value.assess, 
  .fields.objective_value.assign, 
  .fields.objective_value.total_work_order_activities 
  ] |
  @tsv' logging/logs/ordinator.research.log | sed 's/"//g' | sed 's/\\t/\t/g' >./scripts/research/data/operational-002-01.txt

gnuplot -e "
  strategic='scripts/research/data/strategic.txt';
  tactical='scripts/research/data/tactical.txt';
  supervisor='scripts/research/data/supervisor.txt';
  operational_1='scripts/research/data/operational-001-01.txt';
  operational_2='scripts/research/data/operational-001-02.txt';
  operational_3='scripts/research/data/operational-002-01.txt';
" ./scripts/research/3-by-2-objective-value-plot.gp
