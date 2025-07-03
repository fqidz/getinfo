use std::{fmt::Display, fs, path::PathBuf, str::FromStr, time::Duration};

use gi_core::Error;

const SYS_BATTERIES_PATH: &str = "/sys/class/power_supply";

type Percentage = f32;
type AmpereHours = i32;
type Ampere = i32;

pub struct Batteries {
    items: Vec<Battery>,
}

pub struct Battery {
    pub path: PathBuf,
    pub name: String,
    pub charge_full: AmpereHours,
}

#[derive(Clone, Eq, PartialEq)]
pub enum BatteryInfoName {
    ChargeNow,
    ChargeNowPercentage,
    ChargeFull,
    CurrentNow,
    Status,
    TimeRemaining,
}

#[derive(Eq, PartialEq)]
pub enum BatteryStatus {
    Full,
    Charging,
    NotCharging,
    Discharging,
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
                .parse::<AmpereHours>()?;

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
                items: battery_infos,
            })
        }
    }

    pub fn get_battery(&self, battery_name: &str) -> Result<&Battery, Error> {
        if let Some(battery) = self
            .items
            .iter()
            .find(|battery| battery.name == *battery_name)
        {
            return Ok(battery);
        }
        Err(Error::BatteryNotFound {
            name: battery_name.to_string(),
        })
    }

    pub fn get_charge_full_single(&self, battery_name: &str) -> Result<AmpereHours, Error> {
        Ok(self.get_battery(battery_name)?.charge_full)
    }

    pub fn get_charge_now_single(&self, battery_name: &str) -> Result<AmpereHours, Error> {
        Ok(
            fs::read_to_string(self.get_battery(battery_name)?.path.join("charge_now"))?
                .trim_end()
                .parse::<AmpereHours>()?,
        )
    }

    /// Percentage from 0.0 to 1.0 inclusive
    pub fn get_charge_percentage_single(&self, battery_name: &str) -> Result<Percentage, Error> {
        let battery = self.get_battery(battery_name)?;
        let charge_full = battery.charge_full as Percentage;
        let charge = (self.get_charge_now_single(battery_name)?) as Percentage;
        Ok(charge / charge_full)
    }

    pub fn get_status_single(&self, battery_name: &str) -> Result<BatteryStatus, Error> {
        fs::read_to_string(self.get_battery(battery_name)?.path.join("status"))?
            .trim_end()
            .parse::<BatteryStatus>()
    }

    pub fn get_current_now_single(&self, battery_name: &str) -> Result<Ampere, Error> {
        Ok(
            fs::read_to_string(self.get_battery(battery_name)?.path.join("current_now"))?
                .trim_end()
                .parse::<Ampere>()?,
        )
    }

    pub fn get_time_remaining_single(&self, battery_name: &str) -> Result<Duration, Error> {
        let charge_now = self.get_charge_now_single(battery_name)?;
        let current_now = self.get_current_now_single(battery_name)?;
        let hours = charge_now as f32 / current_now as f32;
        if hours.is_finite() {
            let secs = hours * 3600.0;
            Ok(Duration::from_secs_f32(secs))
        } else {
            Ok(Duration::default())
        }
    }
}

#[derive(Default)]
pub struct Timestamp {
    hours: u64,
    minutes: u8,
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

pub trait DurationExt {
    fn from_hours_f64(hours: f64) -> Duration;
    fn display_as_timestamp(&self) -> Timestamp;
}

impl DurationExt for Duration {
    fn from_hours_f64(hours: f64) -> Duration {
        Duration::from_secs_f64(hours * 3600.0)
    }

    fn display_as_timestamp(&self) -> Timestamp {
        let total_secs = self.as_secs();

        let hours = total_secs / 3600;
        let minutes = total_secs / 60 % 60;
        let seconds = total_secs % 60;

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
            "Charging" => Ok(Self::Charging),
            "Discharging" => Ok(Self::Discharging),
            "Full" => Ok(Self::Full),
            "Not Charging" => Ok(Self::NotCharging),
            _ => Err(Self::Err::InvalidBatteryStatus {
                status: s.to_string(),
            }),
        }
    }
}

impl Display for BatteryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BatteryStatus::Full => write!(f, "Full"),
            BatteryStatus::Charging => write!(f, "Charging"),
            BatteryStatus::Discharging => write!(f, "Discharging"),
            BatteryStatus::NotCharging => write!(f, "Not Charging"),
        }
    }
}

impl FromStr for BatteryInfoName {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "charge_now" | "charge" => Ok(Self::ChargeNow),
            "charge_now_percentage"
            | "charge_percentage"
            | "now_percentage"
            | "percentage"
            | "percent" => Ok(Self::ChargeNowPercentage),
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
