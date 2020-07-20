extern crate rusoto_core;
extern crate rusoto_s3;

use rusoto_core::Region;
use rusoto_s3::{
    Delete, DeleteObjectsRequest, ListObjectsV2Output, ListObjectsV2Request, ObjectIdentifier,
    S3Client, S3,
};

pub struct BucketRepo {
    client: S3Client,
}

impl BucketRepo {
    pub fn new() -> BucketRepo {
        BucketRepo {
            client: S3Client::new(Region::default()),
        }
    }

    pub async fn get_evidence(
        &self,
        investigation_bucket: &str,
        investigation_id: &str,
    ) -> Result<ListObjectsV2Output, Box<dyn std::error::Error>> {
        let req = ListObjectsV2Request {
            bucket: investigation_bucket.to_string(),
            prefix: Some(investigation_id.to_string()),
            ..Default::default()
        };

        Ok(self.client.list_objects_v2(req).await?)
    }

    pub async fn delete_evidence(
        &self,
        investigation_bucket: &str,
        resources: &Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let req = DeleteObjectsRequest {
            bucket: investigation_bucket.to_string(),
            delete: Delete {
                objects: resources
                    .iter()
                    .map(|id| ObjectIdentifier {
                        key: id.clone(),
                        version_id: None,
                    })
                    .collect(),
                ..Default::default()
            },
            ..Default::default()
        };

        self.client.delete_objects(req).await?;

        Ok(())
    }
}
