use std::env;

use application::Application;
use clap::Parser;
use config::CmdConfig;
use util::log_util::Log;

mod application;
mod buffer;
mod config;
mod errors;
mod modules;
mod plugin;
mod util;
mod view;
mod workspace;

#[macro_use]
extern crate log;
extern crate simplelog;

use crate::errors::*;

pub static mut APPLICATION: Option<Application> = None;

pub fn get_application() -> &'static mut Application {
    unsafe { APPLICATION.as_mut().unwrap()}
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let config = CmdConfig::parse();
    Log::init(config.level)?;

    let application = Application::new(&args)?;

    unsafe {
        APPLICATION = Some(application);
    }

    get_application().run()
}
