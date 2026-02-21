# Monorepo Organization
The core scheduling functionality is implemented in Rust and can be found in the ['crates'](./crates)
directory.

## Core application - Rust 
The core rust code consists of several crates. Each crate has an associated README.md
where **function**, **design decisions**, and **developer** information can be found.


- [crates/ordinator-actors](crates/ordinator-actors/)
    - [crates/ordinator-actors](crates/ordinator-actors/ordinator-actor-core)
    - [crates/ordinator-actors](crates/ordinator-actors/ordinator-actor-weekly)
    - [crates/ordinator-actors](crates/ordinator-actors/ordinator-actor-project)
    - [crates/ordinator-actors](crates/ordinator-actors/ordinator-actor-daily)
    - [crates/ordinator-actors](crates/ordinator-actors/ordinator-actor-operational)

- [crates/ordinator-api-server](crates/ordinator-api-server)
- [crates/ordinator-orchestrator](crates/ordinator-orchestrator)
- [crates/ordinator-total-data-processing](crates/ordinator-total-data-processing)
- [crates/ordinator-contracts](crates/ordinator-contracts)
- [crates/ordinator-orchestrator-actor-traits](crates/ordinator-orchestrator-actor-traits)
- [crates/ordinator-configuration](crates/ordinator-configuration)
- [crates/ordinator-scheduling-environment](crates/ordinator-scheduling-environment)


### Testing
- ['crates/ordinator-system-tests'](crates/ordinator-system-tests)
- ['crates/ordinator-test-support'](crates/ordinator-test-support)


### CLI
- ['crates/ordinator-imperium'](crates/ordinator-imperium)


## Frontend Clients - React & TypeScript

