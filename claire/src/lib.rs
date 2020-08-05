mod execute;
mod instance;
mod service;
mod storage;

use anyhow::Result;
use chrono::Utc;
use serde_json::to_string_pretty;
use service::clear::ClearInvestigationService;
use service::exec::ExecuteInvestigationService;
use service::investigation::InvestigationsService;
use service::purge::PurgeService;
use std::io::{stdin, stdout, Write};

static CLAIRE: &str = "CLAIRE";
static INVESTIGATION_TAG_KEY: &str = "InvestigationId";

pub async fn clear_investigation(investigation_id: &str) -> Result<()> {
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

pub async fn list_investigations(investigation_bucket: &str) -> Result<()> {
    let investigations = InvestigationsService::new(investigation_bucket)
        .get_investigations(None)
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

pub async fn purge_investigation(investigation_bucket: &str, investigation_id: &str) -> Result<()> {
    let ps = PurgeService::new(investigation_bucket);
    let (resources, objects) = ps.get_resources_to_purge(investigation_id).await?;

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

    ps.purge_resources(&resources, &objects).await
}

pub async fn start_investigation(instance_id: &str, reason: &str) -> Result<()> {
    let service = ExecuteInvestigationService::new();
    let execution_id = service.start(instance_id, reason).await?;

    println!("Investigation started: {}", execution_id);
    let mut refresh = true;
    let mut previous_status = String::new();
    let finished_status = ["ExecutionAborted", "ExecutionFailed", "ExecutionSucceeded"];
    let mut found_id = false;

    while refresh {
        if !found_id {
            if let Some(investigation_id) = service.get_investigation_id(instance_id).await {
                found_id = true;
                println!("Investigation created: {}", investigation_id);
            }
        }

        let investigation = service.status(&execution_id).await?;
        let current_status = format!(
            "{} {}",
            investigation.status,
            investigation.get_task_name()?
        );

        if current_status == previous_status {
            print!(".");
        } else {
            print!("\n{} {}", Utc::now(), current_status);
            previous_status = current_status;
        }
        if std::io::stdout().flush().is_err() {}

        refresh = match finished_status
            .iter()
            .position(|s| s == &investigation.status)
        {
            Some(_) => false,
            None => true,
        };

        if refresh {
            std::thread::sleep(std::time::Duration::from_secs(1))
        } else if investigation.status == "ExecutionFailed" {
            println!(
                "Execution Failed with:\n{}",
                to_string_pretty(&investigation.details).unwrap_or(String::new())
            )
        }
    }

    Ok(())
}
