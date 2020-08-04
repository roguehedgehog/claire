use crate::execute::state::{InvestigationStatus, StartInvestigationRequest, StateMachineRepo};

use anyhow::Result;

pub struct ExecuteInvestigationService {
    state_machine_repo: StateMachineRepo,
}

impl ExecuteInvestigationService {
    pub fn new() -> Self {
        Self {
            state_machine_repo: StateMachineRepo::new(),
        }
    }

    pub async fn start(&self, instance_id: &str, reason: &str) -> Result<String> {
        let req = StartInvestigationRequest {
            instance_id: instance_id.to_string(),
            reason: reason.to_string(),
        };

        Ok(self.state_machine_repo.start_investigation(&req).await?)
    }

    pub async fn status(&self, execution_arn: &str) -> Result<InvestigationStatus> {
        self.state_machine_repo
            .get_investigation_status(&execution_arn)
            .await
    }
}
