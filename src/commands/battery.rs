use std::{sync::mpsc, thread, time::Duration};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use gi_battery::Batteries;
use notify::{Config, Event, PollWatcher, RecursiveMode, Watcher};

pub fn cli() -> Command {
    Command::new("battery")
        .about("Scripts for battery info")
        .arg(
            Arg::new("watch")
                .short('w')
                .long("watch")
                .conflicts_with("poll")
                .action(ArgAction::SetTrue)
                .help("Watches/follows the requested info and outputs all whenever one changes"),
        )
        .arg(
            Arg::new("poll")
                .short('p')
                .long("poll")
                .conflicts_with("watch")
                .value_parser(value_parser!(u64))
                .value_name("MILLISECONDS")
                .help("Outputs after every interval"),
        )
}

// TODO: proper error handling
// TODO: move watch and poll to the gi_battery crate
pub fn exec(args: &ArgMatches) {
    let batteries = Batteries::init().unwrap();
    if args.get_flag("watch") {
        let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
        let config = Config::default()
            .with_compare_contents(true)
            .with_poll_interval(Duration::from_secs(1));

        let mut watcher = PollWatcher::new(tx, config).unwrap();

        let path = batteries.get_battery("BAT1").unwrap().path.join("charge_now");
        watcher.watch(&path, RecursiveMode::NonRecursive).unwrap();

        let mut charge = batteries.get_charge_single("BAT1").unwrap();
        let mut percentage = batteries.get_percentage_single("BAT1").unwrap();
        println!("{charge} {percentage}%");

        for res in rx {
            match res {
                Ok(_event) => {
                    charge = batteries.get_charge_single("BAT1").unwrap();
                    percentage = batteries.get_percentage_single("BAT1").unwrap();
                    println!("{charge} {percentage}%");
                }
                Err(e) => eprintln!("Watcher error: {}", e),
            }
        }
    } else if let Some(milliseconds) = args.get_one::<u64>("poll") {
        loop {
            let charge = batteries.get_charge_single("BAT1").unwrap();
            let percentage = batteries.get_percentage_single("BAT1").unwrap();
            println!("{charge} {percentage}%");
            thread::sleep(Duration::from_millis(*milliseconds));
        }
    } else {
        let charge = batteries.get_charge_single("BAT1").unwrap();
        let percentage = batteries.get_percentage_single("BAT1").unwrap();
        println!("{charge} {percentage}%");
    }
}
