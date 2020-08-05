use crate::storage::bucket::BucketRepo;
use anyhow::{bail, Result};
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
            dt: NaiveDateTime::parse_from_str(&dt, "%F.%T")?,
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

        Ok(dirs
            .iter()
            .map(|o| match &o.prefix {
                Some(key) => Ok(&key[0..key.len() - 1]),
                None => bail!("Prefix is missing a name"),
            })
            .map(|investigation_id| Investigation::new(&investigation_id?))
            .filter_map(Result::ok)
            .collect())
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
