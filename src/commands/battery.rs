use std::collections::HashSet;
use std::fmt::Display;
use std::{sync::mpsc, time::Duration};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use gi_battery::{Batteries, BatteryInfoName, BatteryStatus, get_main_battery_name};
use gi_core::AsTimestamp;
use notify::{Config, Event, PollWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use serde::ser::SerializeMap;

use crate::commands::{Field, FieldValue, FormatOutputType, SubCommandExt};

pub trait BatteryInfoNameExt {
    fn files_to_watch(&self) -> Vec<&str>;
}

impl BatteryInfoNameExt for BatteryInfoName {
    fn files_to_watch(&self) -> Vec<&str> {
        match self {
            // No need to watch charge_full
            BatteryInfoName::ChargeFull => Vec::new(),
            BatteryInfoName::ChargeNow => vec!["charge_now"],
            BatteryInfoName::Capacity => vec!["charge_now"],
            BatteryInfoName::CurrentNow => vec!["current_now"],
            BatteryInfoName::Status => vec!["status"],
            BatteryInfoName::TimeRemaining => vec!["charge_now", "current_now"],
        }
    }
}

pub fn cli() -> Command {
    Command::new("battery")
        .about("Scripts for battery info")
        .common_args()
        .arg(
            Arg::new("info_names")
                .value_name("INFO_NAME")
                .action(ArgAction::Append)
                .value_parser(value_parser!(BatteryInfoName))
                .value_delimiter(',')
                .default_value("capacity,charge,charge_full,current_now,time_remaining,status")
                .help("Specify which info(s) to get (e.g. 'charge_full,capacity,status')"),
        )
        .arg(
            Arg::new("name")
                .value_name("BAT")
                .short('n')
                .long("name")
                .help("Specify battery name in the case of multiple batteries (e.g. 'BAT1'). Defaults to lowest-numbered battery"),
        )
        .arg(
            Arg::new("format_output")
                .short('f')
                .long("format-output")
                .value_parser(value_parser!(FormatOutputType))
                .value_name("FORMAT_TYPE")
                .default_value("no_symbols")
                .help("Specify how the output fields should be formatted"),
        )
}

struct BatteryContext<'a> {
    battery_name: String,
    format_output: &'a FormatOutputType,
    separator: String,
    output_as_json: bool,
}

struct BatterySubcommand<'a> {
    batteries: Batteries,
    info_names: &'a Vec<&'a BatteryInfoName>,
    context: BatteryContext<'a>,
}

#[derive(Default)]
struct BatteryOutput<'a> {
    fields: Vec<Field<'a>>,
    separator: Option<String>,
}

impl<'a> BatteryOutput<'a> {
    pub fn new(fields: Vec<Field<'a>>, separator: Option<String>) -> Self {
        Self { fields, separator }
    }
}

impl<'a> Display for BatteryOutput<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.fields
                .iter()
                .map(|f| match &f.value {
                    FieldValue::I32(v) => v.to_string(),
                    FieldValue::U64(v) => v.to_string(),
                    FieldValue::F32(v) => v.to_string(),
                    FieldValue::String(v) => v.to_string(),
                    FieldValue::Timestamp(timestamp) => timestamp.to_string(),
                })
                .collect::<Vec<_>>()
                .join(&self.separator.clone().unwrap())
        )
    }
}

impl<'a> Serialize for BatteryOutput<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_map(Some(self.fields.len()))?;
        for field in &self.fields {
            match &field.value {
                // TODO: fix duplicate code
                FieldValue::I32(v) => state.serialize_entry(field.label, v)?,
                FieldValue::U64(v) => state.serialize_entry(field.label, v)?,
                FieldValue::F32(v) => state.serialize_entry(field.label, v)?,
                FieldValue::String(v) => state.serialize_entry(field.label, v)?,
                FieldValue::Timestamp(v) => state.serialize_entry(field.label, v)?,
            }
        }
        state.end()
    }
}

