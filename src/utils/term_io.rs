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
}
