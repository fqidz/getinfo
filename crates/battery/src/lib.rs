// https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-class-power
use std::{fmt::Display, fs, io, path::PathBuf, str::FromStr};

use gi_core::Error;
use serde::Serialize;


const SYS_BATTERIES_PATH: &str = "/sys/class/power_supply";

pub type Capacity = f32;
pub type MicroAmpHours = i32;
pub type MicroAmp = i32;
pub type Seconds = u64;

pub struct Batteries {
    pub main_battery_name: String,
    items: Vec<Battery>,
}

pub struct Battery {
    pub path: PathBuf,
    pub name: String,
    pub charge_full: MicroAmpHours,
}

#[derive(Clone, Eq, PartialEq)]
pub enum BatteryInfoName {
    ChargeFull,
    ChargeNow,
    Capacity,
    CurrentNow,
    Status,
    TimeRemaining,
}

#[derive(Eq, PartialEq)]
pub enum BatteryStatus {
    Unknown,
    Charging,
    Discharging,
    NotCharging,
    Full,
}

impl Battery {
    pub fn get_charge_full(&self) -> MicroAmpHours {
        self.charge_full
    }

    pub fn get_charge_now(&self) -> Result<MicroAmpHours, Error> {
        Ok(self
            .read_from_sysfs("charge_now")?
            .parse::<MicroAmpHours>()?)
    }

    pub fn get_capacity(&self) -> Result<Capacity, Error> {
        Ok(self.get_charge_now()? as f32 / self.get_charge_full() as f32)
    }

    pub fn get_current_now(&self) -> Result<MicroAmp, Error> {
        Ok(self.read_from_sysfs("current_now")?.parse::<MicroAmp>()?)
    }

    pub fn get_status(&self) -> Result<BatteryStatus, Error> {
        self.read_from_sysfs("status")?.parse::<BatteryStatus>()
    }

    pub fn get_time_remaining(&self) -> Result<Seconds, Error> {
        let hours = (self.get_charge_now()? as f32 / self.get_current_now()? as f32).abs();
        if hours.is_infinite() {
            return Ok(0);
        }
        let secs = hours * 3600.0;

        // Discard milliseconds
        Ok(secs as u64)
    }

    fn read_from_sysfs(&self, file_name: &str) -> io::Result<String> {
        let file_path = self.path.join(file_name);
        Ok(fs::read_to_string(file_path)?.trim_end().to_string())
    }
}

impl Batteries {
    pub fn init() -> Result<Batteries, Error> {
        let sys_dir = fs::read_dir(SYS_BATTERIES_PATH)?;
        let battery_dirs = sys_dir
            .filter_map(|dir| {
                if let Ok(dir) = dir {
                    return Some(dir);
                }
                None
            })
            .collect::<Vec<_>>();

        let mut battery_infos = Vec::with_capacity(1);

        for battery_dir in battery_dirs {
            let path = battery_dir.path();
            let name = path
                .file_name()
                .expect("Directory name should always be valid.")
                .to_string_lossy()
                .to_string();

            if !name.starts_with("BAT") {
                continue;
            }

            let charge_full = fs::read_to_string(path.join("charge_full"))?
                .trim_end()
                .parse::<MicroAmpHours>()?;

            battery_infos.push(Battery {
                path,
                name,
                charge_full,
            });
        }

        if battery_infos.is_empty() {
            Err(Error::NoBatteriesFound {
                path: SYS_BATTERIES_PATH.to_string(),
            })
        } else {
            Ok(Batteries {
                main_battery_name: get_main_battery_name()?,
                items: battery_infos,
            })
        }
    }

    pub fn get_main_battery(&self) -> Option<&Battery> {
        self.items
            .iter()
            .find(|battery| battery.name == *self.main_battery_name)
    }

    pub fn get_battery(&self, battery_name: &str) -> Option<&Battery> {
        self.items
            .iter()
            .find(|battery| battery.name == *battery_name)
    }
}

