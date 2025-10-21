<!-- 
ISSUE #000 TODO [ ] 2025-07-23 make every field in `SchedulingEnvironment` `pub(crate)` or lower
ISSUE #000 TODO [ ] 2025-07-23 move SAP logic and specifics out of `SchedulingEnvironment` and into the infrastructure layer
ISSUE #000 TODO [ ] 2025-07-23 Implement the applicative rules and rule engine.
-->
# Ordinator
Ordinator is a mathematical scheduling system using Ab-RCU-LNS to model the relavant stakeholders.

# System Architecture
Found in [ARCHITECTURE](ARCHITECTURE.md)

# Quick Start
To quickly get the system to run on a artificial test data set with 100 work orders and 3 technicians:

```bash
cargo test --test master_system_system_0100_work_orders_03_technicians -- --ignored --nocapture
```

Then go to `localhost:3000/swagger` for the API specification.

# Developer Contribution
Found in [CONTRIBUTING](/CONTRIBUTING.md)
