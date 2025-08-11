run:
    cargo run --release --bin ordinator-api-server 2> temp_output_from_program.log


run-test:
    cargo test test_complete_system -- --ignored --nocapture 2> temp_output_from_program.log

bs-test:
    bs target/debug/deps/ordinator_tactical_actor-cd5c23df1ab83245 

export-ts-bindings:
    cargo +nightly test export_bindings
    
build-ordinator-frontends:
    mkdir -p dist/static_files/scheduler
    mkdir -p dist/static_files/supervisor
    cd static_files/ && pnpm install && pnpm -r build
    cp -r static_files/packages/scheduler/dist/* dist/static_files/scheduler/
    cp -r static_files/packages/supervisor/dist/* dist/static_files/supervisor/

build-ordinator-api-windows:
    cross build --target x86_64-pc-windows-gnu --release && cp target/x86_64-pc-windows-gnu/release/ordinator-api-server.exe ./dist/

build-ordinator-api-linux:
    cargo build --release && cp target/release/ordinator-api-server ./dist/ 

create-required-directories:
    mkdir -p dist/logging/logs/
    mkdir -p dist/benches
    mkdir -p dist/profiling
    mkdir -p dist/xlsx_dumps
    cp -r configuration dist/configuration
    cp .env.example dist/.env.example
    cp -r temp_scheduling_environment_database dist/temp_scheduling_environment_database

build-ordinator-for-deployment-windows: (build-ordinator-frontends) (build-ordinator-api-windows) (create-required-directories)

build-ordinator-for-deployment-linux: (build-ordinator-frontends) (build-ordinator-api-linux) (create-required-directories)

version-bump SEMVER EXECUTE="":
    #!/usr/bin/env fish
    cargo release --no-publish {{SEMVER}} {{EXECUTE}} &&  rg -o '([0-9]\.[0-9]\.[0-9])' scheduling_system/Cargo.toml > version 

release-on-github VERSION:    
    gh release create {{VERSION}} ./target/release/imperium --title "Release {{VERSION}}" --notes "download the imperium executable with: ```curl -L --output imperium https://github.com/scipo-code/ordinator-api/releases/download/v0.2.2/imperium```"  

bs:
    #!/usr/bin/env nu
    cargo build --release | ~/.cargo/bin/bs target/release/ordinator-api-server

tr REGEX:
    tail -F logging/logs/ordinator.operational.log | rg {{ REGEX }} | jq
    
list-all-work-orders: 
    #!/usr/bin/env nu
    let work_order_state = imperium status work-orders work-order-state df normal | from json
    $work_order_state | get Orchestrator | get WorkOrderStatus | get Multiple | columns | hx

# call-create-all-plot-for-ablns: call-strategic-inclusion-script call-strategic-exclusion-script call-strategic-resources-addition-script call-strategic-resources-subtraction-script call-strategic-work-order-value-script
#     echo "All 5 simulation scripts have been called"

profile-thread TID DURATION:
    #!/usr/bin/env bash
    set -euo pipefail

    usage() {
        echo "Usage: $0 <TID> [<duration_sec>]"
        echo
        echo "TID: THREAD ID to profile"
        echo "duration_sec: Optional. Time to profile. Defaults to 10s if not provided."
        exit 1
    }

    TID={{ TID }} 
    DURATION={{ DURATION }}

    OUTPUT="profiling/out.perf"
    FOLDED="profiling/folded.perf"
    SVG="profiling/flamegraph.svg"

    rm -rf profiling/*

    echo "Recording perf data for TID=$TID for $DURATION seconds..."
    sudo perf record --call-graph dwarf --all-user -F 999 -o "profiling/perf.data" -g --tid "$TID" -- sleep "$DURATION"

    echo "Converting perf.data to out.perf..."
    sudo perf script -i "profiling/perf.data" > "$OUTPUT"

    echo "Converting the out.perf to a folded file"
    stackcollapse-perf.pl "$OUTPUT" > "$FOLDED"

    echo "Generating flame graph out.svg..."
    flamegraph.pl "$FOLDED" > "$SVG"

    echo "Done. Opening svg file"
    firefox "$SVG"


# call-strategic-inclusion-script:
#     #!/usr/bin/env nu
#     nu imperium/scripts/strategic/simulate_scheduling_inclusion.nu

# call-strategic-exclusion-script:
#     #!/usr/bin/env nu
#     nu imperium/scripts/strategic/simulate_scheduling_exclusion.nu 

# call-strategic-resources-addition-script:
#     #!/usr/bin/env nu
#     nu imperium/scripts/strategic/simulate_resources_addition.nu

# call-strategic-resources-subtraction-script:
#     #!/usr/bin/env nu
#     nu imperium/scripts/strategic/simulate_resources_subtraction.nu

# call-strategic-work-order-value-script:
#     #!/usr/bin/env nu
#     nu imperium/scripts/strategic/simulate_weight_update.nu

