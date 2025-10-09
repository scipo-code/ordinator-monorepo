#!/usr/bin/perl
use strict;
use warnings;

my $increment = 0;
my $found_first = 0;

while (<>) {
  if (/WorkOrderNumber/) {
    $increment = 0;
    $found_first = 0;
  } elsif (/operations_builder\((\d+)/) {
    if (!$found_first) {
      $found_first = 1;
    } else {
        $increment += 10;
        s/(operations_builder\(\s*)(\d+)/$1 . ($2 + $increment)/e;
    }
  }
  print;

}
