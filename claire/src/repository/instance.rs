extern crate rusoto_core;
extern crate rusoto_ec2;

use rusoto_core::Region;
use rusoto_ec2::{DeleteSnapshotRequest, Ec2, Ec2Client};
pub struct InstanceRepo {
    client: Ec2Client,
}

impl InstanceRepo {
    pub fn new() -> InstanceRepo {
        InstanceRepo {
            client: Ec2Client::new(Region::default()),
        }
    }

    pub async fn delete_snapshots(
        &self,
        snapshots: &Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for snapshot in snapshots {
            let req = DeleteSnapshotRequest {
                snapshot_id: snapshot.clone(),
                dry_run: Some(false),
            };

            let resp = self.client.delete_snapshot(req).await;
            println!("{:?}", resp);
        }

        Ok(())
    }
}