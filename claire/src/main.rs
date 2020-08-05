#[macro_use]
extern crate clap;
extern crate tokio;

use anyhow::Result;
use claire::{
    clear_investigation, investigation_status, list_investigations, purge_investigation,
    start_investigation,
};
use clap::{App, Arg, ArgMatches, SubCommand};

#[tokio::main]
async fn main() -> Result<()> {
    let app = create_app();
    let args = app.get_matches();

    if let Some(args) = args.subcommand_matches("clear") {
        return clear_investigation(argvalue(args, "investigation_id")).await;
    }

    if let Some(args) = args.subcommand_matches("investigate") {
        return start_investigation(
            argvalue(args, "investigation_bucket"),
            argvalue(args, "instance_id"),
            argvalue(args, "reason"),
        )
        .await;
    }

    if let Some(args) = args.subcommand_matches("list") {
        return list_investigations(argvalue(args, "investigation_bucket")).await;
    }
    if let Some(args) = args.subcommand_matches("status") {
        return investigation_status(
            argvalue(args, "investigation_bucket"),
            argvalue(args, "instance_id"),
        )
        .await;
    }

    if let Some(args) = args.subcommand_matches("purge") {
        return purge_investigation(
            argvalue(args, "investigation_bucket"),
            argvalue(args, "investigation_id"),
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
        .help("The name of the S3 bucket where evidence is stored.");

    let instance_id = Arg::with_name("instance_id")
        .required(true)
        .takes_value(true)
        .help("The instance to investigate");

    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .subcommand(SubCommand::with_name("clear").arg(&id))
        .subcommand(
            SubCommand::with_name("investigate").arg(&instance_id).arg(
                Arg::with_name("reason")
                    .takes_value(true)
                    .required(true)
                    .help("The reason for the investigation"),
            ),
        )
        .subcommand(SubCommand::with_name("list").arg(&bucket))
        .subcommand(SubCommand::with_name("purge").arg(&bucket).arg(&id))
        .subcommand(
            SubCommand::with_name("status")
                .arg(&bucket)
                .arg(&instance_id),
        )
}

fn argvalue<'a>(matches: &'a ArgMatches, name: &str) -> &'a str {
    match matches.value_of(name) {
        Some(name) => name,
        None => "",
    }
}
