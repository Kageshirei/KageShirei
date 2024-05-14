#[macro_use]
extern crate log;

use clap::Parser;

use crate::cli_interface::{CliArguments, Commands, CompileArguments, CompileSubcommands};

mod cli_interface;

fn setup_logging(debug_level: u8) -> anyhow::Result<()> {
    let mut base_config = fern::Dispatch::new()
        .format(|out, message, record| {
            let level_padding = if record.level().to_string().len() < 5 {
                " ".repeat(5 - record.level().to_string().len() + 1).to_string()
            } else {
                " ".to_string()
            };

            let colors = fern::colors::ColoredLevelConfig::new()
                .info(fern::colors::Color::Green)
                .warn(fern::colors::Color::Yellow)
                .error(fern::colors::Color::Red)
                .debug(fern::colors::Color::Blue)
                .trace(fern::colors::Color::Magenta);

            let additional_info = if record.level() > log::LevelFilter::Debug {
                format!(" [{}:{}]", record.file().unwrap_or(""), record.line().unwrap_or(0))
            } else {
                "".to_string()
            };

            out.finish(format_args!(
                "[{}]{}[{}]{} {}",
                colors.color(record.level()),
                level_padding,
                humantime::format_rfc3339_seconds(std::time::SystemTime::now()),
                additional_info,
                message
            ))
        })
        .level(log::LevelFilter::Trace)
        .chain(std::io::stdout());

    base_config = match debug_level {
        0 => base_config.level(log::LevelFilter::Info),
        1 => base_config.level(log::LevelFilter::Debug),
        _ => base_config.level(log::LevelFilter::Trace),
    };

    base_config.apply()?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = CliArguments::parse();

    setup_logging(args.debug)?;
    trace!("Parsed arguments: {:?}", args);

    match args.command {
        Commands::Compile(compile_args) => {
            match compile_args.command {
                CompileSubcommands::Agent => {
                    unimplemented!("Agent compilation not implemented yet");
                }
                CompileSubcommands::Gui => {
                    info!("Compiling C2 GUI");
                }
            }
        }
    }

    Ok(())
}