impl<'a> BatterySubcommand<'a> {
    fn new(
        batteries: Batteries,
        info_names: &'a Vec<&'a BatteryInfoName>,
        context: BatteryContext<'a>,
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

        for filename in &files_to_watch {
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
        let battery = self
            .batteries
            .get_battery(&self.context.battery_name)
            .unwrap();

        let mut battery_output =
            BatteryOutput::new(Vec::with_capacity(1), Some(self.context.separator.clone()));

        for info_name in self.info_names.iter() {
            let field_value = match info_name {
                BatteryInfoName::ChargeNow => {
                    let value = battery.get_charge_now().unwrap();
                    match self.context.format_output {
                        FormatOutputType::Raw => FieldValue::I32(value),
                        FormatOutputType::NoSymbols => FieldValue::I32(value),
                        FormatOutputType::Formatted => {
                            FieldValue::String(format!("{}mAh", as_amps(value)))
                        }
                    }
                }
                BatteryInfoName::Capacity => {
                    let value = battery.get_capacity().unwrap();
                    match self.context.format_output {
                        FormatOutputType::Raw => FieldValue::F32(value),
                        FormatOutputType::NoSymbols => FieldValue::F32(value * 100.0),
                        FormatOutputType::Formatted => {
                            FieldValue::String(format!("{}%", value * 100.0))
                        }
                    }
                }
                BatteryInfoName::ChargeFull => {
                    let value = battery.get_charge_full();
                    match self.context.format_output {
                        FormatOutputType::Raw => FieldValue::I32(value),
                        FormatOutputType::NoSymbols => FieldValue::I32(as_amps(value)),
                        FormatOutputType::Formatted => {
                            FieldValue::String(format!("{}mAh", as_amps(value)))
                        }
                    }
                }
                BatteryInfoName::CurrentNow => {
                    let value = battery.get_current_now().unwrap();
                    match self.context.format_output {
                        FormatOutputType::Raw => FieldValue::I32(value),
                        FormatOutputType::NoSymbols => FieldValue::I32(as_amps(value)),
                        FormatOutputType::Formatted => {
                            FieldValue::String(format!("{}mA", as_amps(value)))
                        }
                    }
                }
                BatteryInfoName::TimeRemaining => {
                    let value = match battery.get_status().unwrap() {
                        BatteryStatus::Unknown
                        | BatteryStatus::NotCharging
                        | BatteryStatus::Full => 0,
                        BatteryStatus::Charging | BatteryStatus::Discharging => {
                            battery.get_time_remaining().unwrap()
                        }
                    };
                    match self.context.format_output {
                        FormatOutputType::Raw => FieldValue::U64(value),
                        FormatOutputType::NoSymbols => FieldValue::Timestamp(value.as_timestamp()),
                        FormatOutputType::Formatted => {
                            FieldValue::String(value.as_timestamp().to_string())
                        }
                    }
                }
                BatteryInfoName::Status => {
                    FieldValue::String(battery.get_status().unwrap().to_string())
                }
            };
            let field = Field::new(info_name.as_str(), field_value);
            battery_output.fields.push(field);
        }

        if self.context.output_as_json {
            serde_json::to_string(&battery_output).expect("always valid")
        } else {
            battery_output.to_string()
        }
    }
}

// TODO: proper error handling
pub fn exec(args: &ArgMatches) {
    let format_output = args
        .get_one::<FormatOutputType>("format_output")
        .expect("has a default value");
    let separator = args
        .get_one::<String>("separator")
        .expect("has a default value");
    let output_as_json = args.get_one::<bool>("json").expect("has a default value");
    let input_info_names = args
        .get_many::<BatteryInfoName>("info_names")
        .expect("has a default value")
        .collect::<Vec<_>>();
    let default_battery_name = get_main_battery_name().unwrap();
    let battery_name = args
        .get_one::<String>("name")
        .unwrap_or(&default_battery_name);

    let batteries = Batteries::init().unwrap();

    let mut battery_subcommand = BatterySubcommand::new(
        batteries,
        &input_info_names,
        BatteryContext {
            battery_name: battery_name.to_string(),
            format_output,
            separator: separator.to_string(),
            output_as_json: *output_as_json,
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

#[inline]
fn as_amps(micro_amps: i32) -> i32 {
    micro_amps / 1000
}
