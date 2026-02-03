use crate::{
    cli::CliArgs,
    core::{
        AppError, AppResult,
        port_spec::{PortSpec, ResolvedPortSpec},
    },
};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub list: bool,
    pub ports: Vec<ResolvedPortSpec>,
    pub baud: u32,
    pub output: Option<PathBuf>,
    pub highlight: Vec<String>,
    pub filter: Option<String>,
    pub exclude: Vec<String>,
    pub runtime: RuntimeConfig,
}

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub event_bus_capacity: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            event_bus_capacity: 1024,
        }
    }
}

impl TryFrom<CliArgs> for Config {
    type Error = AppError;

    fn try_from(args: CliArgs) -> AppResult<Self> {
        if !args.list && args.port.is_empty() {
            return Err(AppError::Config(
                "no ports specified (use -p/--port or --list)".to_string(),
            ));
        }

        let parsed = args
            .port
            .iter()
            .map(|raw| {
                raw.parse::<PortSpec>()
                    .map_err(|e| AppError::PortInvalidFormat(e.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let ports = parsed
            .into_iter()
            .map(|p| p.resolve(args.baud))
            .collect::<Vec<_>>();

        Ok(Self {
            list: args.list,
            ports,
            baud: args.baud,
            output: args.output,
            highlight: args.highlight,
            filter: args.filter,
            exclude: args.exclude,
            runtime: RuntimeConfig::default(),
        })
    }
}
