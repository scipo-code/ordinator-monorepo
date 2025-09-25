#!/usr/bin/env -S perl
use strict;
use warnings;

use Getopt::Long;
use Math::Random;

my $count = 1200;
my $output_file;
GetOptions(
    'count=i'    => \$count,
    'output=s'   => \$output_file,
) or die "Usage: $0 [--count N] [--output FILE] \n";

print "vec![\n";

for (my $i = 0; $i < $count; $i++) {
  my $work_order_number = 1111990400 + $i;
  my $priority = (poisson_priority(1,2));
  my @operations = generate_operations();


    print "    WorkOrderData {\n";
    print "        work_order_number: WorkOrderNumber($work_order_number),\n";
    print "        priority: WorkOrderType::Wdf(Priority::new_int($priority)),\n";
    print "        operations: vec![\n";


    for my $op (@operations) {
      print "            OperationInput {\n";
      print "                activity: $op->{activity},\n";
      print "                work_remaining: $op->{work_remaining},\n";
      print "                early_start: (2025, 1, 1, 7, 0, 0),\n";
      print "                early_finish: (2025, 1, 2, 7, 0, 0),\n";
      print "                preparation: $op->{preparation},\n";
      print "                resource: Resources::$op->{resource},\n";
      print "            },\n";
    }
    print "        ],\n";
    print "        basic_start: (2025, 1, 1),\n";
    print "        basic_finish: (2025, 1, 1),\n";
    print "        easd: (2025, 1, 1),\n";
    print "        lafd: (2024, 10, 1),\n";
    print "        codes: (true, true),\n";
    print "    },\n";

}
 
print "]\n";
  
sub poisson_priority {
  my $lambda = 2.5;
  my $r = rand();

  return 1 if $r < 0.10;
  return 2 if $r < 0.35;
  return 3 if $r < 0.65;
  return 4 if $r < 0.85;
  return 5 if $r < 0.95;
  return 6;
}

sub generate_operations {

  my @list_of_resources = qw(MtnCran MtnElec MtnInst MtnLagg MtnMech MtnRigg MtnRope MtnRous MtnScaf MtnTele Prodtech);

    my @operations;
  for (my $j = 1; $j < 4; $j++) {

    my $resource_index = int(rand @list_of_resources);
    my $resource = $list_of_resources[$resource_index];

    my $base = -log(rand()) * 15;

    my $work = sprintf("%.1f", $base < 0.5 ? 0.5 : $base > 40 ? 40 : $base);
  

    my $prep = $work > 10 ? "2.0" : "1.0";

    push @operations, {
      activity => $j * 10,
      work_remaining => $work,
      preparation => $prep,
      resource => $resource,
    }
  }
  return @operations;
}
