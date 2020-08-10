use anyhow::Result;
use claire::{
    clear_investigation, download_investigation, investigation_status, isolate_instance,
    list_investigations, manual_investigation, purge_investigation, revoke_access,
    start_investigation,
};
use clap::{crate_authors, crate_name, crate_version, App, Arg, ArgMatches, SubCommand};
use env_logger::Env;

#[tokio::main]
async fn main() -> Result<()> {
    let app = create_app();
    let args = app.get_matches();

    env_logger::init_from_env(
        Env::default()
            .filter_or("CLAIRE_LOG_LEVEL", "info")
            .write_style_or("CLAIRE_LOG_STYLE", "always"),
    );

    if let Some(args) = args.subcommand_matches("clear") {
        return clear_investigation(get(args, "investigation_id")).await;
    }

    if let Some(args) = args.subcommand_matches("download") {
        return download_investigation(
            get(args, "investigation_bucket"),
            get(args, "investigation_id"),
            get(args, "destination"),
        )
        .await;
    }

    if let Some(args) = args.subcommand_matches("investigate") {
        return start_investigation(
            get(args, "investigation_bucket"),
            get(args, "instance_id"),
            get(args, "reason"),
            args.is_present("isolate"),
        )
        .await;
    }

    if let Some(args) = args.subcommand_matches("revoke") {
        return revoke_access(
            get(args, "investigation_bucket"),
            get(args, "investigation_id"),
        )
        .await;
    }

    if let Some(args) = args.subcommand_matches("isolate") {
        return isolate_instance(
            get(args, "investigation_bucket"),
            get(args, "investigation_id"),
        )
        .await;
    }

    if let Some(args) = args.subcommand_matches("list") {
        return list_investigations(get(args, "investigation_bucket")).await;
    }

    if let Some(args) = args.subcommand_matches("manual") {
        return manual_investigation(
            get(args, "investigation_id"),
            get(args, "investigation_bucket"),
            get(args, "key_name"),
        )
        .await;
    }

    if let Some(args) = args.subcommand_matches("status") {
        return investigation_status(get(args, "investigation_bucket"), get(args, "instance_id"))
            .await;
    }

    if let Some(args) = args.subcommand_matches("purge") {
        return purge_investigation(
            get(args, "investigation_bucket"),
            get(args, "investigation_id"),
        )
        .await;
    }

    Ok(())
}

fn create_app<'a, 'b>() -> App<'a, 'b> {
    let id = Arg::with_name("investigation_id")
        .env("INVESTIGATION_ID")
        .required(true);

    let bucket = Arg::with_name("investigation_bucket")
        .long("bucket")
        .env("INVESTIGATION_BUCKET")
        .required(true)
        .long_help(
            "The name of the S3 bucket where evidence is stored, i.e. [your-prefix]-investigations.",
        );

    let instance_id = Arg::with_name("instance_id")
        .required(true)
        .takes_value(true)
        .help("The instance to investigate");

    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about("Initialate, manage and clear CLAIRE investigations")
        .subcommand(SubCommand::with_name("clear").arg(&id).about(
            "Removes the CLARE tags from investigated resources \
             use this command to clear the investigation but leave the collected evidence",
        ))
        .subcommand(
            SubCommand::with_name("download")
                .arg(&id)
                .arg(
                    Arg::with_name("destination")
                        .required(true)
                        .takes_value(true)
                        .help("The destination to download the investigation data"),
                )
                .arg(&bucket)
                .about("Download investigation evidence to a local directory"),
        )
        .subcommand(
            SubCommand::with_name("investigate")
                .arg(&instance_id)
                .arg(
                    Arg::with_name("reason")
                        .takes_value(true)
                        .required(true)
                        .help("The reason for the investigation"),
                )
                .arg(&bucket)
                .arg(Arg::with_name("isolate").short("i").long("isolate"))
                .about("Starts an investigation into the given instance"),
        )
        .subcommand(SubCommand::with_name("revoke").arg(&id).arg(&bucket).about(
            "Revoke instance permissions and invalidate any tokens that may have been stolen.",
        ))
        .subcommand(
            SubCommand::with_name("isolate")
                .arg(&id)
                .arg(&bucket)
                .about(
                    "Apply restrictive security group, add NCL entry and remove instance profile",
                ),
        )
        .subcommand(
            SubCommand::with_name("list")
                .arg(&bucket)
                .about("List the investigations"),
        )
        .subcommand(
            SubCommand::with_name("manual")
                .arg(&bucket)
                .arg(&id)
                .arg(
                    Arg::with_name("key_name")
                        .takes_value(true)
                        .required(true)
                        .help("The KeyPair stored on AWS to SSH into this instance."),
                )
                .about(
                    "Spin up an instance and attach snapshots of a suspicious \
            instance so an investigation can be continued manually.",
                ),
        )
        .subcommand(SubCommand::with_name("purge").arg(&bucket).arg(&id).about(
            "Purge the investigation, removes evidence from S3, \
        untags and deletes snapshots",
        ))
        .subcommand(
            SubCommand::with_name("status")
                .arg(&bucket)
                .arg(&instance_id)
                .about("View the status of an investigation"),
        )
}

fn get<'a>(matches: &'a ArgMatches, name: &str) -> &'a str {
    matches
        .value_of(name)
        .expect(&format!("{} must be provided", name))
}
