use crate::repository::bucket::BucketRepo;
use crate::repository::instance::InstanceRepo;
use crate::repository::tagged::TagRepo;
use std::collections::HashMap;

pub struct PurgeService {
    tag_repo: TagRepo,
    bucket_repo: BucketRepo,
    instance_repo: InstanceRepo,
}

impl PurgeService {
    pub fn new() -> PurgeService {
        PurgeService {
            tag_repo: TagRepo::new(),
            bucket_repo: BucketRepo::new(),
            instance_repo: InstanceRepo::new(),
        }
    }

    pub async fn get_resources_to_purge(
        &self,
        investigation_bucket: &str,
        investigation_id: &str,
    ) -> Result<ResourceCollection, Box<dyn std::error::Error>> {
        let tags: HashMap<&str, &str> = [("InvestigationId", investigation_id)]
            .iter()
            .cloned()
            .collect();

        let tagged_resources = self.tag_repo.get_resources(tags).await?;
        let evidence = self
            .bucket_repo
            .get_evidence(investigation_bucket, investigation_id)
            .await?;

        let mut col = ResourceCollection { resources: vec![] };
        if let Some(tags) = tagged_resources.tags {
            tags.iter().for_each(|tag| {
                col.resources.push(Resource {
                    category: tag
                        .resource_type
                        .clone()
                        .unwrap_or("resource type not found".to_string()),
                    id: tag
                        .resource_id
                        .clone()
                        .unwrap_or("resource id not found".to_string()),
                });
            });
        }

        if let Some(objects) = evidence.contents {
            objects.iter().for_each(|o| {
                col.resources.push(Resource {
                    category: "object".to_string(),
                    id: o.key.clone().unwrap_or("empty".to_string()),
                })
            })
        }

        Ok(col)
    }

    pub async fn purge_resources(
        &self,
        investigation_bucket: &str,
        coll: &ResourceCollection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.untag_resources(&coll).await?;
        self.delete_objects(investigation_bucket, &coll).await?;
        self.delete_snapshots(&coll).await?;

        Ok(())
    }

    async fn untag_resources(
        &self,
        coll: &ResourceCollection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let resources = coll
            .resources
            .iter()
            .filter(|r| !r.is_object())
            .map(|r| r.id.clone())
            .collect::<Vec<String>>();

        if resources.len() == 0 {
            return Ok(());
        }

        self.tag_repo
            .delete_tags(resources, vec!["CLAIRE", "InvestigationId"])
            .await?;

        Ok(())
    }

    async fn delete_objects(
        &self,
        investigation_bucket: &str,
        coll: &ResourceCollection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let resources: Vec<String> = coll
            .resources
            .iter()
            .filter(|r| r.is_object())
            .map(|r| r.id.clone())
            .collect();

        if resources.len() == 0 {
            return Ok(());
        }
        self.bucket_repo
            .delete_evidence(investigation_bucket, &resources)
            .await?;

        Ok(())
    }

    async fn delete_snapshots(
        &self,
        coll: &ResourceCollection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let snapshots: Vec<String> = coll
            .resources
            .iter()
            .filter(|r| r.is_snapshot())
            .map(|r| r.id.clone())
            .collect();

        if snapshots.len() == 0 {
            return Ok(());
        }
        self.instance_repo.delete_snapshots(&snapshots).await?;

        Ok(())
    }
}
#[derive(Clone)]
pub struct Resource {
    category: String,
    id: String,
}

impl Resource {
    fn is_deletable(&self) -> bool {
        self.category == "snapshot" || self.is_object()
    }

    fn is_object(&self) -> bool {
        self.category == "object"
    }

    fn is_snapshot(&self) -> bool {
        self.category == "snapshot"
    }
}

#[derive(Clone)]
pub struct ResourceCollection {
    resources: Vec<Resource>,
}

impl std::fmt::Display for ResourceCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let mut resources = self.resources.clone();
        resources.sort_by(|a, b| a.is_deletable().cmp(&b.is_deletable()));
        writeln!(f, "These changes will be made:\n").expect("The text could not be written.");
        resources.iter().for_each(|r| {
            writeln!(
                f,
                "{:>10} {:>10} - {}",
                if r.is_deletable() { "Delete" } else { "Untag" },
                r.category,
                r.id
            )
            .expect("The value could not be written")
        });

        Ok(())
    }
}

impl ResourceCollection {
    pub fn is_empty(&self) -> bool {
        self.resources.len() == 0
    }
}
