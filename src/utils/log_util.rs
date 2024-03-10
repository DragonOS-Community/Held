use std::{fs::File, io};

use log::LevelFilter;
use simplelog::{CombinedLogger, WriteLogger};

pub struct Log;

impl Log {
    pub fn init(level: LevelFilter) -> io::Result<()> {
        CombinedLogger::init(vec![
            // TermLogger::new(
            //     level.to_simplelog_filter(),
            //     simplelog::Config::default(),
            //     simplelog::TerminalMode::default(),
            //     simplelog::ColorChoice::Auto,
            // ),
            WriteLogger::new(
                level,
                simplelog::Config::default(),
                File::create("held.log")?,
            ),
        ])
        .unwrap();

        Ok(())
    }
}
