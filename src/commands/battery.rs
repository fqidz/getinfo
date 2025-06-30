use clap::{arg, Command};

pub fn cli() -> Command {
    Command::new("battery")
        .about("Scripts for battery info")
        .arg(arg!([TEST]))
}
