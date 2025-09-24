#!/usr/bin/env fish
# This is the script associated with the `test_complete_system` integration test.

rm $ORDINATOR_LOG_DIR/ordinator/ordinator.research.log

cargo test --test master_system_test_2 -- --ignored --nocapture 2>temp_output_from_program.log &

while test ! -f $ORDINATOR_LOG_DIR/ordinator/ordinator.research.log; or grep -q not READY $ORDINATOR_LOG_DIR/ordinator/ordinator.research.log
    echo "ORDINATOR NOT READY"
    sleep 1
end

sleep 2m

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
  join(" ")' $ORDINATOR_LOG_DIR/ordinator/ordinator.research.log | sed 's/"//g' >./scripts/research/data/strategic.dat

jq '
  select(.threadName == "TEST_TACTICAL" and (.fields | has("objective_value"))) |
  [ 
  .timestamp, 
  .fields.objective_value.objective_value, 
  .fields.objective_value.urgency[1], 
  .fields.objective_value.resource_penalty[1] 
  ] |
  join(" ")' $ORDINATOR_LOG_DIR/ordinator/ordinator.research.log | sed 's/"//g' >./scripts/research/data/tactical.dat

jq '
  select(.threadName == "TEST_SUPERVISOR" and (.fields | has("objective_value"))) |
  [ 
  .timestamp, 
  .fields.objective_value.percent 
  ] |
  join(" ")' $ORDINATOR_LOG_DIR/ordinator/ordinator.research.log | sed 's/"//g' >./scripts/research/data/supervisor.dat

jq '
  select(.threadName == "TEST_OP-001-01" and (.fields | has("objective_value"))) |
  [ 
  .timestamp, 
  .fields.objective_value.hands_on_tool_time, 
  .fields.objective_value.assess, 
  .fields.objective_value.assign, 
  .fields.objective_value.total_work_order_activities 
  ] |
  join(" ")' $ORDINATOR_LOG_DIR/ordinator/ordinator.research.log | sed 's/"//g' >./scripts/research/data/operational-001-01.dat

jq '
  select(.threadName == "TEST_OP-001-02" and (.fields | has("objective_value"))) |
  [ 
  .timestamp, 
  .fields.objective_value.hands_on_tool_time, 
  .fields.objective_value.assess, 
  .fields.objective_value.assign, 
  .fields.objective_value.total_work_order_activities 
  ] |
  join(" ")' $ORDINATOR_LOG_DIR/ordinator/ordinator.research.log | sed 's/"//g' >./scripts/research/data/operational-001-02.dat

jq '
  select(.threadName == "TEST_OP-002-01" and (.fields | has("objective_value"))) |
  [ 
  .timestamp, 
  .fields.objective_value.hands_on_tool_time, 
  .fields.objective_value.assess, 
  .fields.objective_value.assign, 
  .fields.objective_value.total_work_order_activities 
  ] |
  join(" ")' $ORDINATOR_LOG_DIR/ordinator/ordinator.research.log | sed 's/"//g' >./scripts/research/data/operational-002-01.dat

gnuplot -e "
  strategic='scripts/research/data/strategic.dat';
  tactical='scripts/research/data/tactical.dat';
  supervisor='scripts/research/data/supervisor.dat';
  operational_1='scripts/research/data/operational-001-01.dat';
  operational_2='scripts/research/data/operational-001-02.dat';
  operational_3='scripts/research/data/operational-002-01.dat';
" ./scripts/research/3-by-2-objective-value-plot.gp
