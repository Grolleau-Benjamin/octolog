use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "octolog",
    version,
    about = "Multi-serial-port log monitor (CLI/TUI)",
    after_help = "Examples:\n  octolog --list\n  octolog -p /dev/ttyACM0:115200:Sensor -p /dev/ttyACM1:TFM\n  octolog -p /dev/ttyUSB0 --baud 9600\n"
)]
pub struct CliArgs {
    /// Print available ports and exit
    #[arg(long)]
    pub list: bool,

    /// Serial ports to monitor
    ///
    /// Format: path[:baudrate][:alias]
    ///
    /// Examples:
    ///   -p /dev/ttyACM0:115200:Sensor
    ///   -p /dev/ttyACM1:TFM
    #[arg(short = 'p', long = "port", value_name = "PORT", num_args = 1..)]
    pub port: Vec<String>,

    /// Default baudrate (used when a port does not specify one)
    #[arg(short = 'b', long, value_name = "BAUD", default_value_t = 115200)]
    pub baud: u32,

    /// Write rendered output to a file
    #[arg(short = 'o', long = "output", value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Highlight patterns (can be repeated)
    ///
    /// Examples:
    ///   --highlight ERROR --highlight WARN
    #[arg(long = "highlight", value_name = "PATTERN", num_args = 1..)]
    pub highlight: Vec<String>,

    /// Only keep log lines containing this substring
    ///
    /// Example:
    ///   --filter "AT+"
    #[arg(long = "filter", value_name = "TEXT")]
    pub filter: Option<String>,

    /// Drop log lines containing these substrings (can be repeated)
    ///
    /// Example:
    ///   --exclude "DEBUG" --exclude "heartbeat"
    #[arg(long = "exclude", value_name = "TEXT", num_args = 1..)]
    pub exclude: Vec<String>,
}
