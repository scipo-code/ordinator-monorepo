use std::collections::HashMap;

use ordinator_operational_actor::messages::OperationalRequestMessage;
use ordinator_operational_actor::messages::OperationalResponseMessage;
use ordinator_orchestrator_actor_traits::Communication;
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
use ordinator_strategic_actor::messages::WeeklyRequestMessage;
use ordinator_strategic_actor::messages::WeeklyResponseMessage;
use ordinator_supervisor_actor::messages::DailyRequestMessage;
use ordinator_supervisor_actor::messages::DailyResponseMessage;
use ordinator_tactical_actor::messages::ProjectRequestMessage;
use ordinator_tactical_actor::messages::ProjectResponseMessage;

pub struct ActorRegistry
{
    pub strategic_agent_sender: Communication<WeeklyRequestMessage, WeeklyResponseMessage>,
    pub tactical_agent_sender: Communication<ProjectRequestMessage, ProjectResponseMessage>,
    pub supervisor_agent_senders:
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
    //     let mut supervisor_statai: Vec<DailyResponseStatus> = vec![];
    //     let mut operational_statai: Vec<OperationalResponseStatus> = vec![];

    //     let strategic = self.strategic_agent_sender.receiver.recv()??;

    //     let strategic_status = if let WeeklyResponseMessage::Status(strategic)
    // = strategic {         strategic
    //     } else {
    //         panic!()
    //     };

    //     let tactical = self.tactical_agent_sender.receiver.recv()??;
    //     let tactical_status = if let ProjectResponseMessage::Status(tactical) =
    // tactical {         tactical
    //     } else {
    //         panic!()
    //     };

    //     for receiver in self.supervisor_agent_senders.iter() {
    //         let supervisor = receiver.1.receiver.recv()??;
    //         if let DailyResponseMessage::Status(supervisor) = supervisor {
    //             supervisor_statai.push(supervisor);
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
    //         strategic_status,
    //         tactical_status,
    //         supervisor_statai,
    //         operational_statai,
    //     };
    //     Ok(agent_status)
    // }
}
