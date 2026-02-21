use std::fmt::Debug;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Context;
use anyhow::Result;
use arc_swap::ArcSwap;
use ordinator_configuration::SystemConfigurations;
use ordinator_actor_operational::OperationalApi;
use ordinator_actor_operational::algorithm::operational_solution::OperationalSolution;
use ordinator_orchestrator_actor_traits::ActorFactory;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
use ordinator_actor_weekly::WeeklyApi;
use ordinator_actor_weekly::algorithm::weekly_solution::WeeklySolution;
use ordinator_actor_daily::DailyApi;
use ordinator_actor_daily::algorithm::daily_solution::DailySolution;
use ordinator_actor_project::ProjectApi;
use ordinator_actor_project::algorithm::project_solution::ProjectSolution;

use crate::Orchestrator;
use crate::StartError;

type ActorFactoryDependencies<Ss> = (
    Arc<Mutex<SchedulingEnvironment>>,
    Arc<ArcSwap<Ss>>,
    Arc<ArcSwap<SystemConfigurations>>,
);

impl<Ss> Orchestrator<Ss>
where
    Ss: SystemSolutions<
            Weekly = WeeklySolution,
            Project = ProjectSolution,
            Daily = DailySolution,
            Operational = OperationalSolution,
        >
        + Send
        + Sync
        + 'static
        + Debug,
{
    // TODO: Move the actor registry out of the factory_dependencies
    pub fn extract_factory_dependencies(
        &self,
        asset: &Asset,
    ) -> Result<ActorFactoryDependencies<Ss>>
    {
        Ok((
            Arc::clone(&self.scheduling_environment),
            // TODO: Determine proper location for system_solutions
            Arc::clone(
                self.system_solutions
                    .lock()
                    .unwrap()
                    .get(asset)
                    .with_context(|| format!("Missing SystemSolution for Asset {asset}"))?,
            ),
            Arc::clone(&self.system_configurations),
        ))
    }

    // The `ActorCompositeId` uniquely identifies an actor and specifies all
    // characteristics required for that specific actor instance
    pub fn start_weekly_actor(&mut self, id: &ActorCompositeId) -> Result<()>
    {
        let build_dependencies = self.extract_factory_dependencies(id.asset())?;

        // TODO: Determine source for actor ID (database, API, etc.)
        let communication = <WeeklyApi as ActorFactory<Ss>>::construct_actor(
            id.clone(),
            build_dependencies.0,
            build_dependencies.1,
            build_dependencies.2,
            self.state_link_bus.lock().unwrap().add_rx(),
            self.error_sender.clone(),
        )
        .with_context(|| format!("Could not create WeeklyActor for Asset {}", id.asset()))?;

        // TODO: Extract registry access pattern into a helper method
        self.actor_registries
            .lock()
            .unwrap()
            .get_mut(id.asset())
            .expect("The ActorRegistry for asset should exist before creating Actors on it")
            .weekly_agent_sender = communication;
        Ok(())
    }

    pub fn start_project_actor(&mut self, id: &ActorCompositeId) -> Result<()>
    {
        // TODO: Insert entry into the `SchedulingEnvironment`
        let build_dependencies = self.extract_factory_dependencies(id.asset())?;

        // TODO: Determine source for actor ID
        let communication = <ProjectApi as ActorFactory<Ss>>::construct_actor(
            id.clone(),
            build_dependencies.0,
            build_dependencies.1,
            build_dependencies.2,
            self.state_link_bus.lock().unwrap().add_rx(),
            self.error_sender.clone(),
        )
        .with_context(|| format!("Could not create ProjectActor for Asset {}", id.asset()))?;

        self.actor_registries
            .lock()
            .unwrap()
            .get_mut(id.asset())
            .expect("The ActorRegistry for asset should exist before creating Actors on it")
            .project_agent_sender = communication;
        Ok(())
    }

    // TODO: Move the ActorSpecification into the SchedulingEnvironment
    pub fn start_daily_actor(&mut self, id: &ActorCompositeId) -> Result<()>
    {
        // TODO: Insert entry into the `SchedulingEnvironment`
        let build_dependencies = self.extract_factory_dependencies(id.asset())?;

        let communication = <DailyApi as ActorFactory<Ss>>::construct_actor(
            id.clone(),
            build_dependencies.0,
            build_dependencies.1,
            build_dependencies.2,
            self.state_link_bus.lock().unwrap().add_rx(),
            self.error_sender.clone(),
        )
        .with_context(|| format!("Could not create dailyActor for Asset {}", id.asset()))?;

        self.actor_registries
            .lock()
            .unwrap()
            .get_mut(id.asset())
            .expect("The ActorRegistry for asset should exist before creating Actors on it")
            .daily_agent_senders
            .insert(id.clone(), communication);
        Ok(())
    }

    // TODO: Build operational actor based on SchedulingEnvironment state
    pub fn start_operational_actor(&self, id: &ActorCompositeId) -> Result<(), StartError>
    {
        // TODO: Use typed errors instead of generic error handling
        let build_dependencies = self
            .extract_factory_dependencies(id.asset())
            .map_err(|_e| StartError::CouldNotCreateDependencies)?;

        let communication = <OperationalApi as ActorFactory<Ss>>::construct_actor(
            id.clone(),
            build_dependencies.0,
            build_dependencies.1,
            build_dependencies.2,
            self.state_link_bus.lock().unwrap().add_rx(),
            self.error_sender.clone(),
        )
        .with_context(|| format!("Could not create OperationalActor for Asset {}", id.asset()))
        .map_err(|_e| StartError::CouldNotConstruct)?;

        let mut binding = self.actor_registries.lock().unwrap();
        let hash_map = binding
            .get_mut(id.asset())
            .expect("The ActorRegistry for asset should exist before creating Actors on it");

        if hash_map.operational_agent_senders.contains_key(id) {
            return Err(StartError::AlreadyRunning);
        }

        hash_map
            .operational_agent_senders
            .insert(id.clone(), communication);
        Ok(())
    }
}
