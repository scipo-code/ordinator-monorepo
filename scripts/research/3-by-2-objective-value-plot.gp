
set terminal pngcairo enhanced font "Arial,10" size 1200,800
set output 'plot.png'
set multiplot layout 3,2
set fit errorvariables

set xdata time
set timefmt '%Y-%m-%dT%H:%M:%SZ'
set format x '%H:%M:%S'
set xlabel 'Time'
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

set xtics rotate by 270

set style line 1 lc "#000000" lt 1 lw 2.5

plot strategic using 1:2 axes x1y1 with lines linestyle 1 title 'Urgency', strategic using 1:4 axes x1y2 with lines linestyle 1 title 'clustering'  

plot tactical using 1:2 with lines linestyle 1 title 'Urgency'  

plot supervisor using 1:2 with lines linestyle 1 title 'Number of work orders assigned'  

set yrange [0:100]
plot operational_1 using 1:2 with lines linestyle 1 title 'Utilization [%]'  

plot operational_2 using 1:2 with lines linestyle 1 title 'Utilization [%]'  

plot operational_3 using 1:2 with lines linestyle 1 title 'Utilization [%]'  


unset multiplot
