use core::Script;

use clap::{arg, Command};

pub struct Battery;

impl Script for Battery {
    fn build_command() -> clap::Command {
        Command::new("battery")
            .about("Scripts for battery info")
            .arg(arg!([TEST]))
    }
}
