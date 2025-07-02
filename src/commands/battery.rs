use clap::{arg, ArgMatches, Command};
use gi_battery::Batteries;

pub fn cli() -> Command {
    Command::new("battery")
        .about("Scripts for battery info")
        .arg(arg!([TEST]))
}

pub fn exec(_args: &ArgMatches) {
    let batteries = Batteries::init().unwrap();
    let charge = batteries.get_battery_charge("BAT1").unwrap();
    let percentage = batteries.get_battery_percentage("BAT1").unwrap();
    println!("{charge} {percentage}%");
}
