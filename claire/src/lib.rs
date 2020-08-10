mod access;
mod execute;
mod instance;
mod service;
mod storage;

use anyhow::{bail, Result};
use chrono::Utc;
use execute::state::InvestigationStatus;
use serde_json::to_string_pretty;
use service::clear::ClearInvestigationService;
use service::download::DownloadService;
use service::exec::ExecuteInvestigationService;
use service::investigation::InvestigationsService;
use service::isolate::IsolateInstanceService;
use service::manual::ManualInvestigationService;
use service::purge::PurgeService;
use service::revoke::RevokeInstancePermissionsService;
use std::io::{stdin, stdout, Write};
use std::thread::sleep;
use std::time::Duration;

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

pub async fn download_investigation(
    investigation_bucket: &str,
    investigation_id: &str,
    destination: &str,
) -> Result<()> {
    let service = DownloadService::new(investigation_bucket);
    service.download(investigation_id, destination).await?;

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

    confirm()?;

    ps.purge_resources(&resources, &objects).await
}

pub async fn start_investigation(
    investigation_bucket: &str,
    instance_id: &str,
    reason: &str,
) -> Result<()> {
    let service = ExecuteInvestigationService::new(investigation_bucket);
    let execution_id = service.start(instance_id, reason).await?;

    println!("Investigation started: {}", execution_id);
    let mut refresh = true;
    let mut previous_status: Option<InvestigationStatus> = None;
    let mut found_id = false;

    while refresh {
        if !found_id {
            if let Some(investigation_id) = service.get_investigation_id(instance_id).await {
                found_id = true;
                println!("\nInvestigation created: {}\n", investigation_id);
            }
        }

        let status = service.last_update(&execution_id).await?;
        refresh = print_investigation_status(&status, &previous_status)?;
        if refresh {
            sleep(Duration::from_secs(1));
            previous_status = Some(status);
        }
    }

    Ok(())
}

pub async fn investigation_status(investigation_bucket: &str, instance_id: &str) -> Result<()> {
    let service = ExecuteInvestigationService::new(investigation_bucket);
    let details = service.execution_detials(instance_id).await?;
    let mut previous_status: Option<InvestigationStatus> = None;

    let mut refresh = true;

    println!("Status of investigation {}:", details.investigation_id);
    while refresh {
        let status = service.last_update(&details.execution_arn).await?;
        refresh = print_investigation_status(&status, &previous_status)?;
        if refresh {
            sleep(Duration::from_secs(1));
            previous_status = Some(status);
        }
    }

    Ok(())
}

fn print_investigation_status(
    current: &InvestigationStatus,
    previous: &Option<InvestigationStatus>,
) -> Result<bool> {
    let finished_status = ["ExecutionAborted", "ExecutionFailed", "ExecutionSucceeded"];
    let current_status = format!("{} {}", current.status, current.get_task_name()?);
    let previous_status = match previous {
        Some(previous) => format!("{} {}", previous.status, previous.get_task_name()?),
        None => String::new(),
    };

    if current_status == previous_status {
        print!(".");
    } else {
        print!("\n{} {}", Utc::now(), current_status);
    }
    if stdout().flush().is_err() {}

    let refresh = match finished_status.iter().position(|s| s == &current.status) {
        Some(_) => false,
        None => true,
    };

    if !refresh && current.status == "ExecutionFailed" {
        println!(
            "Execution Failed with:\n{}",
            to_string_pretty(&current.details).unwrap_or(String::new())
        )
    }

    Ok(refresh)
}

pub async fn manual_investigation(
    investigation_id: &str,
    investigation_bucket: &str,
    key_name: &str,
) -> Result<()> {
    let service = ManualInvestigationService::new(investigation_bucket);
    let (extractor, vols) = service.create_resources(investigation_id, key_name).await?;

    println!(
        "Extractor {} and volume(s) {} are being created.",
        extractor,
        vols.join(", ")
    );

    let mut instance_ready = false;
    let mut vols_ready = false;
    while !instance_ready || !vols_ready {
        sleep(Duration::from_secs(1));
        print!(".");
        if stdout().flush().is_err() {}

        if !instance_ready {
            instance_ready = service.is_instance_ready(&extractor).await?;
            if instance_ready {
                println!("Instance is ready");
            }
        }

        if !vols_ready {
            vols_ready = service.is_volumes_ready(&vols).await?;
            if vols_ready {
                println!("Volume(s) ready");
            }
        }
    }

    println!("Attaching volume(s)");
    let devices = service.attach_volumes(&extractor, &vols).await?;

    println!("Volumes attached at:");
    for (device, _) in devices {
        println!("{}", device);
    }

    println!("IP {}", service.get_ip(&extractor).await?);

    Ok(())
}

pub async fn revoke_access(investigation_bucket: &str, investigation_id: &str) -> Result<()> {
    let service = RevokeInstancePermissionsService::new(investigation_bucket);
    let investigation = service.get_investigation(investigation_id).await?;
    println!("Found investigation {}", investigation.bucket);

    let profile = service.remove_profile(&investigation.instance_id).await?;
    if profile.is_empty() {
        println!(
            "Instance {} does not have a profile",
            investigation.instance_id
        )
    } else {
        println!(
            "Profile {} was removed from instance {}",
            profile, investigation.instance_id
        );
    }

    let roles = service.get_roles(&investigation.instance_id).await?;
    if roles.is_empty() {
        bail!("There are no tokens to invalidate because the profile does not have any roles assigned.");
    }

    println!("These roles will have their existing tokens invalidated:\n");
    for role in &roles {
        println!("{}", role);
    }

    println!("\n\
    Invalidating tokens will require an app or user to clear their cached token(s) and generating new ones.\n\
    Apps which get tokens through the EC2 meta-data service will have to wait for Amazon to refresh their tokens,\n\
    a manual refresh can be initialed by reapplying the instance profile to each affected instance.");

    confirm()?;

    service.invalidate_tokens(&roles).await?;

    println!("ClaireInvalidateTokens inline policy added to roles");

    Ok(())
}

pub async fn isolate_instance(investigation_bucket: &str, investigation_id: &str) -> Result<()> {
    let service = IsolateInstanceService::new(investigation_bucket);
    let investigation = service.isolate(investigation_id).await?;

    println!(
        "The instance {} has been contained",
        investigation.instance_id
    );

    Ok(())
}

fn confirm() -> Result<()> {
    let mut input = String::new();

    print!("\nType `yes` to confirm these changes> ");
    if stdout().flush().is_err() {}

    stdin().read_line(&mut input)?;
    if input.trim() != "yes" {
        bail!("Aboring");
    }

    Ok(())
}
