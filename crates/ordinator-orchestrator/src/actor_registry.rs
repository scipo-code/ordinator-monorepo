use std::collections::HashMap;

use ordinator_actor_operational::messages::OperationalRequestMessage;
use ordinator_actor_operational::messages::OperationalResponseMessage;
use ordinator_orchestrator_actor_traits::Communication;
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
use ordinator_actor_weekly::messages::WeeklyRequestMessage;
use ordinator_actor_weekly::messages::WeeklyResponseMessage;
use ordinator_actor_daily::messages::DailyRequestMessage;
use ordinator_actor_daily::messages::DailyResponseMessage;
use ordinator_actor_project::messages::ProjectRequestMessage;
use ordinator_actor_project::messages::ProjectResponseMessage;

pub struct ActorRegistry
{
    pub weekly_agent_sender: Communication<WeeklyRequestMessage, WeeklyResponseMessage>,
    pub project_agent_sender: Communication<ProjectRequestMessage, ProjectResponseMessage>,
    pub daily_agent_senders:
        HashMap<ActorCompositeId, Communication<DailyRequestMessage, DailyResponseMessage>>,
    pub operational_agent_senders:
        HashMap<ActorCompositeId, Communication<OperationalRequestMessage, OperationalResponseMessage>>,
}

impl ActorRegistry
{
    pub fn get_operational_addr(
        &self,
        operational_id: &String,
    ) -> Option<&Communication<OperationalRequestMessage, OperationalResponseMessage>>
    {
        self.operational_agent_senders
            .iter()
            .find(|(id, _)| &id.0 == operational_id)
            .map(|(_, addr)| addr)
    }

    // TODO: Make this function generic over message types. The outer message
    // type should be consistent across agents, possibly a `Status` type.
    // Note: Genericization may introduce complexity; consider design impact.
    // pub fn recv_all_agents_status(&self) -> Result<AgentStatus> {
    //     let mut daily_statai: Vec<DailyResponseStatus> = vec![];
    //     let mut operational_statai: Vec<OperationalResponseStatus> = vec![];

    //     let weekly = self.weekly_agent_sender.receiver.recv()??;

    //     let weekly_status = if let WeeklyResponseMessage::Status(weekly)
    // = weekly {         weekly
    //     } else {
    //         panic!()
    //     };

    //     let project = self.project_agent_sender.receiver.recv()??;
    //     let project_status = if let ProjectResponseMessage::Status(project) =
    // project {         project
    //     } else {
    //         panic!()
    //     };

    //     for receiver in self.daily_agent_senders.iter() {
    //         let daily = receiver.1.receiver.recv()??;
    //         if let DailyResponseMessage::Status(daily) = daily {
    //             daily_statai.push(daily);
    //         } else {
    //             panic!()
    //         }
    //     }
    //     for receiver in self.operational_agent_senders.iter() {
    //         let operational = receiver.1.receiver.recv()??;

    //         if let OperationalResponseMessage::Status(operational) = operational
    // {             operational_statai.push(operational);
    //         } else {
    //             panic!()
    //         }
    //     }

    //     // I am not sure that this is what we want
    //     let agent_status = AgentStatus {
    //         weekly_status,
    //         project_status,
    //         daily_statai,
    //         operational_statai,
    //     };
    //     Ok(agent_status)
    // }
}
