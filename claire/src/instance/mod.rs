extern crate rusoto_core;
extern crate rusoto_ec2;
extern crate rusoto_lambda;

use crate::instance::tag::Resource;
use anyhow::{anyhow, bail, Result};
use bytes::Bytes;
use rusoto_core::Region;
use rusoto_ec2::{
    AttachVolumeRequest, CreateVolumeRequest, DescribeIamInstanceProfileAssociationsRequest,
    DescribeInstancesRequest, DescribeVolumesRequest, DisassociateIamInstanceProfileRequest, Ec2,
    Ec2Client, Filter, IamInstanceProfileAssociation, Instance, Volume,
};
use rusoto_lambda::{InvocationRequest, Lambda, LambdaClient};
use serde_json::json;

pub mod snapshot;
pub mod tag;

const CREATE_EXTRACTOR_FN_NAME: &str = "claire_manual_create_evidence_extractor";

pub struct InstanceRepo {
    ec2: Ec2Client,
    lambda: LambdaClient,
}

impl InstanceRepo {
    pub fn new() -> Self {
        Self {
            ec2: Ec2Client::new(Region::default()),
            lambda: LambdaClient::new(Region::default()),
        }
    }

    pub async fn new_extractor_instance(
        &self,
        investigation_id: &str,
        key_name: &str,
    ) -> Result<Instance> {
        let payload = json!({
            "investigation_id": investigation_id,
            "key_name": key_name,
        });
        let req = InvocationRequest {
            function_name: CREATE_EXTRACTOR_FN_NAME.to_string(),
            payload: Some(Bytes::from(payload.to_string())),
            ..Default::default()
        };

        let resp = match self.lambda.invoke(req).await?.payload {
            Some(payload) => payload,
            None => bail!("There was no response from the create extractor lambda"),
        };
        let resp: serde_json::Value = serde_json::from_slice(&resp[..])?;
        if let Some(err) = resp.get("errorMessage") {
            bail!("Create instance failed with {}", err.to_string());
        }

        if let Some(id) = resp.get("extractor_id") {
            return self
                .get_instance(
                    id.as_str()
                        .ok_or(anyhow!("Could not convert instance id to str"))?,
                )
                .await;
        }

        bail!("Unexpected response creating instance {}", resp);
    }

    pub async fn get_instance(&self, instance_id: &str) -> Result<Instance> {
        let req = DescribeInstancesRequest {
            instance_ids: Some(vec![instance_id.to_string()]),
            ..Default::default()
        };

        let resp = self.ec2.describe_instances(req).await?;
        let instance = match resp
            .reservations
            .and_then(|vec| vec.into_iter().nth(0))
            .and_then(|r| r.instances)
            .and_then(|vec| vec.into_iter().nth(0))
        {
            Some(instance) => instance,
            None => bail!("Instance {} could not be found.", instance_id),
        };

        Ok(instance)
    }

    pub async fn create_volumes(
        &self,
        instance: &Instance,
        snapshots: &Vec<&Resource>,
    ) -> Result<Vec<Volume>> {
        let mut vols: Vec<Volume> = Vec::with_capacity(snapshots.len());
        for snapshot in snapshots {
            let req = CreateVolumeRequest {
                availability_zone: instance
                    .placement
                    .clone()
                    .and_then(|p| p.availability_zone)
                    .ok_or(anyhow!("Missing availability_zone"))?
                    .clone(),
                snapshot_id: Some(snapshot.id.to_string()),
                ..Default::default()
            };

            vols.push(self.ec2.create_volume(req).await?);
        }

        Ok(vols)
    }

    pub async fn get_volumes(&self, volume_ids: &Vec<String>) -> Result<Vec<Volume>> {
        let req = DescribeVolumesRequest {
            volume_ids: Some(volume_ids.clone()),
            ..Default::default()
        };

        let resp = self.ec2.describe_volumes(req).await?;
        match resp.volumes {
            Some(vols) => Ok(vols),
            None => bail!(
                "No volumes could be found matching {}",
                volume_ids.join(", ")
            ),
        }
    }

    pub async fn attach_volumes(
        &self,
        instance_id: &str,
        devices: Vec<(String, String)>,
    ) -> Result<()> {
        for (device, vol) in devices {
            let req = AttachVolumeRequest {
                instance_id: instance_id.to_string(),
                volume_id: vol,
                device,
                ..Default::default()
            };

            self.ec2.attach_volume(req).await?;
        }

        Ok(())
    }

    pub async fn get_investigation_id(&self, instance_id: &str) -> Result<String> {
        let req = DescribeInstancesRequest {
            instance_ids: Some(vec![instance_id.to_string()]),
            ..Default::default()
        };

        let resp = self.ec2.describe_instances(req).await?;
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

    pub async fn get_profile_association(
        &self,
        instance_id: &str,
    ) -> Result<IamInstanceProfileAssociation> {
        let req = DescribeIamInstanceProfileAssociationsRequest {
            filters: Some(vec![Filter {
                name: Some("instance-id".to_string()),
                values: Some(vec![instance_id.to_string()]),
            }]),
            ..Default::default()
        };

        let resp = self
            .ec2
            .describe_iam_instance_profile_associations(req)
            .await?
            .iam_instance_profile_associations
            .ok_or(anyhow!(
                "No profile associations could be found for instance {}",
                instance_id
            ))?;

        let assoc = resp
            .get(0)
            .ok_or(anyhow!("Could not get first profile association"))?;

        Ok(assoc.clone())
    }

    pub async fn remove_instance_profile(&self, assoc_id: &str) -> Result<()> {
        let req = DisassociateIamInstanceProfileRequest {
            association_id: assoc_id.to_string(),
        };
        self.ec2.disassociate_iam_instance_profile(req).await?;
        Ok(())
    }
}
