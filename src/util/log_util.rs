use std::{fs::File, io};

use log::LevelFilter;
use simplelog::{CombinedLogger, WriteLogger};

pub struct Log;

impl Log {
    pub fn init(level: LevelFilter) -> io::Result<()> {
        CombinedLogger::init(vec![
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
