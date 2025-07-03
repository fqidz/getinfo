use std::fmt::Write as _;
use std::time::Instant;
use std::{sync::mpsc, thread, time::Duration};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use gi_battery::{
    Batteries, BatteryInfoName, BatteryStatus, DurationExt, Timestamp, get_main_battery_name,
};
use notify::{Config, Event, PollWatcher, RecursiveMode, Watcher};

pub fn cli() -> Command {
    Command::new("battery")
        .about("Scripts for battery info")
        .arg(
            Arg::new("info_names")
                .value_name("INFO_NAME")
                .action(ArgAction::Append)
                .value_parser(value_parser!(BatteryInfoName))
                .value_delimiter(',')
                .default_value("percent,charge,charge_full,current_now,time_remaining,status")
                .help("Specify which info(s) to get (e.g. 'charge_full,charge_percentage')"),
        )
        .arg(
            Arg::new("name")
                .value_name("BAT")
                .short('n')
                .long("name")
                .help("Specify battery name in the case of multiple batteries (e.g. 'BAT1'). Defaults to lowest-numbered battery"),
        )
        .arg(
            Arg::new("raw")
                .short('r')
                .long("raw")
                .action(ArgAction::SetTrue)
                .help("Output values as their raw values"),
        )
        .arg(
            Arg::new("watch")
                .short('w')
                .long("watch")
                .conflicts_with("poll")
                .action(ArgAction::SetTrue)
                .help("Outputs only when info changes"),
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
        .arg(
            Arg::new("separator")
                .short('s')
                .long("separator")
                .value_name("STRING")
                .default_value(" ")
                .help("Character or string to use for separating output infos"),
        )
}

// TODO: proper error handling
// TODO: move a lot of the things here to the gi_battery crate
pub fn exec(args: &ArgMatches) {
    let is_raw = args.get_one::<bool>("raw").unwrap();
    let batteries = Batteries::init().unwrap();
    let default_battery_name = get_main_battery_name().unwrap();
    let battery_name = args
        .get_one::<String>("name")
        .unwrap_or(&default_battery_name);

    let separator = args
        .get_one::<String>("separator")
        .expect("has a default value");

    let input_info_names = parse_info_names(args);

    if args.get_flag("watch") {
        let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
        let config = Config::default()
            .with_compare_contents(true)
            .with_poll_interval(Duration::from_secs(1));

        let mut watcher = PollWatcher::new(tx, config).unwrap();

        let path = &batteries.get_battery(battery_name).unwrap().path;

        // TODO: better way to decide what files to watch
        for info_name in &input_info_names {
            match info_name {
                BatteryInfoName::ChargeNow | BatteryInfoName::ChargeNowPercentage => watcher
                    .watch(&path.join("charge_now"), RecursiveMode::NonRecursive)
                    .unwrap(),
                BatteryInfoName::ChargeFull => { /* no-op */ }
                BatteryInfoName::CurrentNow => {
                    watcher
                        .watch(&path.join("current_now"), RecursiveMode::NonRecursive)
                        .unwrap();
                }
                BatteryInfoName::Status => watcher
                    .watch(&path.join("status"), RecursiveMode::NonRecursive)
                    .unwrap(),
                BatteryInfoName::TimeRemaining => {
                    watcher
                        .watch(&path.join("charge_now"), RecursiveMode::NonRecursive)
                        .unwrap();
                    watcher
                        .watch(&path.join("current_now"), RecursiveMode::NonRecursive)
                        .unwrap();
                    watcher
                        .watch(&path.join("status"), RecursiveMode::NonRecursive)
                        .unwrap();
                }
            }
        }

        let mut previous_output = format_output(
            &batteries,
            &input_info_names,
            battery_name,
            separator,
            *is_raw,
        );
        println!("{}", previous_output);

        // TODO: find a better way to prevent outputting redundant values other than checking it
        // with the previous output
        let mut time_last_outputted = Instant::now();
        for _res in rx {
            // prevent outputs in quick succession, such as when percentage reaches '100', it also
            // updates status from 'Charging' to 'Full'
            let time_outputted_now = Instant::now();

            if time_last_outputted.elapsed().as_millis() > 0 {
                let output = format_output(
                    &batteries,
                    &input_info_names,
                    battery_name,
                    separator,
                    *is_raw,
                );
                if previous_output != output {
                    println!("{}", output);
                    previous_output = output;
                }
            }
            time_last_outputted = time_outputted_now
        }
    } else if let Some(milliseconds) = args.get_one::<u64>("poll") {
        loop {
            println!(
                "{}",
                format_output(
                    &batteries,
                    &input_info_names,
                    battery_name,
                    separator,
                    *is_raw
                )
            );
            thread::sleep(Duration::from_millis(*milliseconds));
        }
    } else {
        println!(
            "{}",
            format_output(
                &batteries,
                &input_info_names,
                battery_name,
                separator,
                *is_raw
            )
        );
    }
}

// TODO: find a better way to do this
#[inline]
fn format_output(
    batteries: &Batteries,
    info_names: &Vec<&BatteryInfoName>,
    battery_name: &str,
    separator: &str,
    is_raw: bool,
) -> String {
    let mut output = String::with_capacity(5);
    for (i, info_name) in info_names.iter().enumerate() {
        match info_name {
            BatteryInfoName::ChargeNow => {
                if is_raw {
                    write!(
                        output,
                        "{}",
                        batteries.get_charge_now_single(battery_name).unwrap()
                    )
                    .unwrap();
                } else {
                    write!(
                        output,
                        "{}mAh",
                        batteries.get_charge_now_single(battery_name).unwrap() / 1000
                    )
                    .unwrap();
                }
            }
            BatteryInfoName::ChargeNowPercentage => {
                if is_raw {
                    write!(
                        output,
                        "{}",
                        batteries
                            .get_charge_percentage_single(battery_name)
                            .unwrap()
                    )
                    .unwrap();
                } else {
                    write!(
                        output,
                        "{}%",
                        batteries
                            .get_charge_percentage_single(battery_name)
                            .unwrap()
                            * 100.0
                    )
                    .unwrap();
                }
            }
            BatteryInfoName::ChargeFull => {
                if is_raw {
                    write!(
                        output,
                        "{}",
                        batteries.get_charge_full_single(battery_name).unwrap()
                    )
                    .unwrap();
                } else {
                    write!(
                        output,
                        "{}mAh",
                        batteries.get_charge_full_single(battery_name).unwrap() / 1000
                    )
                    .unwrap();
                }
            }
            BatteryInfoName::CurrentNow => {
                if is_raw {
                    write!(
                        output,
                        "{}",
                        batteries.get_current_now_single(battery_name).unwrap()
                    )
                    .unwrap();
                } else {
                    write!(
                        output,
                        "{}mA",
                        batteries.get_current_now_single(battery_name).unwrap() / 1000
                    )
                    .unwrap();
                }
            }
            BatteryInfoName::TimeRemaining => {
                if is_raw {
                    write!(
                        output,
                        "{}",
                        batteries
                            .get_time_remaining_single(battery_name)
                            .unwrap()
                            .as_secs()
                    )
                    .unwrap();
                } else {
                    let timestamp = match batteries.get_status_single(battery_name).unwrap() {
                        BatteryStatus::Full | BatteryStatus::NotCharging => Timestamp::default(),
                        BatteryStatus::Charging | BatteryStatus::Discharging => batteries
                            .get_time_remaining_single(battery_name)
                            .unwrap()
                            .display_as_timestamp(),
                    };
                    write!(output, "{}", timestamp).unwrap();
                }
            }
            BatteryInfoName::Status => {
                write!(
                    output,
                    "{}",
                    batteries.get_status_single(battery_name).unwrap()
                )
                .unwrap();
            }
        }
        if i < info_names.len() - 1 {
            write!(output, "{}", separator).unwrap();
        }
    }
    output
}

pub fn parse_info_names(matches: &ArgMatches) -> Vec<&BatteryInfoName> {
    matches
        .get_many::<BatteryInfoName>("info_names")
        .expect("has a default value")
        .collect::<Vec<_>>()
}
