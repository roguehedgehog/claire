extern crate rusoto_core;
extern crate rusoto_ec2;

use anyhow::{bail, Result};
use rusoto_core::Region;
use rusoto_ec2::{DescribeInstancesRequest, Ec2, Ec2Client};

pub mod snapshot;
pub mod tag;

pub struct InstanceRepo {
    client: Ec2Client,
}

impl InstanceRepo {
    pub fn new() -> Self {
        Self {
            client: Ec2Client::new(Region::default()),
        }
    }

    pub async fn get_investigation_id(&self, instance_id: &str) -> Result<String> {
        let req = DescribeInstancesRequest {
            instance_ids: Some(vec![instance_id.to_string()]),
            ..Default::default()
        };

        let resp = self.client.describe_instances(req).await?;
        let tags = match resp
            .reservations
            .and_then(|vec| vec.into_iter().nth(0))
            .and_then(|r| r.instances)
            .and_then(|vec| vec.into_iter().nth(0))
            .and_then(|i| i.tags)
        {
            Some(tags) => tags,
            None => bail!("The instance {} is not tagged", instance_id),
        };

        for tag in &tags {
            if tag.key == Some("InvestigationId".to_string()) {
                if let Some(id) = &tag.value {
                    return Ok(id.to_string());
                } else {
                    bail!("The investigation id is not set")
                }
            }
        }

        bail!("The investigation for {} could not be found", instance_id);
    }
}
