use crate::instance::Resource;
use crate::instance::SnapshotRepo;
use crate::instance::TagRepo;
use crate::service::clear::ClearInvestigationService;
use crate::service::investigation::InvestigationsService;
use crate::storage::BucketRepo;
use crate::INVESTIGATION_TAG_KEY;
use anyhow::Result;
use futures::try_join;

use rusoto_s3::Object;

pub struct PurgeService {
    tag_repo: TagRepo,
    bucket_repo: BucketRepo,
    snapshot_repo: SnapshotRepo,
    investigation_service: InvestigationsService,
}

impl PurgeService {
    pub fn new(investigation_bucket: &str) -> PurgeService {
        PurgeService {
            tag_repo: TagRepo::new(),
            bucket_repo: BucketRepo::new(investigation_bucket),
            snapshot_repo: SnapshotRepo::new(),
            investigation_service: InvestigationsService::new(investigation_bucket),
        }
    }

    pub async fn get_resources_to_purge(
        &self,
        investigation_id: &str,
    ) -> Result<(Vec<Resource>, Vec<Object>)> {
        let investigation_id = self
            .investigation_service
            .get_investigation(investigation_id)
            .await?
            .bucket;

        let resources = self
            .tag_repo
            .get_resources(INVESTIGATION_TAG_KEY, &investigation_id)
            .await?;
        let evidence = self.bucket_repo.get_evidence(&investigation_id).await?;

        Ok((resources, evidence))
    }

    pub async fn purge_resources(
        &self,
        resources: &Vec<Resource>,
        evidence: &Vec<Object>,
    ) -> Result<()> {
        let service = ClearInvestigationService::new();
        let clear = service.untag_resources(resources);
        let rm_snapshots = self.delete_snapshots(resources);
        let rm_objects = self.delete_objects(evidence);

        try_join!(clear, rm_snapshots, rm_objects)?;

        Ok(())
    }

    async fn delete_objects(&self, evidence: &Vec<Object>) -> Result<()> {
        if evidence.len() == 0 {
            return Ok(());
        }
        self.bucket_repo.delete_evidence(&evidence).await?;

        Ok(())
    }

    async fn delete_snapshots(&self, resources: &Vec<Resource>) -> Result<()> {
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
