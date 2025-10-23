set terminal pngcairo enhanced font "Liberation Mono,14" size 1300,1350
timestamp = strftime("%Y-%m-%d", time(0))
set output timestamp . output_file
set multiplot layout 3,2
set fit errorvariables

set xdata time
set timefmt '%Y-%m-%dT%H:%M:%SZ'
set format x '%H:%M:%S'
set xlabel ''

set key top right
set key box
set key outside above offset 0,0.0
set key width 2
set key spacing 1.5

set grid linewidth 2 linestyle 6

set xtics rotate by 315

set style line 1 lc "#000000" lt 1 lw 2.5

set lmargin 12
set rmargin 5.5
set ylabel "Urgency\n[priority \\& LAFD] (MIN)"
plot strategic using 1:2 with lines linestyle 1 title "Scheduler (<2y) - Strategic Model" 

set lmargin 10
set rmargin 7.5
set ylabel "Urgency\n[priority \\& LAFD] (MIN)"
plot tactical using 1:2 with lines linestyle 1 title "Scheduler (<4M) - Tactical Model" 

set lmargin 12
set rmargin 5.5
set yrange [0:100]
set ylabel "Projected Scheduling\nCompliance [%] (MAX)"
plot supervisor using 1:2 with lines linestyle 1 title "Supervisor (<4W)" 

set yrange [0:100]
set lmargin 10
set rmargin 7.5
set ylabel "Hands On Tool Time\n[%] (MAX)"
plot operational_1 using 1:2 with lines linestyle 1 title 'Technician MTN-MECH'  

set lmargin 12
set rmargin 5.5
plot operational_2 using 1:2 with lines linestyle 1 title 'Technician MTN-ELEC'  

set lmargin 10
set rmargin 7.5
plot operational_3 using 1:2 with lines linestyle 1 title 'Technician MTN-INST'  



unset multiplot

