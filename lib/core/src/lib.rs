use clap::Command;

pub trait Script {
    fn build_command() -> Command;
}
