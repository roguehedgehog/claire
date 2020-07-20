mod repository;
mod service;

use service::purge::PurgeService;
use std::io::{stdin, stdout, Write};

pub async fn purge_investigation(
    investigation_bucket: &str,
    investigation_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let ps = PurgeService::new();
    let resources = ps
        .get_resources_to_purge(investigation_bucket, investigation_id)
        .await?;

    if resources.is_empty() {
        println!(
            "There are no resources to purge for investigation {}",
            investigation_id,
        );
        return Ok(());
    }
    println!(
        "These are the resources for investigation {}\n{}",
        investigation_id, resources
    );

    print!("\nType `yes` to confirm these changes> ");
    stdout().flush()?;

    let mut input = String::new();
    stdin().read_line(&mut input)?;
    if input.trim() != "yes" {
        println!("Aboring");
        return Ok(());
    }

    ps.purge_resources(investigation_bucket, &resources).await
}
