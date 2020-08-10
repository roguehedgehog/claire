extern crate rusoto_core;
extern crate rusoto_ec2;
use anyhow::Result;
use rusoto_core::Region;
use rusoto_ec2::{
    CreateTagsRequest, DeleteTagsRequest, DescribeTagsRequest, Ec2, Ec2Client, Filter, Tag,
    TagDescription,
};

pub struct TagRepo {
    client: Ec2Client,
}

impl TagRepo {
    pub fn new() -> TagRepo {
        TagRepo {
            client: Ec2Client::new(Region::default()),
        }
    }

    pub async fn get_resources(&self, tag_key: &str, tag_value: &str) -> Result<Vec<Resource>> {
        let req = DescribeTagsRequest {
            filters: Some(vec![Filter {
                name: Some(format!("tag:{}", tag_key)),
                values: Some(vec![tag_value.to_string()]),
            }]),
            ..Default::default()
        };

        let mut resources = vec![];
        if let Some(tags) = self.client.describe_tags(req).await?.tags {
            for tag in &tags {
                resources.push(Resource::from_tag_description(tag));
            }
        }

        Ok(resources)
    }

    pub async fn create_tag(&self, resource_id: &str, key: &str, value: &str) -> Result<()> {
        let req = CreateTagsRequest {
            resources: vec![resource_id.to_string()],
            tags: vec![Tag {
                key: Some(key.to_string()),
                value: Some(value.to_string()),
            }],
            ..Default::default()
        };

        self.client.create_tags(req).await?;

        Ok(())
    }

    pub async fn delete_tags(&self, resources: &Vec<Resource>, tags: &Vec<&str>) -> Result<()> {
        let tags = tags
            .iter()
            .map(|name| Tag {
                key: Some(name.to_string()),
                value: None,
            })
            .collect();

        let req = DeleteTagsRequest {
            resources: resources.iter().map(|r| r.id.clone()).collect(),
            tags: Some(tags),
            dry_run: Some(false),
        };

        self.client.delete_tags(req).await?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct Resource {
    category: String,
    pub id: String,
}

impl Resource {
    fn from_tag_description(tag: &TagDescription) -> Resource {
        Resource {
            category: tag
                .resource_type
                .clone()
                .unwrap_or("resource type not found".to_string()),
            id: tag
                .resource_id
                .clone()
                .unwrap_or("resource id not found".to_string()),
        }
    }
    pub fn is_deletable(&self) -> bool {
        self.is_snapshot()
    }

    pub fn is_snapshot(&self) -> bool {
        self.category == "snapshot"
    }
}

impl std::fmt::Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{:>10} - {}", self.category, self.id).expect("The resource cannot be displayed");

        Ok(())
    }
}
