use crate::instance::tag::{Resource, TagRepo};
use crate::instance::InstanceRepo;
use crate::service::investigation::InvestigationsService;

use anyhow::{anyhow, Result};

pub struct ManualInvestigationService {
    instance: InstanceRepo,
    investigation_service: InvestigationsService,
    tag_repo: TagRepo,
}

impl ManualInvestigationService {
    pub fn new(investigation_bucket: &str) -> Self {
        Self {
            instance: InstanceRepo::new(),
            investigation_service: InvestigationsService::new(investigation_bucket),
            tag_repo: TagRepo::new(),
        }
    }

    pub async fn create_resources(
        &self,
        investigation_id: &str,
        key_name: &str,
    ) -> Result<(String, Vec<String>)> {
        let investigation_id = self
            .investigation_service
            .get_investigation(investigation_id)
            .await?
            .bucket;

        let extractor = self
            .instance
            .new_extractor_instance(&investigation_id, key_name)
            .await?;

        let instance_id = extractor
            .instance_id
            .clone()
            .ok_or(anyhow!("instance_id missing on extractor"))?;

        let resources = self
            .tag_repo
            .get_resources("InvestigationId", &investigation_id)
            .await?;

        let snapshots = resources
            .iter()
            .filter(|r| r.is_snapshot())
            .collect::<Vec<&Resource>>();

        let resp = self.instance.create_volumes(&extractor, &snapshots).await?;
        let vols = resp
            .into_iter()
            .map(|v| {
                v.volume_id
                    .ok_or(anyhow!("volume_id is missing on created volume"))
            })
            .filter_map(Result::ok)
            .collect::<Vec<String>>();

        Ok((instance_id, vols))
    }

    pub async fn is_instance_ready(&self, instance_id: &str) -> Result<bool> {
        let instance = self.instance.get_instance(instance_id).await?;

        let ready = instance
            .state
            .ok_or(anyhow!("Instance is missing state"))?
            .name
            .ok_or(anyhow!("Instance state is missing name"))?
            == "running";

        Ok(ready)
    }

    pub async fn is_volumes_ready(&self, vols: &Vec<String>) -> Result<bool> {
        let vols = self.instance.get_volumes(vols).await?;
        let ready = vols
            .into_iter()
            .map(|v| v.state.ok_or(anyhow!("Volume is missing state")))
            .filter_map(Result::ok)
            .fold(true, |ready, state| ready && state == "available");

        Ok(ready)
    }

    pub async fn attach_volumes(
        &self,
        instance_id: &str,
        volumes: &Vec<String>,
    ) -> Result<Vec<(String, String)>> {
        let mut devices = Vec::with_capacity(volumes.len());
        let device_prefix = "/dev/xvd";
        let first_device = 'f' as u8;
        for (i, vol) in volumes.iter().enumerate() {
            devices.push((
                format!("{}{}", device_prefix, (first_device + i as u8) as char),
                vol.clone(),
            ));
        }

        self.instance
            .attach_volumes(instance_id, devices.clone())
            .await?;

        Ok(devices)
    }

    pub async fn get_ip(&self, instance_id: &str) -> Result<String> {
        self.instance
            .get_instance(instance_id)
            .await?
            .public_ip_address
            .clone()
            .ok_or(anyhow!(
                "instance {} has not been assigned a public ip address",
                instance_id
            ))
    }
}
