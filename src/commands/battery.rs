use std::collections::HashSet;
use std::str::FromStr;
use std::{sync::mpsc, time::Duration};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use gi_battery::{
    AsTimestamp, Batteries, BatteryInfoName, BatteryStatus, Timestamp, get_main_battery_name,
};
use notify::{Config, Event, PollWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use serde::ser::SerializeMap;

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
        .arg(
            Arg::new("format_output")
                .short('f')
                .long("format-output")
                .value_parser(value_parser!(FormatOutputType))
                .value_name("FORMAT_TYPE")
                .default_value("no_symbols")
                .help("Specify how the output fields should be formatted"),
        )
        .arg(
            Arg::new("json")
                .short('j')
                .long("json")
                .conflicts_with("separator")
                .action(ArgAction::SetTrue)
                .help("Formats output into json"),
        )
}

enum FieldValue {
    I32(i32),
    U64(u64),
    F32(f32),
    String(String),
    Timestamp(Timestamp),
}

struct Field<'a> {
    label: &'a str,
    value: FieldValue,
}

impl<'a> Field<'a> {
    pub fn new(label: &'a str, value: FieldValue) -> Self {
        Self { label, value }
    }
}

#[derive(Clone)]
enum FormatOutputType {
    Raw,
    NoSymbols,
    Formatted,
}

impl FromStr for FormatOutputType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "raw" => Ok(Self::Raw),
            "no_symbols" => Ok(Self::NoSymbols),
            "formatted" => Ok(Self::Formatted),
            _ => Err(format!("Invalid format-output type: {}", s)),
        }
    }
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
struct BatteryOutput<'a>(Vec<Field<'a>>);

// Try to parse each of the fields as their actual type, and if that fails, fallback to the
// original string. This is useful for, example, turning numbers into JSON numbers so that users
// can do mathematical operations and what not.
// TODO: Option for user to specify field key
// TODO: Either turn this into a derive macro or convert the 'if let Some(v) ...' into a macro_rules
impl<'a> Serialize for BatteryOutput<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_map(Some(self.0.len()))?;
        for field in &self.0 {
            match &field.value {
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
    fn init(
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

        let mut battery_output = BatteryOutput::default();
        for info_name in self.info_names.iter() {
            match info_name {
                BatteryInfoName::ChargeNow => {
                    let value = battery.get_charge_now().unwrap();
                    let field_value = match self.context.format_output {
                        FormatOutputType::Raw => FieldValue::I32(value),
                        FormatOutputType::NoSymbols => FieldValue::I32(value / 1000),
                        FormatOutputType::Formatted => {
                            FieldValue::String(format!("{}mAh", value / 1000))
                        }
                    };
                    let field = Field::new(info_name.as_str(), field_value);
                    battery_output.0.push(field);
                }
                BatteryInfoName::Capacity => {
                    let value = battery.get_capacity().unwrap();
                    let field_value = match self.context.format_output {
                        FormatOutputType::Raw => FieldValue::F32(value),
                        FormatOutputType::NoSymbols => FieldValue::F32(value * 100.0),
                        FormatOutputType::Formatted => {
                            FieldValue::String(format!("{}%", value * 100.0))
                        }
                    };
                    let field = Field::new(info_name.as_str(), field_value);
                    battery_output.0.push(field);
                }
                BatteryInfoName::ChargeFull => {
                    let value = battery.get_charge_full();
                    let field_value = match self.context.format_output {
                        FormatOutputType::Raw => FieldValue::I32(value),
                        FormatOutputType::NoSymbols => FieldValue::I32(value / 1000),
                        FormatOutputType::Formatted => {
                            FieldValue::String(format!("{}mAh", value / 1000))
                        }
                    };
                    let field = Field::new(info_name.as_str(), field_value);
                    battery_output.0.push(field);
                }
                BatteryInfoName::CurrentNow => {
                    let value = battery.get_current_now().unwrap();
                    let field_value = match self.context.format_output {
                        FormatOutputType::Raw => FieldValue::I32(value),
                        FormatOutputType::NoSymbols => FieldValue::I32(value / 1000),
                        FormatOutputType::Formatted => {
                            FieldValue::String(format!("{}mA", value / 1000))
                        }
                    };
                    let field = Field::new(info_name.as_str(), field_value);
                    battery_output.0.push(field);
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
                    // battery.get_time_remaining().unwrap()
                    let field_value = match self.context.format_output {
                        FormatOutputType::Raw => FieldValue::U64(value),
                        FormatOutputType::NoSymbols => FieldValue::Timestamp(value.as_timestamp()),
                        FormatOutputType::Formatted => {
                            FieldValue::String(value.as_timestamp().to_string())
                        }
                    };
                    let field = Field::new(info_name.as_str(), field_value);
                    battery_output.0.push(field);
                }
                BatteryInfoName::Status => {
                    let field_value = FieldValue::String(battery.get_status().unwrap().to_string());
                    let field = Field::new(info_name.as_str(), field_value);
                    battery_output.0.push(field);
                }
            }
        }
        serde_json::to_string(&battery_output).expect("always valid")
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

    let mut battery_subcommand = BatterySubcommand::init(
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
