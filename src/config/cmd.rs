use clap::Parser;
use log::LevelFilter;

#[derive(Parser)]
#[command(name = "held")]
#[command(author = "heyicong@dragonos.org")]
#[command(version = "1.0")]
#[command(about = "a termial editor", long_about = None)]
pub struct CmdConfig {
    /// open file
    pub file: Option<String>,

    /// log level
    #[arg(value_enum, short, long, default_value = "warn")]
    pub level: LevelFilter,
}
