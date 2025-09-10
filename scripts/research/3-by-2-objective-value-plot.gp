
set terminal pngcairo enhanced font "Arial,10" size 1200,800
set output 'plot.png'
set multiplot layout 3,2
set fit errorvariables

set datafile separator '\t' missing "NaN"

set xdata time
set timefmt '%Y-%m-%dT%H:%M:%SZ'
set format x '%H:%M:%S'
set xlabel 'Time'
set ylabel 'Objective Value'

set title 'Strategic Objective Values Over Time'
plot strategic using 1:2 with linespoints title 'Urgency'  

set title 'Tactical Objective Values Over Time'
plot tactical using 1:2 with linespoints title 'Urgency'  

set title 'Supervisor Objective Values Over Time'
plot supervisor using 1:2 with linespoints title 'Number of work orders assigned'  

set title 'Operational 1 Objective Values Over Time'
plot operational_1 using 1:2 with linespoints title 'Utilization [%]'  

set title 'Operational 2 Objective Values Over Time'
plot operational_2 using 1:2 with linespoints title 'Utilization [%]'  

set title 'Operational 3 Objective Values Over Time'
plot operational_3 using 1:2 with linespoints title 'Utilization [%]'  


unset multiplot
