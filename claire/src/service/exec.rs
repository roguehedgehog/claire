use crate::execute::{InvestigationStatus, StartInvestigationRequest, StateMachineRepo};
use crate::instance::InstanceRepo;
use crate::storage::BucketRepo;

use anyhow::{bail, Result};

pub struct ExecutionDetails {
    pub execution_arn: String,
    pub investigation_id: String,
}

pub struct ExecuteInvestigationService {
    state_machine_repo: StateMachineRepo,
    instance_repo: InstanceRepo,
    bucket_repo: BucketRepo,
}

impl ExecuteInvestigationService {
    pub fn new(investigation_bucket: &str) -> Self {
        Self {
            state_machine_repo: StateMachineRepo::new(),
            instance_repo: InstanceRepo::new(),
            bucket_repo: BucketRepo::new(investigation_bucket),
        }
    }

    pub async fn start(&self, instance_id: &str, reason: &str, isolate: bool) -> Result<String> {
        let req = StartInvestigationRequest {
            instance_id: instance_id.to_string(),
            reason: reason.to_string(),
            isolate,
        };

        Ok(self.state_machine_repo.start_investigation(&req).await?)
    }

    pub async fn execution_detials(&self, instance_id: &str) -> Result<ExecutionDetails> {
        let investigation_id = match self.get_investigation_id(instance_id).await {
            Some(id) => id,
            None => bail!(
                "The investigation for instance {} could not be found",
                instance_id
            ),
        };

        let tags = self.bucket_repo.get_alert_tags(&investigation_id).await?;
        if let Some(execution_arn) = tags
            .into_iter()
            .find(|t| t.key == "CLAIRE_EXEC")
            .and_then(|t| Some(t.value))
        {
            return Ok(ExecutionDetails {
                execution_arn,
                investigation_id,
            });
        } else {
            bail!("Excution ARN could not be found on alert.json")
        }
    }

    pub async fn get_investigation_id(&self, instance_id: &str) -> Option<String> {
        return match self.instance_repo.get_investigation_id(instance_id).await {
            Ok(id) => Some(id),
            Err(_) => None,
        };
    }

    pub async fn last_update(&self, execution_arn: &str) -> Result<InvestigationStatus> {
        self.state_machine_repo
            .get_investigation_status(&execution_arn)
            .await
    }
}
