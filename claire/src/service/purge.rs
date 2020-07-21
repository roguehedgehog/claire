use crate::instance::snapshot::SnapshotRepo;
use crate::instance::tag::Resource;
use crate::instance::tag::TagRepo;
use crate::service::clear::ClearInvestigationService;
use crate::storage::bucket::BucketRepo;
use crate::INVESTIGATION_TAG_KEY;

use rusoto_s3::Object;

pub struct PurgeService {
    tag_repo: TagRepo,
    bucket_repo: BucketRepo,
    snapshot_repo: SnapshotRepo,
}

impl PurgeService {
    pub fn new() -> PurgeService {
        PurgeService {
            tag_repo: TagRepo::new(),
            bucket_repo: BucketRepo::new(),
            snapshot_repo: SnapshotRepo::new(),
        }
    }

    pub async fn get_resources_to_purge(
        &self,
        investigation_bucket: &str,
        investigation_id: &str,
    ) -> Result<(Vec<Resource>, Vec<Object>), Box<dyn std::error::Error>> {
        let resources = self
            .tag_repo
            .get_resources(INVESTIGATION_TAG_KEY, investigation_id)
            .await?;
        let evidence = self
            .bucket_repo
            .get_evidence(investigation_bucket, investigation_id)
            .await?;

        Ok((resources, evidence))
    }

    pub async fn purge_resources(
        &self,
        investigation_bucket: &str,
        resources: &Vec<Resource>,
        evidence: &Vec<Object>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        ClearInvestigationService::new()
            .untag_resources(resources)
            .await?;
        self.delete_snapshots(resources).await?;
        self.delete_objects(investigation_bucket, evidence).await?;

        Ok(())
    }

    async fn delete_objects(
        &self,
        investigation_bucket: &str,
        evidence: &Vec<Object>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if evidence.len() == 0 {
            return Ok(());
        }
        self.bucket_repo
            .delete_evidence(investigation_bucket, &evidence)
            .await?;

        Ok(())
    }

    async fn delete_snapshots(
        &self,
        resources: &Vec<Resource>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let snapshots: Vec<String> = resources
            .iter()
            .filter(|r| r.is_snapshot())
            .map(|r| r.id.clone())
            .collect();

        if snapshots.len() == 0 {
            return Ok(());
        }
        self.snapshot_repo.delete_snapshots(&snapshots).await?;

        Ok(())
    }
}
