mod instance;
mod service;
mod storage;

use service::clear::ClearInvestigationService;
use service::list::ListInvestigationsService;
use service::purge::PurgeService;
use std::io::{stdin, stdout, Write};

static CLAIRE: &str = "CLAIRE";
static INVESTIGATION_TAG_KEY: &str = "InvestigationId";

pub async fn clear_investigation(investigation_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let resources = ClearInvestigationService::new()
        .clear_investigation(investigation_id)
        .await?;

    if resources.len() == 0 {
        println!("No resources found for investigation {}", investigation_id);
    } else {
        println!("These resources have been untagged:");
        for resource in &resources {
            println!("{}", resource);
        }
    }
    Ok(())
}

pub async fn list_investigations(
    investigation_bucket: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let investigations = ListInvestigationsService::new()
        .get_investigations(investigation_bucket)
        .await?;
    if investigations.is_empty() {
        println!("There are no investigations")
    } else {
        for investigation in investigations {
            println!("{}", investigation.bucket)
        }
    }

    Ok(())
}

pub async fn purge_investigation(
    investigation_bucket: &str,
    investigation_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let ps = PurgeService::new();
    let (resources, objects) = ps
        .get_resources_to_purge(investigation_bucket, investigation_id)
        .await?;

    let mut resources = resources.clone();
    resources.sort_by(|a, b| a.is_deletable().cmp(&b.is_deletable()));

    if resources.is_empty() && objects.is_empty() {
        println!(
            "There are no resources to purge for investigation {}",
            investigation_id,
        );
        return Ok(());
    }

    println!(
        "These are the resources for investigation {}\n\
        The following changes will be made: ",
        investigation_id
    );

    for resource in &resources {
        println!(
            "{:>10} {}",
            if resource.is_deletable() {
                "Delete"
            } else {
                "Untag"
            },
            resource
        );
    }

    for object in &objects {
        println!("Delete {}", object.key.clone().unwrap());
    }

    print!("\nType `yes` to confirm these changes> ");
    stdout().flush()?;

    let mut input = String::new();
    stdin().read_line(&mut input)?;
    if input.trim() != "yes" {
        println!("Aboring");
        return Ok(());
    }

    ps.purge_resources(investigation_bucket, &resources, &objects)
        .await
}
