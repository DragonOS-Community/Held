use std::{
    fmt::Display,
    io::{self, stdout, Write},
};

use crossterm::{style::Print, ExecutableCommand};

pub struct TermIO;

impl TermIO {
    pub fn write_str<D: Display>(str: D) -> io::Result<()> {
        stdout().execute(Print(str)).unwrap().flush()?;
        Ok(())
    }
    pub fn enqueue(cmd: impl Command) -> io::Result<()> {
        match stdout().queue(cmd) {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Failed to enqueue command Error: {}", e);
                Err(io::Error::new(io::ErrorKind::Other, e))
            },
        }
    }
    pub fn execute(cmd: impl Command) -> io::Result<()> {
        match stdout().execute(cmd) {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Failed to execute command Error: {}", e);
                Err(io::Error::new(io::ErrorKind::Other, e))
            },
        }
    }
    pub fn execute_queue() -> io::Result<()> {
        stdout().flush()?;
        Ok(())
    }
}
