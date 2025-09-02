use std::{fmt::Display, str::FromStr};

use clap::{Arg, ArgAction, Command, value_parser};
use gi_core::Timestamp;
use serde::{ser::SerializeMap, Serialize};

pub mod battery;

pub trait SubCommandExt {
    fn arg_watch(self) -> Self;
    fn arg_poll(self) -> Self;
    fn arg_separator(self) -> Self;
    fn arg_json(self) -> Self;
    fn common_args(self) -> Self;
}

impl SubCommandExt for Command {
    fn arg_watch(self) -> Self {
        self.arg(
            Arg::new("watch")
                .short('w')
                .long("watch")
                .conflicts_with("poll")
                .action(ArgAction::SetTrue)
                .help("Outputs when info changes"),
        )
    }

    fn arg_poll(self) -> Self {
        self.arg(
            Arg::new("poll")
                .short('p')
                .long("poll")
                .conflicts_with("watch")
                .value_parser(value_parser!(u64))
                .value_name("MILLISECONDS")
                .help("Outputs after every interval"),
        )
    }

    fn arg_separator(self) -> Self {
        self.arg(
            Arg::new("separator")
                .short('s')
                .long("separator")
                .conflicts_with("json")
                .value_name("STRING")
                .default_value(" ")
                .help("Character or string to use for separating output infos"),
        )
    }

    fn arg_json(self) -> Self {
        self.arg(
            Arg::new("json")
                .short('j')
                .long("json")
                .conflicts_with("separator")
                .action(ArgAction::SetTrue)
                .help("Formats output into json"),
        )
    }

    fn common_args(self) -> Self {
        self.arg_watch().arg_poll().arg_separator().arg_json()
    }
}

pub enum FieldValue {
    I32(i32),
    U64(u64),
    F32(f32),
    String(String),
    Timestamp(Timestamp),
}

pub struct Field<'a> {
    pub label: &'a str,
    pub value: FieldValue,
}

impl<'a> Field<'a> {
    pub fn new(label: &'a str, value: FieldValue) -> Self {
        Self { label, value }
    }
}

#[derive(Clone)]
pub enum FormatOutputType {
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

#[derive(Default)]
pub struct Output<'a> {
    pub fields: Vec<Field<'a>>,
    pub separator: Option<&'a str>,
}

impl<'a> Output<'a> {
    pub fn new(fields: Vec<Field<'a>>, separator: Option<&'a str>) -> Self {
        Self { fields, separator }
    }
}

impl<'a> Display for Output<'a> {
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
                .join(self.separator.expect("should always have a separator"))
        )
    }
}

impl<'a> Serialize for Output<'a> {
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
