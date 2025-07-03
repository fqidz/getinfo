use std::{fs, path::PathBuf, str::FromStr};

use gi_core::Error;

const SYS_BATTERIES_PATH: &str = "/sys/class/power_supply";

type Percentage = f32;
type AmpereHours = i32;

pub struct Batteries {
    items: Vec<Battery>,
}

pub struct Battery {
    pub path: PathBuf,
    pub name: String,
    pub charge_full: AmpereHours,
}

#[derive(Clone)]
pub enum BatteryInfoName {
    ChargeNow,
    ChargeNowPercentage,
    ChargeFull,
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
            _ => Err(Self::Err::InvalidInfoName {
                name: s.to_string(),
            }),
        }
    }
}
