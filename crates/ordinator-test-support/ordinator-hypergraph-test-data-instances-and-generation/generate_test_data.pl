#!/usr/bin/env perl
use strict;
use warnings;
use JSON::PP;
use POSIX qw(strftime mktime);

# Configuration
my $NUM_PERIODS = 52;
my $NUM_WORK_ORDERS = 1000;
my $NUM_TECHNICIANS = 100;
my $BASE_DATE = "2025-01-13";  # Monday, start of first period

# Skills available
my @SKILLS = ("MtnMech", "MtnElec");

# Parse base date
my ($base_year, $base_month, $base_day) = split(/-/, $BASE_DATE);

sub add_days {
    my ($date_str, $days) = @_;
    my ($y, $m, $d) = split(/-/, $date_str);
    my $epoch = mktime(0, 0, 12, $d, $m - 1, $y - 1900);
    $epoch += $days * 86400;
    return strftime("%Y-%m-%d", localtime($epoch));
}

sub format_datetime {
    my ($date_str, $hour, $min, $sec) = @_;
    return sprintf("%sT%02d:%02d:%02d", $date_str, $hour, $min, $sec);
}

# Generate Periods
# Each period is a NaiveDate (just the date string)
# Periods are 14 days apart
my @periods = ();
for my $i (0 .. $NUM_PERIODS - 1) {
    my $period_start = add_days($BASE_DATE, $i * 14);
    push @periods, $period_start;
}

# Generate WorkOrders
# WorkOrder structure:
# {
#   "work_order_number": u64 (10 digits),
#   "basic_start_date": "YYYY-MM-DD",
#   "activities": [{ "activity_number": u64, "number_of_people": u64, "resource": "MtnMech"|"MtnElec" }]
# }
my @work_orders = ();
for my $i (0 .. $NUM_WORK_ORDERS - 1) {
    my $work_order_number = 1000000001 + $i;  # 10-digit number

    # Pick a random period and a random day within it
    my $period_idx = int(rand($NUM_PERIODS));
    my $period_start = $periods[$period_idx];
    my $day_offset = int(rand(14));  # 0-13 days within the period
    my $basic_start_date = add_days($period_start, $day_offset);

    # Generate 1-3 activities per work order
    my $num_activities = 1 + int(rand(3));
    my @activities = ();
    for my $j (0 .. $num_activities - 1) {
        my $activity_number = ($j + 1) * 10;  # 10, 20, 30...
        my $number_of_people = 1 + int(rand(3));  # 1-3 people
        my $skill = $SKILLS[int(rand(scalar @SKILLS))];
        push @activities, {
            activity_number => $activity_number,
            number_of_people => $number_of_people,
            resource => $skill,
        };
    }

    push @work_orders, {
        work_order_number => $work_order_number,
        basic_start_date => $basic_start_date,
        activities => \@activities,
    };
}

# Generate Technicians
# Technician structure for JSON (intermediate format):
# {
#   "id": usize,
#   "skills": ["MtnMech", "MtnElec"],
#   "availabilities": [["2025-01-13T08:00:00", "2025-01-13T17:00:00"], ...]
# }
my @technicians = ();
for my $i (0 .. $NUM_TECHNICIANS - 1) {
    my $tech_id = $i + 1;

    # Each technician has 1-2 skills
    my @tech_skills;
    if (rand() < 0.3) {
        # 30% have both skills
        @tech_skills = @SKILLS;
    } else {
        # 70% have one skill
        @tech_skills = ($SKILLS[int(rand(scalar @SKILLS))]);
    }

    # Each technician has availability for multiple periods
    # For simplicity, give each technician availability across all 52 periods
    # but only specific days (e.g., weekdays)
    my @availabilities = ();

    # Generate one availability block per period (covering the first week of each period)
    for my $period_idx (0 .. $NUM_PERIODS - 1) {
        my $period_start = $periods[$period_idx];
        # Availability for the first 5 days of each period (Mon-Fri of first week)
        my $avail_start = format_datetime($period_start, 8, 0, 0);
        my $avail_end_date = add_days($period_start, 4);  # 5 days
        my $avail_end = format_datetime($avail_end_date, 17, 0, 0);
        push @availabilities, [$avail_start, $avail_end];
    }

    push @technicians, {
        id => $tech_id,
        skills => \@tech_skills,
        availabilities => \@availabilities,
    };
}

# Write JSON files
my $json = JSON::PP->new->pretty->canonical;

# Write periods.json
open(my $pf, '>', 'tests/periods.json') or die "Cannot open periods.json: $!";
print $pf $json->encode(\@periods);
close($pf);

# Write work_orders.json
open(my $wf, '>', 'tests/work_orders.json') or die "Cannot open work_orders.json: $!";
print $wf $json->encode(\@work_orders);
close($wf);

# Write technicians.json
open(my $tf, '>', 'tests/technicians.json') or die "Cannot open technicians.json: $!";
print $tf $json->encode(\@technicians);
close($tf);

print "Generated:\n";
print "  - tests/periods.json ($NUM_PERIODS periods)\n";
print "  - tests/work_orders.json ($NUM_WORK_ORDERS work orders)\n";
print "  - tests/technicians.json ($NUM_TECHNICIANS technicians)\n";
