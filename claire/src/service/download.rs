use super::investigation::InvestigationsService;
use anyhow::{Context, Result};
use std::process::{Command, Stdio};

pub struct DownloadService {
    investigation_service: InvestigationsService,
    investigation_bucket: String,
}

impl DownloadService {
    pub fn new(bucket: &str) -> Self {
        Self {
            investigation_service: InvestigationsService::new(bucket),
            investigation_bucket: bucket.to_string(),
        }
    }

    pub async fn download(
        &self,
        investigation_id: &str,
        dest: &str,
        skip_memory: bool,
    ) -> Result<()> {
        let investigation = self
            .investigation_service
            .get_investigation(investigation_id)
            .await?;

        let mut cmd = Command::new("aws");
        cmd.stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .arg("s3")
            .arg("sync")
            .arg(format!(
                "s3://{}/{}",
                self.investigation_bucket, investigation.bucket
            ))
            .arg(format!("{}/{}/", dest, investigation.bucket));

        if skip_memory {
            cmd.arg("--exclude").arg("*/memory.lime.compressed");
        }

        cmd.spawn()
            .with_context(|| "Failed to use awscli to download investigation assets")?
            .wait()?;

        Ok(())
    }
}
