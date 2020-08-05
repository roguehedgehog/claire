extern crate rusoto_core;
extern crate rusoto_s3;

use anyhow::Result;
use rusoto_core::Region;
use rusoto_s3::{
    CommonPrefix, Delete, DeleteObjectsRequest, ListObjectsV2Request, Object, ObjectIdentifier,
    S3Client, S3,
};

pub struct BucketRepo {
    bucket: String,
    client: S3Client,
}

impl BucketRepo {
    pub fn new(investigation_bucket: &str) -> BucketRepo {
        BucketRepo {
            bucket: investigation_bucket.to_string(),
            client: S3Client::new(Region::default()),
        }
    }

    pub async fn get_investigations(&self, prefix: Option<String>) -> Result<Vec<CommonPrefix>> {
        let req = ListObjectsV2Request {
            bucket: self.bucket.clone(),
            delimiter: Some("/".to_string()),
            prefix,
            ..Default::default()
        };

        Ok(self
            .client
            .list_objects_v2(req)
            .await?
            .common_prefixes
            .unwrap_or(vec![]))
    }

    pub async fn get_evidence(&self, investigation_id: &str) -> Result<Vec<Object>> {
        let req = ListObjectsV2Request {
            bucket: self.bucket.clone(),
            prefix: Some(investigation_id.to_string()),
            ..Default::default()
        };

        Ok(self
            .client
            .list_objects_v2(req)
            .await?
            .contents
            .unwrap_or(vec![]))
    }

    pub async fn delete_evidence(&self, evidence: &Vec<Object>) -> Result<()> {
        let req = DeleteObjectsRequest {
            bucket: self.bucket.clone(),
            delete: Delete {
                objects: evidence
                    .iter()
                    .map(|o| ObjectIdentifier {
                        key: o.key.clone().unwrap(),
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
