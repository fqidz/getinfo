use std::collections::HashSet;
use std::fmt::Write as _;
use std::{sync::mpsc, time::Duration};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use gi_battery::{
    get_main_battery_name, AsTimestamp, Batteries, BatteryInfoName, BatteryStatus, Timestamp
};
use notify::{Config, Event, PollWatcher, RecursiveMode, Watcher};
use serde::Serialize;

pub trait BatteryInfoNameExt {
    fn files_to_watch(&self) -> Vec<&str>;
}

impl BatteryInfoNameExt for BatteryInfoName {
    fn files_to_watch(&self) -> Vec<&str> {
        match self {
            // No need to watch charge_full
            BatteryInfoName::ChargeFull => Vec::new(),
            BatteryInfoName::ChargeNow => vec!["charge_now"],
            BatteryInfoName::ChargeNowPercentage => vec!["charge_now"],
            BatteryInfoName::CurrentNow => vec!["current_now"],
            BatteryInfoName::Status => vec!["status"],
            BatteryInfoName::TimeRemaining => vec!["charge_now", "current_now"],
        }
    }
}

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
                .conflicts_with("json")
                .value_name("STRING")
                .default_value(" ")
                .help("Character or string to use for separating output infos"),
        )
        // TODO: make this into `--format` with choices: normal, json, json_pretty
        .arg(
            Arg::new("json")
                .short('j')
                .long("json")
                .conflicts_with("separator")
                .action(ArgAction::SetTrue)
                .help("Formats output into json"),
        )
}

struct BatteryContext {
    battery_name: String,
    is_raw: bool,
    separator: String,
    do_output_json: bool,
}

struct BatterySubcommand<'a> {
    batteries: Batteries,
    info_names: &'a Vec<&'a BatteryInfoName>,
    context: BatteryContext,
}

#[derive(Default, Serialize)]
struct BatteryOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    charge_full: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    charge_now: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    charge_now_percentage: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    current_now: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    time_remaining: Option<String>,
}


impl<'a> BatterySubcommand<'a> {
    fn init(
        batteries: Batteries,
        info_names: &'a Vec<&'a BatteryInfoName>,
        context: BatteryContext,
    ) -> Self {
        Self {
            batteries,
            info_names,
            context,
        }
    }

    fn watch(&mut self) {
        let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
        let config = Config::default()
            .with_compare_contents(true)
            .with_poll_interval(Duration::from_secs_f64(0.2));

        let mut watcher = PollWatcher::new(tx, config).unwrap();

        let battery_path = &self
            .batteries
            .get_battery(&self.context.battery_name)
            .unwrap()
            .path;

        // Although `notify` already handles duplicate watched files properly, we filter out duplicate
        // files just to avoid the extra calls to `watcher.watch(...)`. Have not tested if this is
        // faster/more efficient.
        let mut files_to_watch = HashSet::new();
        for info_name in self.info_names.iter() {
            for filename in info_name.files_to_watch() {
                files_to_watch.insert(filename);
            }
        }

        for filename in files_to_watch {
            watcher
                .watch(&battery_path.join(filename), RecursiveMode::NonRecursive)
                .unwrap();
        }

        let mut previous_output = self.get_output_string();
        println!("{}", previous_output);

        // TODO: find a better way to prevent outputting redundant values other than checking it
        // with the previous output
        for _res in rx {
            let output = self.get_output_string();
            if previous_output != output {
                println!("{}", output);
                previous_output = output;
            }
        }
    }

    fn poll(&self, milliseconds: u64) {
        let duration = Duration::from_millis(milliseconds);
        loop {
            println!("{}", self.get_output_string());
            std::thread::sleep(duration);
        }
    }

    fn get_output_string(&self) -> String {
        let main_battery = self.batteries.get_main_battery().unwrap();
        let mut battery_output = BatteryOutput::default();
        for (i, info_name) in self.info_names.iter().enumerate() {
            match info_name {
                BatteryInfoName::ChargeNow => {
                    let string = if self.context.is_raw {
                        main_battery.get_charge_now().unwrap().to_string()
                    } else {
                        format!("{}mAh", main_battery.get_charge_now().unwrap() / 1000)
                    };
                    battery_output.charge_now = Some(string);
                }
                BatteryInfoName::ChargeNowPercentage => {
                    let string = if self.context.is_raw {
                        main_battery.get_charge_now_percentage().unwrap().to_string()
                    } else {
                        format!("{}%", main_battery.get_charge_now_percentage().unwrap() * 100.0)
                    };
                    battery_output.charge_now_percentage = Some(string);
                }
                BatteryInfoName::ChargeFull => {
                    let string = if self.context.is_raw {
                        main_battery.get_charge_full().to_string()
                    } else {
                        format!("{}mAh", main_battery.get_charge_full() / 1000)
                    };
                    battery_output.charge_full = Some(string);
                }
                BatteryInfoName::CurrentNow => {
                    let string = if self.context.is_raw {
                        main_battery.get_current_now().unwrap().to_string()
                    } else {
                        format!("{}mA", main_battery.get_current_now().unwrap() / 1000)
                    };
                    battery_output.current_now = Some(string);
                }
                BatteryInfoName::TimeRemaining => {
                    let string = if self.context.is_raw {
                        main_battery.get_time_remaining().unwrap().to_string()
                    } else {
                        let timestamp = match main_battery.get_status().unwrap() {
                            BatteryStatus::Full
                            | BatteryStatus::NotCharging
                            | BatteryStatus::Unknown => Timestamp::default(),
                            BatteryStatus::Charging | BatteryStatus::Discharging => {
                                main_battery.get_time_remaining().unwrap().as_timestamp()
                            }
                        };
                        timestamp.to_string()
                    };
                    battery_output.time_remaining = Some(string);
                }
                BatteryInfoName::Status => {
                    battery_output.status = Some(main_battery.get_status().unwrap().to_string());
                }
            }
            // if i < self.info_names.len() - 1 {
            //     write!(output, "{}", self.context.separator).unwrap();
            // }
        }
        serde_json::to_string(&battery_output).expect("always valid")
    }
}

// TODO: proper error handling
// TODO: move a lot of the things here to the gi_battery crate
pub fn exec(args: &ArgMatches) {
    let is_raw = args.get_one::<bool>("raw").unwrap();
    let do_output_json = args.get_one::<bool>("json").unwrap();
    let batteries = Batteries::init().unwrap();
    let default_battery_name = get_main_battery_name().unwrap();
    let battery_name = args
        .get_one::<String>("name")
        .unwrap_or(&default_battery_name);

    let separator = args
        .get_one::<String>("separator")
        .expect("has a default value");

    let input_info_names = args
        .get_many::<BatteryInfoName>("info_names")
        .expect("has a default value")
        .collect::<Vec<_>>();

    let mut battery_subcommand = BatterySubcommand::init(
        batteries,
        &input_info_names,
        BatteryContext {
            battery_name: battery_name.to_string(),
            is_raw: *is_raw,
            separator: separator.to_string(),
            do_output_json: *do_output_json,
        },
    );

    if args.get_flag("watch") {
        battery_subcommand.watch();
    } else if let Some(milliseconds) = args.get_one::<u64>("poll") {
        battery_subcommand.poll(*milliseconds)
    } else {
        println!("{}", battery_subcommand.get_output_string());
    }
}
