use clap::{ArgMatches, Command};
use gi_media_player::foo;
use crate::commands::{FormatOutputType, SubCommandExt};

pub fn cli() -> Command {
    Command::new("media")
        .about("Scripts for media player info")
        .common_args()
}

struct MediaContext<'a> {
    battery_name: &'a str,
    format_output: &'a FormatOutputType,
    separator: &'a str,
    output_as_json: bool,
}

struct MediaSubcommand;

impl MediaSubcommand {
    fn new() -> Self {
        todo!()
    }

    fn watch(&mut self) {
        todo!()
    }

    fn poll(&self, milliseconds: u64) {
        todo!()
    }

    fn get_output_string(&self) -> String {
        todo!()
    }
}

pub async fn exec(args: &ArgMatches) {
    foo();
}
