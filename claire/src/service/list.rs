use crate::storage::bucket::BucketRepo;
use chrono::NaiveDateTime;
use failure::bail;
#[derive(Debug)]
pub struct Investigation {
    instance_id: String,
    dt: NaiveDateTime,
    pub bucket: String,
}

impl Investigation {
    pub fn new(investigation_id: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let details = investigation_id.split("_").collect::<Vec<&str>>();
        let dt = details[0];
        let instance_id = details[1];
        Ok(Self {
            bucket: investigation_id.to_string(),
            dt: NaiveDateTime::parse_from_str(&dt, "%F %T.%f")?,
            instance_id: instance_id.to_string(),
        })
    }
}

pub struct ListInvestigationsService {
    bucket_repo: BucketRepo,
}

impl ListInvestigationsService {
    pub fn new() -> Self {
        Self {
            bucket_repo: BucketRepo::new(),
        }
    }

    pub async fn get_investigations(
        &self,
        investigation_bucket: &str,
    ) -> Result<Vec<Investigation>, Box<dyn std::error::Error>> {
        let dirs = self
            .bucket_repo
            .get_investigations(investigation_bucket)
            .await?;

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
}
