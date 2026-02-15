pub mod status_codes;

use serde::Deserialize;
use serde::Serialize;
use status_codes::MaterialStatus;
use status_codes::SystemStatusCodes;
use status_codes::SystemStatusCodesBuilder;
use status_codes::UserStatusCodes;
use status_codes::UserStatusCodesBuilder;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkOrderAnalytic
{
    pub(crate) system_status_codes: SystemStatusCodes,
    pub(crate) user_status_codes: UserStatusCodes,
}

pub struct WorkOrderAnalyticBuilder
{
    system_status_codes: Option<SystemStatusCodes>,
    user_status_codes: Option<UserStatusCodes>,
}

impl WorkOrderAnalyticBuilder
{
    pub fn build(self) -> WorkOrderAnalytic
    {
        WorkOrderAnalytic {
            // FIXME: Design system so fixed values are calculated from application parameters
            // rather than hardcoded, as they may vary between models
            system_status_codes: self
                .system_status_codes
                .expect("Check that all builder steps are followed"),
            user_status_codes: self
                .user_status_codes
                .expect("Check that the code is actually created correctly"),
        }
    }

    pub fn system_status_codes<F>(mut self, configure: F) -> Self
    where
        F: FnOnce(SystemStatusCodesBuilder) -> SystemStatusCodesBuilder,
    {
        let system_status_codes_builder = SystemStatusCodes::builder();

        let configured_systes_codes_builder = configure(system_status_codes_builder);

        self.system_status_codes = Some(configured_systes_codes_builder.build());
        self
    }

    pub fn user_status_codes<F>(mut self, configure: F) -> Self
    where
        F: FnOnce(UserStatusCodesBuilder) -> UserStatusCodesBuilder,
    {
        let user_status_codes_builder = UserStatusCodes::builder();

        let configured_user_status_codes_builder = configure(user_status_codes_builder);

        self.user_status_codes = Some(configured_user_status_codes_builder.build());
        self
    }
}

// Builders are used to ease testing despite reading data from external sources
impl WorkOrderAnalytic
{
    pub fn builder() -> WorkOrderAnalyticBuilder
    {
        let system_status_codes = Some(SystemStatusCodes::builder().build());
        let user_status_codes = Some(UserStatusCodes::builder().build());

        WorkOrderAnalyticBuilder {
            system_status_codes,
            user_status_codes,
        }
    }

    // TODO: Implement using status codes and material statuses
    pub fn fixed(&self) -> bool
    {
        todo!(
            "This could be a very important function to implement. Status codes should ideally express this together with material Statuses."
        )
    }

    pub fn awaiting_scheduling(&self) -> bool
    {
        self.user_status_codes.awsc
    }

    pub fn scheduled(&self) -> bool
    {
        self.user_status_codes.sch
    }


    pub fn material(&self) -> MaterialStatus
    {
        MaterialStatus::from(&self.user_status_codes)
    }
}
