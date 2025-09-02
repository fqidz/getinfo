use std::fmt::Display;

use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("No batteries found in {}", .path)]
    NoBatteriesFound { path: String },

    #[error("Invalid info name \"{}\"", .name)]
    InvalidInfoName { name: String },

    #[error("Invalid battery status \"{}\". Expected \"Charging\", \"Discharging\", \"Not Charging\", or \"Full\" ", .status)]
    InvalidBatteryStatus { status: String },

    #[error("Invalid path: {}", .path)]
    InvalidPath { path: String },
}

pub type Seconds = u64;

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
