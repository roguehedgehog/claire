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

    pub async fn download(&self, investigation_id: &str, dest: &str) -> Result<()> {
        let investigation = self
            .investigation_service
            .get_investigation(investigation_id)
            .await?;

        Command::new("aws")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .arg("s3")
            .arg("sync")
            .arg(format!(
                "s3://{}/{}",
                self.investigation_bucket, investigation.bucket
            ))
            .arg(format!("{}/{}/", dest, investigation.bucket))
            .spawn()
            .with_context(|| "Failed to use awscli to download investigation assets")?
            .wait()?;

        Ok(())
    }
}
