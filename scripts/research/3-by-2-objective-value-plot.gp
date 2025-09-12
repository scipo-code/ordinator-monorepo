

# set terminal lua tikz tex size 12cm,8cm color fontscale 1.0

set terminal pngcairo enhanced font "Arial,10" size 1200,800
timestamp = strftime("%Y-%m-%dT%H%M%SZ", time(0))
set output timestamp . '1-1-1-3-objectives.png'
set multiplot layout 3,2
set fit errorvariables

set xdata time
set timefmt '%Y-%m-%dT%H:%M:%SZ'
set format x '%H:%M:%S'
set xlabel ''
set ylabel 'Objective Value'

set key top right
set key box
set key outside above offset 0,0.0
set key width 2
set key spacing 1.5

set grid
set ytics nomirror
set ytics nomirror offset 0,1
set y2tics  offset 0,1

set xtics rotate by 315

set style line 1 lc "#000000" lt 1 lw 2.5

plot strategic using 1:2 axes x1y1 with lines linestyle 1 title 'Brian Model - Urgency (MIN)' \
	#, strategic using 1:4 axes x1y2 with lines linestyle 1 title 'clustering'  

plot tactical using 1:2 with lines linestyle 1 title 'Valentin Model - Urgency (MIN)'  

plot supervisor using 1:2 with lines linestyle 1 title 'Supervisor 1 - Number of work orders assigned (MAX)'  

set yrange [0:100]
plot operational_1 using 1:2 with lines linestyle 1 title 'Technician 1 - Utilization [%] (MAX)'  
plot operational_2 using 1:2 with lines linestyle 1 title 'Technician 2 - Utilization [%] (MAX)'  
plot operational_3 using 1:2 with lines linestyle 1 title 'Technician 3 - Utilization [%] (MAX)'  
unset yrange 


unset multiplot
