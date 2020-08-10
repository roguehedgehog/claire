use crate::access::role::RoleRepo;
use crate::instance::InstanceRepo;
use crate::service::investigation::InvestigationsService;

use anyhow::{anyhow, bail, Result};
use chrono::Utc;
use serde_json::json;
pub struct InvalidateTokensService {
    instances: InstanceRepo,
    roles: RoleRepo,
    investigations: InvestigationsService,
}

impl InvalidateTokensService {
    pub fn new(investigation_bucket: &str) -> Self {
        Self {
            instances: InstanceRepo::new(),
            roles: RoleRepo::new(),
            investigations: InvestigationsService::new(investigation_bucket),
        }
    }

    pub async fn get_roles(&self, investigation_id: &str) -> Result<(String, Vec<String>)> {
        let investigation = self
            .investigations
            .get_investigation(investigation_id)
            .await?;

        let tags = match self
            .instances
            .get_instance(&investigation.instance_id)
            .await?
            .tags
        {
            Some(tags) => tags,
            None => bail!("Cannot determine previous profile, the instance has no tags."),
        };

        let profile = tags.iter().find(|tag| match tag.key.clone() {
            Some(key) => key == "claire_removed_profile",
            None => false,
        }).ok_or(anyhow!("Cannot determine previous instance profile, tag claire_removed_profile does not exist"))?;

        let role_names = self
            .roles
            .get_roles_for_profile(
                &profile
                    .value
                    .clone()
                    .ok_or(anyhow!("tag clare_remove_profile is None"))?,
            )
            .await?
            .iter()
            .map(|r| r.role_name.clone())
            .collect::<Vec<String>>();

        Ok((investigation.bucket, role_names))
    }

    pub async fn invalidate_tokens(&self, roles: &Vec<String>) -> Result<()> {
        let now = Utc::now().format("%FT%T.%fZ").to_string();
        let policy_name = "ClaireInvalidateTokens";
        let invalidate_tokens = json!({
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Deny",
                "Action": ["*"],
                "Resource": ["*"],
                "Condition": {
                    "DateLessThan": {
                        "aws:TokenIssueTime": now
                    }
                }
            }]
        })
        .to_string();

        for role in roles {
            self.roles
                .add_policy_to_role(role, policy_name, &invalidate_tokens)
                .await?;
        }

        Ok(())
    }
}
