use rusoto_core::Region;
use rusoto_iam::{GetInstanceProfileRequest, Iam, IamClient, PutRolePolicyRequest, Role};

use anyhow::Result;

pub struct RoleRepo {
    iam: IamClient,
}

impl RoleRepo {
    pub fn new() -> Self {
        Self {
            iam: IamClient::new(Region::UsEast1),
        }
    }

    pub async fn get_roles_for_profile(&self, profile: &str) -> Result<Vec<Role>> {
        let req = GetInstanceProfileRequest {
            instance_profile_name: profile.to_string(),
            ..Default::default()
        };

        let resp = self.iam.get_instance_profile(req).await?;

        Ok(resp.instance_profile.roles)
    }

    pub async fn add_policy_to_role(
        &self,
        role: &str,
        policy_name: &str,
        policy: &str,
    ) -> Result<()> {
        let req = PutRolePolicyRequest {
            role_name: role.to_string(),
            policy_document: policy.to_string(),
            policy_name: policy_name.to_string(),
        };

        self.iam.put_role_policy(req).await?;

        Ok(())
    }
}
