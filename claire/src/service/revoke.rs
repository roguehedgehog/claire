use crate::access::role::RoleRepo;
use crate::instance::tag::TagRepo;
use crate::instance::InstanceRepo;
use crate::service::investigation::{Investigation, InvestigationsService};

use anyhow::{anyhow, bail, Result};
use chrono::Utc;
use serde_json::json;
pub struct RevokeInstancePermissionsService {
    instances: InstanceRepo,
    roles: RoleRepo,
    investigations: InvestigationsService,
    tags: TagRepo,
}

impl RevokeInstancePermissionsService {
    pub fn new(investigation_bucket: &str) -> Self {
        Self {
            instances: InstanceRepo::new(),
            roles: RoleRepo::new(),
            investigations: InvestigationsService::new(investigation_bucket),
            tags: TagRepo::new(),
        }
    }

    pub async fn get_investigation(&self, investigation_id: &str) -> Result<Investigation> {
        self.investigations
            .get_investigation(investigation_id)
            .await
    }

    pub async fn remove_profile(&self, instance_id: &str) -> Result<String> {
        let instance = self.instances.get_instance(instance_id).await?;

        let profile = match instance.iam_instance_profile {
            Some(p) => p
                .arn
                .ok_or(anyhow!("Instance profile does not have an ARN"))?,
            None => return Ok(String::new()),
        };

        let profile_name = &profile[(profile
            .find("/")
            .ok_or(anyhow!("Expected / in profile ARN"))?
            + 1)..];

        let assoc = match self
            .instances
            .get_profile_association(&instance_id)
            .await?
            .association_id
        {
            Some(assoc) => assoc,
            None => bail!("Instance has a profile but could not find association id"),
        };

        self.tags
            .create_tag(instance_id, "claire_removed_profile", &profile_name)
            .await?;
        self.instances.remove_instance_profile(&assoc).await?;

        Ok(profile_name.to_string())
    }

    pub async fn get_roles_by_instance(&self, instance_id: &str) -> Result<Vec<String>> {
        let tags = match self.instances.get_instance(&instance_id).await?.tags {
            Some(tags) => tags,
            None => bail!("Cannot determine previous profile, the instance has no tags."),
        };

        let profile = tags.iter().find(|tag| match tag.key.clone() {
            Some(key) => key == "claire_removed_profile",
            None => false,
        }).ok_or(anyhow!("Cannot determine previous instance profile, tag claire_removed_profile does not exist"))?;

        self.get_roles_by_profile(
            &profile
                .value
                .clone()
                .ok_or(anyhow!("tag clare_remove_profile is None"))?,
        )
        .await
    }

    pub async fn get_roles_by_profile(&self, profile: &str) -> Result<Vec<String>> {
        let role_names = self
            .roles
            .get_roles_for_profile(profile)
            .await?
            .iter()
            .map(|r| r.role_name.clone())
            .collect::<Vec<String>>();

        Ok(role_names)
    }

    pub async fn deny_instance(&self, roles: &Vec<String>, instance_id: &str) -> Result<()> {
        let policy_name = format!("ClareDenyInstance_{}", instance_id);
        let invalidate_tokens = json!({
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Deny",
                "Action": ["*"],
                "Resource": ["*"],
                "Condition": {
                    "StringLike": {
                        "aws:userid": format!("*{}", instance_id)
                    }
                }
            }]
        })
        .to_string();

        for role in roles {
            self.roles
                .add_policy_to_role(role, &policy_name, &invalidate_tokens)
                .await?;
        }

        Ok(())
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
