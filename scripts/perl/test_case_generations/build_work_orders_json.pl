#!/usr/bin/env -S perl
use strict;
use warnings;
use Getopt::Long;
use JSON::PP; # Use the core JSON module for encoding

# --- Command-line options ---
my $count = 6400;
my $output_file;
GetOptions(
    'count=i'  => \$count,
    'output=s' => \$output_file,
) or die "Usage: $0 [--count N] [--output FILE.json] \n";

# --- Main data generation loop ---
my @work_orders; # This array will hold all the work order data
for (my $i = 0; $i < $count; $i++) {
    my $work_order_number = 1111990000 + $i;
    my $priority          = poisson_priority();
    my @operations        = generate_operations();

    # Generate dates as arrays of numbers [Y, M, D] for proper JSON formatting
    # Note: Fixed potential for day 0 from original `int(rand(30))`
    my $easd = [ 2025, 1 + int(rand(2)), 1 + int(rand(28)) ];
    my $lafd = [ 2025, 1 + int(rand(2)), 1 + int(rand(28)) ];

    # Build a hash for the current work order
    my $work_order_data = {
        work_order_number => "$work_order_number",
        priority          => {Wdf => {Int => $priority}},
        operations        => \@operations, # Embed the array of operation hashes
        basic_start       => [ 2025, 1, 1 ],
        basic_finish      => [ 2025, 1, 1 ],
        easd              => $easd,
        lafd              => $lafd,
        # Use JSON::PP::true for explicit boolean values
        codes             => [ JSON::PP::true, JSON::PP::true ],
    };

    push @work_orders, $work_order_data;
}

# --- Encode the entire data structure to JSON ---
my $json_output = JSON::PP->new->pretty->encode(\@work_orders);

# --- Output to file or STDOUT ---
if ($output_file) {
    open my $fh, '>', $output_file or die "Could not open file '$output_file' $!";
    print $fh $json_output;
    close $fh;
    print STDERR "Successfully wrote $count records to $output_file\n";
}
else {
    print $json_output;
}

# --- Subroutines (modified for JSON output) ---

sub poisson_priority {
    # No changes needed here, it already returns a number.
    my $r = rand();
    return 1 if $r < 0.10;
    return 2 if $r < 0.35;
    return 3 if $r < 0.65;
    return 4 if $r < 0.85;
    return 5 if $r < 0.95;
    return 6;
}

sub generate_operations {
    my @list_of_resources = qw(MTN-CRAN MTN-ELEC MTN-INST MTN-LAGG MTN-MECH MTN-RIGG MTN-ROPE MTN-ROUS MTN-SCAF MTN-TELE PRODTECH);
    my @operations;

    for (my $j = 1; $j < 4; $j++) {
        my $resource_index = int(rand @list_of_resources);
        my $resource       = $list_of_resources[$resource_index];

        # Generate a random work duration from an exponential distribution
        my $base = -log(rand()) * 3;
        
        # Clamp the value between 0.5 and 40.0
        my $work = $base < 0.5 ? 0.5 : $base > 40 ? 40 : $base;
        # Round to one decimal place, ensuring it remains a number for JSON
        $work = sprintf("%.1f", $work) * 1; 

        # Preparation is a number (e.g., 2.0), not a string ("2.0")
        my $prep = $work > 10 ? 2.0 : 1.0;

        push @operations, {
            activity       => $j * 10,
            work_remaining => $work,
            # Dates are now arrays of numbers, the natural JSON equivalent of tuples
            early_start    => [ 2025, 1, 1, 7, 0, 0 ],
            early_finish   => [ 2025, 1, 2, 7, 0, 0 ],
            preparation    => $prep,
            resource       => $resource,
        };
    }
    return @operations;
}
