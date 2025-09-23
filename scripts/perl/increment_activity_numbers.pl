#!/usr/bin/perl
use strict;
use warnings;

my $increment = 0;
my $found_first = 0;

while (<>) {
  if (/WorkOrderData/) {
    $increment = 0;
    $found_first = 0;
  } elsif (/activity:\s*(\d+)/) {
    if (!$found_first) {
      $found_first = 1;
    } else {
        $increment += 10;
        s/(activity:\s*)(\d+)/$1 . ($2 + $increment)/e;
    }
  }
  print;

}
