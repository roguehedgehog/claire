use crate::access::group::SecurityGroupRepo;
use crate::instance::tag::TagRepo;
use crate::instance::InstanceRepo;
use crate::service::investigation::{Investigation, InvestigationsService};
use log::info;
use rusoto_ec2::Instance;

use anyhow::{anyhow, Result};
pub struct IsolateInstanceService {
    instances: InstanceRepo,
    security_groups: SecurityGroupRepo,
    investigations: InvestigationsService,
    tags: TagRepo,
}

impl IsolateInstanceService {
    pub fn new(investigation_bucket: &str) -> Self {
        Self {
            instances: InstanceRepo::new(),
            security_groups: SecurityGroupRepo::new(),
            investigations: InvestigationsService::new(investigation_bucket),
            tags: TagRepo::new(),
        }
    }

    pub async fn isolate(&self, investigation_id: &str) -> Result<Investigation> {
        let investigation = self
            .investigations
            .get_investigation(investigation_id)
            .await?;

        let instance = self
            .instances
            .get_instance(&investigation.instance_id)
            .await?;

        self.apply_security_group(&instance).await?;

        Ok(investigation)
    }

    async fn apply_security_group(&self, instance: &Instance) -> Result<()> {
        let instance = instance.clone();
        let instance_id = instance.instance_id.ok_or(anyhow!("Instance missing id"))?;
        let vpc_id = instance.vpc_id.ok_or(anyhow!("Instance missing vpc id"))?;
        let locked_sg = self
            .security_groups
            .get_group(&vpc_id, "claire_locked_down")
            .await?
            .group_id
            .ok_or(anyhow!(
                "claire_locked_down security group does not have an id"
            ))?;
        if let Some(groups) = instance.security_groups {
            info!(
                "Saving existing security groups to tag claire_removed_groups on {}",
                instance_id
            );
            self.tags
                .create_tag(
                    &instance_id,
                    "claire_removed_groups",
                    &groups
                        .iter()
                        .map(|g| g.group_id.clone().ok_or(anyhow!("Group is missing id")))
                        .filter_map(Result::ok)
                        .collect::<Vec<String>>()
                        .join(","),
                )
                .await?;
        }

        info!("Applying Security Group {} to {}", locked_sg, instance_id);
        self.security_groups
            .set_group(&instance_id, &locked_sg)
            .await?;

        Ok(())
    }
}
