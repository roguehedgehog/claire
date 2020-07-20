extern crate rusoto_core;
extern crate rusoto_ec2;

use rusoto_core::Region;
use rusoto_ec2::{
    DeleteTagsRequest, DescribeTagsRequest, DescribeTagsResult, Ec2, Ec2Client, Filter, Tag,
};
use std::collections::HashMap;
pub struct TagRepo {
    client: Ec2Client,
}

impl TagRepo {
    pub fn new() -> TagRepo {
        TagRepo {
            client: Ec2Client::new(Region::default()),
        }
    }

    pub async fn get_resources(
        &self,
        tags: HashMap<&str, &str>,
    ) -> Result<DescribeTagsResult, Box<dyn std::error::Error>> {
        let req = DescribeTagsRequest {
            dry_run: Some(false),
            max_results: Some(1000),
            filters: Some(
                tags.iter()
                    .map(|(k, v)| Filter {
                        name: Some(format!("tag:{}", k)),
                        values: Some(vec![v.to_string()]),
                    })
                    .collect(),
            ),
            ..Default::default()
        };

        Ok(self.client.describe_tags(req).await?)
    }

    pub async fn delete_tags(
        &self,
        resources: Vec<String>,
        tags: Vec<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tags = tags
            .iter()
            .map(|name| Tag {
                key: Some(name.to_string()),
                value: None,
            })
            .collect();

        let req = DeleteTagsRequest {
            resources: resources,
            tags: Some(tags),
            dry_run: Some(false),
        };

        self.client.delete_tags(req).await?;

        Ok(())
    }
}
