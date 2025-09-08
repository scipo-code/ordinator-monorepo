# This is the script associated with the `test_complete_system` integration test.

cat logging/logs/ordinator.research.log | jq 'select(.threadName == "TEST-OP-002-01" and (.fields | has("operational_objective_value_better")) ) |[ 
  .timestamp, 
  .fields.operational_objective_value_better.hands_on_tool_time, 
  .fields.operational_objective_value_better.assess, 
  .fields.operational_objective_value_better.assign, 
  .fields.operational_objective_value_better.total_work_order_activities 
  ] | @tsv' | sed 's/"//g' | sed 's/\\t/\t/g' | gnuplot -e " set terminal png size 800,600; set output 'plot.png'; set datafile separator '\t'; set xdata time; set timefmt '%Y-%m-%dT%H:%M:%SZ'; set format x '%H:%M:%S'; set xlabel 'Time'; set ylabel 'Objective Value'; set title 'Objective Values Over Time'; plot '-' using 1:2 with linespoints title 'Objective Values';"

# gnuplot -e "set terminal png size 800,600; set output 'plot.png'; set datafile separator '\t'; set xlabel 'Time'; set ylabel 'Objective Value'; set title 'Objective Values Over Time'; plot '-' using 2 with linespoints title 'Objective Values'" 


# cargo run | jq -r '[.timestamp, .fields.objective.objective] | @tsv' | gnuplot -e "set terminal png size 800,600; set output 'plot.png'; set datafile separator '\t'; set xlabel 'Time'; set ylabel 'Objective Value'; set title 'Objective Values Over Time'; plot '-' using 2 with linespoints title 'Objective Values'"


# k | [.timestamp, .value] | @tsv' | gnuplot -e "plot '-' using 1:2 with lines"
