$env.RESOURCE_CONFIG_INITIALIZATION = "./imperium/scripts/weekly/resource_configurations/low_resources.toml"

fish -c "cargo build -p scheduling_system --release"
fish -c "cargo run -p scheduling_system --release &"

print "script"
^sleep 100
imperium orchestrator initialize-crew-from-file df imperium/scripts/weekly/resource_configurations/base_resources.toml

print "script"
^sleep 100
imperium orchestrator initialize-crew-from-file df imperium/scripts/weekly/resource_configurations/high_resources.toml

print "script"
^sleep 60
ps | where name =~ "scheduling_syst" | kill $in.pid.0 --force 

print "script"
cd ../generalized-multi-agent-maintenance-scheduling-system/ | just nushell-weekly-data-extract weekly_objective_value_resource_addition_plot.tex
^sleep 10
print "script"

 
