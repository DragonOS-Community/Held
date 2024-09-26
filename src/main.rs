#![feature(duration_millis_float)]

use std::{fs::File, io};

use app::Application;
use clap::Parser;
use config::{appconfig::DeserializeAppOption, cmd::CmdConfig};
use utils::log_util::Log;

mod app;
mod buffer;
mod config;
mod errors;
mod modules;
mod plugin;
mod util;
mod utils;
mod view;
mod workspace;

#[macro_use]
extern crate log;
extern crate simplelog;

fn main() -> io::Result<()> {
    let config = CmdConfig::parse();
    Log::init(config.level)?;

    let setting;

    let file = File::open("config.yaml");
    if file.is_err() {
        setting = DeserializeAppOption::default();
    } else {
        setting = serde_yaml::from_reader::<File, DeserializeAppOption>(file?).unwrap_or_default();
    }

    Application::new(config.file, setting.to_app_setting())?.run()
}
