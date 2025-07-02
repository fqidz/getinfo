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
        Some(("battery", sub_matches)) => battery::exec(sub_matches),
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    }
}
