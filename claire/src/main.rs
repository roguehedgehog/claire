#[macro_use]
extern crate clap;
extern crate tokio;

use claire::purge_investigation;
use clap::{App, Arg, SubCommand};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = create_app();
    let args = app.get_matches();
    let investigation_id = args
        .value_of("investigation_id")
        .expect("investigation_id is required");

    if let Some(purge_args) = args.subcommand_matches("purge") {
        purge_investigation(
            purge_args
                .value_of("investigation_bucket")
                .expect("bucket is required"),
            investigation_id,
        )
        .await?
    }

    Ok(())
}

fn create_app<'a, 'b>() -> App<'a, 'b> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .arg(
            Arg::with_name("investigation_id")
                .long("investigation_id")
                .env("INVESTIGATION_ID")
                .required(true)
                .help(
                    "Investigation ID is required for all commands, \
                pass it as a command line argument or environment variable.",
                ),
        )
        .subcommand(
            SubCommand::with_name("purge").arg(
                Arg::with_name("investigation_bucket")
                    .long("bucket")
                    .env("INVESTIGATION_BUCKET")
                    .required(true)
                    .help("The name of the s3 bucket where evidence is stored."),
            ),
        )
}