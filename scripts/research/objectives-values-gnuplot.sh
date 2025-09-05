cat logging/logs/ordinator.research.log | jq 'select(.threadName == "TEST-OP-002-01") | .fields | .objective_value | .hands_on_tool_time' tests/logging/logs/ordinator.research.log 



# k | [.timestamp, .value] | @tsv' | gnuplot -e "plot '-' using 1:2 with lines"
