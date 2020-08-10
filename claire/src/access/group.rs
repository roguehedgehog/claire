use anyhow::{anyhow, bail, Result};
use log::info;
use rusoto_core::Region;
use rusoto_ec2::{
    DescribeSecurityGroupsRequest, Ec2, Ec2Client, Filter, ModifyInstanceAttributeRequest,
    SecurityGroup,
};
pub struct SecurityGroupRepo {
    ec2: Ec2Client,
}

impl SecurityGroupRepo {
    pub fn new() -> Self {
        Self {
            ec2: Ec2Client::new(Region::default()),
        }
    }

    pub async fn get_group(&self, vpc_id: &str, name: &str) -> Result<SecurityGroup> {
        let req = DescribeSecurityGroupsRequest {
            filters: Some(vec![
                Filter {
                    name: Some("vpc-id".to_string()),
                    values: Some(vec![vpc_id.to_string()]),
                },
                Filter {
                    name: Some("group-name".to_string()),
                    values: Some(vec![name.to_string()]),
                },
            ]),
            ..Default::default()
        };

        info!("{:?}", req);

        let groups = self
            .ec2
            .describe_security_groups(req)
            .await?
            .security_groups
            .ok_or(anyhow!("No security groups matched {}", name))?;

        if groups.len() > 1 {
            bail!(
                "Found more than one security group called {}: {:?}",
                name,
                groups,
            );
        }

        match groups.get(0) {
            Some(group) => Ok(group.clone()),
            None => bail!("First security group is not set"),
        }
    }

    pub async fn set_group(&self, instance_id: &str, group_id: &str) -> Result<()> {
        let req = ModifyInstanceAttributeRequest {
            instance_id: instance_id.to_string(),
            groups: Some(vec![group_id.to_string()]),
            ..Default::default()
        };

        self.ec2.modify_instance_attribute(req).await?;

        Ok(())
    }
}
