extern crate rusoto_core;
extern crate rusoto_ec2;

use anyhow::Result;
use rusoto_core::Region;
use rusoto_ec2::{DeleteSnapshotRequest, Ec2, Ec2Client};
pub struct SnapshotRepo {
    client: Ec2Client,
}

impl SnapshotRepo {
    pub fn new() -> SnapshotRepo {
        SnapshotRepo {
            client: Ec2Client::new(Region::default()),
        }
    }

    pub async fn delete_snapshots(&self, snapshots: &Vec<String>) -> Result<()> {
        for snapshot in snapshots {
            let req = DeleteSnapshotRequest {
                snapshot_id: snapshot.clone(),
                dry_run: Some(false),
            };

            self.client.delete_snapshot(req).await?;
        }

        Ok(())
    }
}
