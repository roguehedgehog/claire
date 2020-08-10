extern crate rusoto_core;
extern crate rusoto_stepfunctions;

use anyhow::{bail, Result};
use rusoto_core::Region;
use rusoto_stepfunctions::{
    GetExecutionHistoryInput, ListStateMachinesInput, StartExecutionInput, StateMachineListItem,
    StepFunctions, StepFunctionsClient,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const CLAIRE_STATE_MACHINE: &str = "claire_investigation";

#[derive(Serialize, Deserialize)]
pub struct StartInvestigationRequest {
    pub instance_id: String,
    pub reason: String,
    pub isolate: bool,
}

impl StartInvestigationRequest {
    pub fn as_json(&self) -> String {
        return json!({
            "reason": self.reason,
            "detail": {
                "resource": {
                    "resourceType": "Instance",
                    "instanceDetails": {
                        "instanceId": self.instance_id,
                    }
                },
                "severity": if self.isolate { 10 } else { 0 },
            },
        })
        .to_string();
    }
}

pub struct InvestigationStatus {
    pub execution_arn: String,
    pub status: String,
    pub details: Value,
}

impl InvestigationStatus {
    pub fn get_task_name(&self) -> Result<String> {
        let details = match self.details.as_object() {
            Some(d) => d,
            None => bail!("There are no investigation details"),
        };

        for (key, val) in details {
            if key.ends_with("EventDetails") {
                return Ok(match val.get("name") {
                    Some(name) => name.as_str().unwrap_or("").to_string(),
                    None => String::new(),
                });
            }
        }

        Ok(String::new())
    }
}

pub struct StateMachineRepo {
    client: StepFunctionsClient,
}

impl StateMachineRepo {
    pub fn new() -> Self {
        Self {
            client: StepFunctionsClient::new(Region::default()),
        }
    }

    pub async fn start_investigation(&self, req: &StartInvestigationRequest) -> Result<String> {
        let machine = self.get_investigation_machine().await?;
        let resp = self
            .client
            .start_execution(StartExecutionInput {
                state_machine_arn: machine.state_machine_arn,
                input: Some(req.as_json()),
                ..Default::default()
            })
            .await?;

        Ok(resp.execution_arn)
    }

    pub async fn get_investigation_status(
        &self,
        execution_arn: &str,
    ) -> Result<InvestigationStatus> {
        let resp = self
            .client
            .get_execution_history(GetExecutionHistoryInput {
                execution_arn: execution_arn.to_string(),
                reverse_order: Some(true),
                ..Default::default()
            })
            .await?;

        let event = match resp.events.first() {
            Some(event) => event,
            None => bail!("There are no events"),
        };

        let details = serde_json::to_value(event)?;
        Ok(InvestigationStatus {
            execution_arn: execution_arn.to_string(),
            status: event.type_.clone(),
            details,
        })
    }

    async fn get_investigation_machine(&self) -> Result<StateMachineListItem> {
        let machines = self
            .client
            .list_state_machines(ListStateMachinesInput::default())
            .await?
            .state_machines;

        for machine in &machines {
            if machine.name == CLAIRE_STATE_MACHINE {
                return Ok(machine.clone());
            }
        }

        bail!("CLAIRE investigation state machine could not be found");
    }
}
