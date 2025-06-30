use clap::command;
use crate::commands::battery;

mod commands;

fn main() {
    let matches = command!()
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(battery::cli())
        .get_matches();

    match matches.subcommand() {
        Some(("battery", sub_matches)) => println!(
            "'bar-scripts battery' was used, test is: {:?}",
            sub_matches.get_one::<String>("TEST")
        ),
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    }
}
