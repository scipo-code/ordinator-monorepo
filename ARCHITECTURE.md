# Monorepo Organization
The core scheduling functionality is implemented in Rust and can be found in the ['crates'](./crates)
directory.

## Core application - Rust 
The core rust code consists of several crates. Each crate has an associated README.md
where **function**, **design decisions**, and **developer** information can be found.


- [crates/ordinator-actors](crates/ordinator-actors/)
    - [crates/ordinator-actors](crates/ordinator-actors/ordinator-actor-core)
    - [crates/ordinator-actors](crates/ordinator-actors/ordinator-strategic-actor)
    - [crates/ordinator-actors](crates/ordinator-actors/ordinator-tactical-actor)
    - [crates/ordinator-actors](crates/ordinator-actors/ordinator-supervisor-actor)
    - [crates/ordinator-actors](crates/ordinator-actors/ordinator-operational-actor)

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