#[derive(Default, Serialize)]
pub struct Timestamp {
    #[serde(rename = "h")]
    hours: u64,

    #[serde(rename = "m")]
    minutes: u8,

    #[serde(rename = "s")]
    seconds: u8,
}

impl Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:0>2}:{:0>2}:{:0>2}",
            self.hours, self.minutes, self.seconds
        )
    }
}

pub trait AsTimestamp {
    fn as_timestamp(&self) -> Timestamp;
}

impl AsTimestamp for Seconds {
    fn as_timestamp(&self) -> Timestamp {
        let hours = self / 3600;
        let minutes = self / 60 % 60;
        let seconds = self % 60;

        Timestamp {
            hours,
            minutes: minutes.try_into().expect("max value is 59"),
            seconds: seconds.try_into().expect("max value is 59"),
        }
    }
}

pub fn get_main_battery_name() -> Result<String, Error> {
    let sys_dir = fs::read_dir(SYS_BATTERIES_PATH)?;
    let battery_dirs = sys_dir
        .filter_map(|dir| {
            if let Ok(dir) = dir {
                return Some(dir);
            }
            None
        })
        .collect::<Vec<_>>();

    let mut battery_names = Vec::with_capacity(1);

    for battery_dir in battery_dirs {
        let path = battery_dir.path();
        let name = path
            .file_name()
            .expect("Directory name should always be valid.")
            .to_string_lossy()
            .to_string();

        if name.starts_with("BAT") {
            battery_names.push(name);
        }
    }

    if battery_names.len() == 1 {
        Ok(battery_names[0].clone())
    } else if battery_names.len() > 1 {
        battery_names.sort_unstable_by(|a, b| {
            let a_num = a
                .strip_prefix("BAT")
                .expect("Directory starts with 'BAT'")
                .parse::<u8>()
                .expect("It's always BAT[number]");
            let b_num = b
                .strip_prefix("BAT")
                .expect("Directory starts with 'BAT'")
                .parse::<u8>()
                .expect("It's always BAT[number]");

            a_num.cmp(&b_num)
        });
        Ok(battery_names[0].clone())
    } else {
        Err(Error::NoBatteriesFound {
            path: SYS_BATTERIES_PATH.to_string(),
        })
    }
}

impl FromStr for BatteryStatus {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Unknown" => Ok(Self::Unknown),
            "Charging" => Ok(Self::Charging),
            "Discharging" => Ok(Self::Discharging),
            "Not charging" => Ok(Self::NotCharging),
            "Full" => Ok(Self::Full),
            _ => Err(Self::Err::InvalidBatteryStatus {
                status: s.to_string(),
            }),
        }
    }
}

impl BatteryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            BatteryStatus::Full => "Full",
            BatteryStatus::Charging => "Charging",
            BatteryStatus::Discharging => "Discharging",
            BatteryStatus::NotCharging => "Not charging",
            BatteryStatus::Unknown => "Unknown",
        }
    }
}

impl Display for BatteryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl BatteryInfoName {
    pub fn as_str(&self) -> &'static str {
        match self {
            BatteryInfoName::ChargeFull => "charge_full",
            BatteryInfoName::ChargeNow => "charge_now",
            BatteryInfoName::Capacity => "capacity",
            BatteryInfoName::CurrentNow => "current_now",
            BatteryInfoName::Status => "status",
            BatteryInfoName::TimeRemaining => "time_remaining",
        }
    }
}

impl FromStr for BatteryInfoName {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "charge_now" | "charge" => Ok(Self::ChargeNow),
            "capacity" | "charge_percentage" | "percentage" | "percent" => Ok(Self::Capacity),
            "charge_full" => Ok(Self::ChargeFull),
            "current_now" | "current" => Ok(Self::CurrentNow),
            "time_remaining" | "remaining" | "time" => Ok(Self::TimeRemaining),
            "status" => Ok(Self::Status),
            _ => Err(Self::Err::InvalidInfoName {
                name: s.to_string(),
            }),
        }
    }
}

impl Display for BatteryInfoName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
