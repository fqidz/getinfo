use clap::{Arg, ArgAction, Command, value_parser};

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
