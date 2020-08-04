extern crate rusoto_core;
extern crate rusoto_s3;

use anyhow::Result;
use rusoto_core::Region;
use rusoto_s3::{
    CommonPrefix, Delete, DeleteObjectsRequest, ListObjectsV2Request, Object, ObjectIdentifier,
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

    pub async fn get_investigations(
        &self,
        investigation_bucket: &str,
    ) -> Result<Vec<CommonPrefix>> {
        let req = ListObjectsV2Request {
            bucket: investigation_bucket.to_string(),
            delimiter: Some("/".to_string()),
            //prefix: Some("2".to_string()),
            ..Default::default()
        };

        Ok(self
            .client
            .list_objects_v2(req)
            .await?
            .common_prefixes
            .unwrap_or(vec![]))
    }

    pub async fn get_evidence(
        &self,
        investigation_bucket: &str,
        investigation_id: &str,
    ) -> Result<Vec<Object>> {
        let req = ListObjectsV2Request {
            bucket: investigation_bucket.to_string(),
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

    pub async fn delete_evidence(
        &self,
        investigation_bucket: &str,
        evidence: &Vec<Object>,
    ) -> Result<()> {
        let req = DeleteObjectsRequest {
            bucket: investigation_bucket.to_string(),
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
