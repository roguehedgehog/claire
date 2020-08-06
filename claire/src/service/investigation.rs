use crate::storage::bucket::BucketRepo;
use anyhow::{anyhow, bail, Context, Result};
use chrono::NaiveDateTime;
#[derive(Debug, Clone)]
pub struct Investigation {
    instance_id: String,
    dt: NaiveDateTime,
    pub bucket: String,
}

impl Investigation {
    pub fn new(investigation_id: &str) -> Result<Self> {
        let details = investigation_id.split("_").collect::<Vec<&str>>();
        let dt = details[0];
        let instance_id = details[1];
        Ok(Self {
            bucket: investigation_id.to_string(),
            dt: NaiveDateTime::parse_from_str(&dt, "%F.%T")
                .with_context(|| format!("Could not convert key name {} to date", dt))?,
            instance_id: instance_id.to_string(),
        })
    }
}

pub struct InvestigationsService {
    bucket_repo: BucketRepo,
}

impl InvestigationsService {
    pub fn new(investigation_bucket: &str) -> Self {
        Self {
            bucket_repo: BucketRepo::new(investigation_bucket),
        }
    }

    pub async fn get_investigations(&self, prefix: Option<String>) -> Result<Vec<Investigation>> {
        let dirs = self.bucket_repo.get_investigations(prefix).await?;
        let prefix = dirs
            .iter()
            .map(|o| o.prefix.clone().ok_or(anyhow!("The prefix is missing")))
            .collect::<Result<Vec<String>>>()?;

        prefix
            .iter()
            .map(|investigation_id| {
                Investigation::new(&investigation_id[0..investigation_id.len() - 1])
            })
            .collect::<Result<Vec<Investigation>>>()
    }

    pub async fn get_investigation(&self, prefix: &str) -> Result<Investigation> {
        let investigations = self.get_investigations(Some(prefix.to_string())).await?;

        if investigations.len() > 1 {
            let ids = investigations
                .into_iter()
                .map(|i| i.bucket)
                .collect::<Vec<String>>()
                .join("\n");

            bail!(
                "There are multiple investigations which match {}:\n{}",
                prefix,
                ids
            );
        }

        Ok(match investigations.into_iter().nth(0) {
            Some(i) => i,
            None => bail!("Could not find investigation by {}", prefix),
        })
    }
}
