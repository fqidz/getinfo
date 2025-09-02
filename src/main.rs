use crate::commands::{battery, media};
use clap::command;

mod commands;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let matches = command!()
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(battery::cli())
        .subcommand(media::cli())
        .get_matches();

    match matches.subcommand() {
        Some(("battery", sub_matches)) => battery::exec(sub_matches),
        Some(("media", sub_matches)) => media::exec(sub_matches).await,
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    }
}
