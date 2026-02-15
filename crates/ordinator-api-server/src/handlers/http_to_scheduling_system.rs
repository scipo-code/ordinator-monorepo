// TODO: Refactor dependency imports to follow best practices
// TODO: Decide on system message location and architecture
// TODO: Separate handler functions; orchestrator provides Communication, SchedulingEnvironment,
//       and SystemSolutions, while handlers provide required context data

// #[cfg(test)]
// mod tests
// {
//     use std::collections::HashMap;

//     use chrono::Utc;
//     use shared_types::agents::tactical::Days;
//     use shared_types::agents::tactical::TacticalResources;
//     use shared_types::scheduling_environment::time_environment::day::Day;
//     use shared_types::scheduling_environment::work_order::operation::Work;
//     use shared_types::scheduling_environment::worker_environment::resources::Resources;

//     #[test]
//     fn test_day_serialize()
//     {
//         let mut hash_map_nested = HashMap::<Day, Work>::new();

//         let mut hash_map = HashMap::<Resources, Days>::new();
//         let day = Day::new(0, Utc::now());
//         day.to_string();
//         hash_map_nested.insert(day, Work::from(123.0));

//         hash_map.insert(Resources::MtnMech,
// Days::new(hash_map_nested.clone()));         let tactical_resources =
// TacticalResources::new(hash_map.clone());         serde_json::to_string(&
// tactical_resources).unwrap();     }
// }
