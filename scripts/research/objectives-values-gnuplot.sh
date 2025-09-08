# This is the script associated with the `test_complete_system` integration test.

jq '
  select(.threadName == "TEST_STRATEGIC" and (.fields | has("strategic_objective_accepted"))) |
  [ 
  .timestamp, 
  .fields.strategic_objective_accepted.objective_value, 
  .fields.strategic_objective_accepted.urgency[1], 
  .fields.strategic_objective_accepted.resource_penalty[1], 
  .fields.strategic_objective_accepted.clustering_value[1]
  ] |
  @tsv' logging/logs/ordinator.research.log | sed 's/"//g' | sed 's/\\t/\t/g' > ./scripts/research/data/strategic.txt

jq '
  select(.threadName == "TEST_TACTICAL" and (.fields | has("tactical_objective_accepted"))) |
  [ 
  .timestamp, 
  .fields.tactical_objective_accepted.objective_value, 
  .fields.tactical_objective_accepted.urgency[1], 
  .fields.tactical_objective_accepted.resource_penalty[1] 
  ] |
  @tsv' logging/logs/ordinator.research.log | sed 's/"//g' | sed 's/\\t/\t/g' > ./scripts/research/data/tactical.txt

jq '
  select(.threadName == "TEST_SUPERVISOR" and (.fields | has("supervisor_objective_accepted"))) |
  [ 
  .timestamp, 
  .fields.supervisor_objective_accepted 
  ] |
  @tsv' logging/logs/ordinator.research.log | sed 's/"//g' | sed 's/\\t/\t/g' > ./scripts/research/data/supervisor.txt

jq '
  select(.threadName == "TEST_OP-001-01" and (.fields | has("operational_objective_accepted"))) |
  [ 
  .timestamp, 
  .fields.operational_objective_accepted.hands_on_tool_time, 
  .fields.operational_objective_accepted.assess, 
  .fields.operational_objective_accepted.assign, 
  .fields.operational_objective_accepted.total_work_order_activities 
  ] |
  @tsv' logging/logs/ordinator.research.log | sed 's/"//g' | sed 's/\\t/\t/g' > ./scripts/research/data/operational-001-01.txt

jq '
  select(.threadName == "TEST_OP-001-02" and (.fields | has("operational_objective_accepted"))) |
  [ 
  .timestamp, 
  .fields.operational_objective_accepted.hands_on_tool_time, 
  .fields.operational_objective_accepted.assess, 
  .fields.operational_objective_accepted.assign, 
  .fields.operational_objective_accepted.total_work_order_activities 
  ] |
  @tsv' logging/logs/ordinator.research.log | sed 's/"//g' | sed 's/\\t/\t/g' > ./scripts/research/data/operational-001-02.txt

jq '
  select(.threadName == "TEST_OP-002-01" and (.fields | has("operational_objective_accepted"))) |
  [ 
  .timestamp, 
  .fields.operational_objective_accepted.hands_on_tool_time, 
  .fields.operational_objective_accepted.assess, 
  .fields.operational_objective_accepted.assign, 
  .fields.operational_objective_accepted.total_work_order_activities 
  ] |
  @tsv' logging/logs/ordinator.research.log | sed 's/"//g' | sed 's/\\t/\t/g' > ./scripts/research/data/operational-002-01.txt



gnuplot -e "
  strategic='scripts/research/data/strategic.txt';
  tactical='scripts/research/data/tactical.txt';
  supervisor='scripts/research/data/supervisor.txt';
  operational_1='scripts/research/data/operational-001-01.txt';
  operational_2='scripts/research/data/operational-001-02.txt';
  operational_3='scripts/research/data/operational-002-01.txt';
" ./scripts/research/3-by-2-objective-value-plot.gp
